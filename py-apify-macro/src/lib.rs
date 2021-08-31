extern crate proc_macro;
use glob::glob;
use log::info;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
use quote::quote;
use std::path::PathBuf;
use syn::{parse::Parser, punctuated::Punctuated, LitStr, Token};
use uuid;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct PythonFile {
    path: PathBuf,
    file_name: String,
    file_stem: String,
    uuid: String,
}

impl PythonFile {
    fn new(input: PathBuf) -> Self {
        PythonFile {
            file_name: input.file_name().expect("").to_str().expect("").to_string(),
            file_stem: input.file_stem().expect("").to_str().expect("").to_string(),
            path: input,
            uuid: Uuid::new_v4().to_simple().to_string(),
        }
    }

    #[cfg(not(feature = "no-check"))]
    fn check(&self) {
        let python_code = std::fs::read_to_string(self.path.clone())
            .expect("Something went wrong reading the file");

        let display_path = self.path.display().to_string();

        pyo3::prepare_freethreaded_python();

        pyo3::Python::with_gil(|py| {
            pyo3::types::PyModule::from_code(py, &python_code, &self.uuid, &self.uuid)
                .expect("failed to import PyModule");

            let nlp =
                pyo3::types::PyModule::import(py, &self.uuid).expect("failed to import PyModule");

            nlp
                .getattr("call")
                .expect(&format!("`call` function was not found in {:?}. Your python file must include a `call` function that returns json data:\n\ndef call(input):\n\tjson.dumps('{{'foo': 'bar'}}')\n\n", display_path));
        });
    }

    fn request_handler_ident(&self) -> Ident {
        Ident::new(&format!("route_{}", self.uuid), Span::call_site())
    }

    fn route_attribute(&self) -> Literal {
        Literal::string(&format!("/{}?<query>", self.file_stem))
    }

    fn hook_function_ident(&self) -> Ident {
        Ident::new(&format!("hook_{}", self.uuid), Span::call_site())
    }

    fn module_name(&self) -> Literal {
        Literal::string(&self.uuid)
    }
}

fn get_py_files(input: Vec<String>) -> Vec<PythonFile> {
    let mut python_files: Vec<PythonFile> = if !input.is_empty() {
        input
            .iter()
            .flat_map(|elem| {
                glob(elem)
                    .expect("Failed to read glob pattern")
                    .filter_map(|e| e.ok())
            })
            .map(|e| PythonFile::new(e))
            .collect::<Vec<PythonFile>>()
    } else {
        glob("./src/*.py")
            .expect("Failed to read glob pattern")
            .filter_map(|e| e.ok())
            .map(|elem| PythonFile::new(elem))
            .collect::<Vec<PythonFile>>()
    };

    python_files.sort_by(|a, b| a.file_name.partial_cmp(&b.file_name).unwrap());
    python_files.dedup_by(|a, b| a.file_name == b.file_name);

    #[cfg(not(feature = "no-check"))]
    {
        python_files.iter().for_each(|file| file.check());
    }

    return python_files;
}

/// Generates rocket request handler
fn gen_rocket_requests_handlers(input: &Vec<PythonFile>) -> TokenStream2 {
    let routes = input
        .iter()
        .map(|py_file| {
            let rocket_route_attribute = py_file.route_attribute();
            let route_ident = py_file.request_handler_ident();
            let hook_function_ident = py_file.hook_function_ident();

            return quote! {
                #[get(#rocket_route_attribute)]
                fn #route_ident(query: String) -> rocket::response::content::Json<String> {
                    return rocket::response::content::Json(format!(
                        "{}",
                        #hook_function_ident(pyo3::Python::acquire_gil().python(), query)
                    ));
                }
            }
            .into();
        })
        .collect::<Vec<TokenStream2>>();

    return quote! {
        #(#routes)*
    }
    .into();
}

/// Generates a hook function that call Python
fn gen_hooks(input: &Vec<PythonFile>) -> TokenStream2 {
    let hooks = input
        .iter()
        .map(|py_file| {
            let hook_function_ident = py_file.hook_function_ident();
            let module_name = py_file.module_name();
            let file_name = py_file.file_name.clone();

            return quote! {
                fn #hook_function_ident(py_lock: pyo3::Python, input: String) -> String {
                    let nlp = pyo3::types::PyModule::import(
                        py_lock,
                        #module_name,
                    )
                    .expect("failed to import PyModule");

                    nlp
                        .getattr("call")
                        .expect(&format!("`call` function was not found in {}. Your python file must include a `call` function that returns json data:\n\ndef call(input):\n\tjson.dumps('{{'foo': 'bar'}}')\n\n", #file_name))
                        .call1((input,))
                        .unwrap()
                        .extract()
                        .unwrap()
                }
            }
            .into();
        })
        .collect::<Vec<TokenStream2>>();

    return quote! {
        #(#hooks)*
    }
    .into();
}

fn gen_rocket_mount(input: &Vec<PythonFile>) -> TokenStream2 {
    let routes_idents = input
        .iter()
        .map(|py_file| py_file.request_handler_ident())
        .collect::<Vec<Ident>>();

    return quote! {
        rocket::build().mount("/", routes![#(#routes_idents),*])
    }
    .into();
}

fn gen_py_file_loader(input: &Vec<PythonFile>) -> TokenStream2 {
    let loaders = input
        .iter()
        .map(|py_file| {
            let module_name = py_file.module_name();
            let file_name = &py_file.file_name;

            return quote! {
                pyo3::types::PyModule::from_code(
                    py,
                    include_str!(#file_name),
                    #file_name,
                    #module_name,
                )
                .unwrap();
            }
            .into();
        })
        .collect::<Vec<TokenStream2>>();

    return quote! {
        #(#loaders)*
    }
    .into();
}

#[proc_macro]
pub fn apify(item: TokenStream) -> TokenStream {
    let args: Vec<String> = Punctuated::<LitStr, Token![,]>::parse_terminated
        .parse(item)
        .expect("invalid arguments")
        .into_iter()
        .map(|e| e.value())
        .collect();

    let python_files = get_py_files(args);

    let routes = gen_rocket_requests_handlers(&python_files);
    let mount = gen_rocket_mount(&python_files);
    let loader = gen_py_file_loader(&python_files);
    let hooks = gen_hooks(&python_files);

    return quote! {
        pyo3::prepare_freethreaded_python();

        pyo3::Python::with_gil(|py| {
            #loader
            #routes
            #hooks
            #mount
        })
    }
    .into();
}
