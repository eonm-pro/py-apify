use crate::hook::HookFunctionIdent;
use crate::python_file::PythonFile;
use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
use quote::quote;

use crate::form::FormIdent;

pub struct RouteAttribute<'a> {
    route_name: &'a str,
}

impl<'a> From<&'a PythonFile> for RouteAttribute<'a> {
    fn from(python_file: &'a PythonFile) -> RouteAttribute<'a> {
        RouteAttribute {
            route_name: &python_file.file_stem,
        }
    }
}

impl<'a> Into<TokenStream2> for RouteAttribute<'a> {
    fn into(self) -> TokenStream2 {
        let route_attribute = Literal::string(&format!("/{}?<query..>", self.route_name));

        quote! {
            #[get(#route_attribute)]
        }
    }
}

pub struct RequestHandlerIdent {
    ident: Ident,
}

impl From<&PythonFile> for RequestHandlerIdent {
    fn from(python_file: &PythonFile) -> RequestHandlerIdent {
        RequestHandlerIdent {
            ident: Ident::new(&format!("route_{}", python_file.uuid), Span::call_site()),
        }
    }
}

impl Into<Ident> for RequestHandlerIdent {
    fn into(self) -> Ident {
        self.ident
    }
}

pub struct RequestHandler<'a> {
    ident: RequestHandlerIdent,
    route_attribute: RouteAttribute<'a>,
    hook_function_ident: HookFunctionIdent,
    form_ident: FormIdent,
}

impl<'a> From<&'a PythonFile> for RequestHandler<'a> {
    fn from(python_file: &'a PythonFile) -> RequestHandler<'a> {
        RequestHandler {
            ident: python_file.into(),
            route_attribute: RouteAttribute::from(python_file),
            hook_function_ident: python_file.into(),
            form_ident: FormIdent::from(python_file).into(),
        }
    }
}

impl<'a> Into<TokenStream2> for RequestHandler<'a> {
    fn into(self) -> TokenStream2 {
        let route_attribute: TokenStream2 = self.route_attribute.into();
        let route_ident: Ident = self.ident.into();
        let hook_function_ident: Ident = self.hook_function_ident.into();
        let form_ident: Ident = self.form_ident.into();

        quote! {
            #route_attribute
            fn #route_ident(query: #form_ident) -> rocket::response::content::Json<String> {
                return rocket::response::content::Json(format!(
                    "{}",
                    #hook_function_ident(pyo3::Python::acquire_gil().python(), query)
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_route_attribute() {
        let py_file = PythonFile {
            file_name: "test.py".into(),
            file_stem: "test".into(),
            uuid: "27597466".into(),
            main_func_args: vec![],
            path: PathBuf::from("/test.py"),
        };

        let token_stream: TokenStream2 = RouteAttribute::from(&py_file).into();

        let target_ts = quote! {
            #[get("/test?<query..>")]
        };

        assert_eq!(token_stream.to_string(), target_ts.to_string());
    }

    #[test]
    fn test_route_handler() {
        let py_file = PythonFile {
            file_name: "test.py".into(),
            file_stem: "test".into(),
            uuid: "27597466".into(),
            main_func_args: vec![],
            path: PathBuf::from("/test.py"),
        };

        let token_stream: TokenStream2 = RequestHandler::from(&py_file).into();

        let target_ts = quote! {
            #[get("/test?<query..>")]
            fn route_27597466(query: Form_27597466) -> rocket::response::content::Json<String> {
                return rocket::response::content::Json(format!(
                    "{}",
                    hook_27597466(pyo3::Python::acquire_gil().python(), query)
                ));
            }
        };

        assert_eq!(token_stream.to_string(), target_ts.to_string());
    }
}
