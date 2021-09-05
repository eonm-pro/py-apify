use crate::python_file::PythonFile;
use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
use quote::quote;

use crate::py_arg::PyArg;

pub struct FormIdent {
    ident: Ident,
}

impl From<&PythonFile> for FormIdent {
    fn from(python_file: &PythonFile) -> FormIdent {
        FormIdent {
            ident: Ident::new(&format!("Form_{}", python_file.uuid), Span::call_site()),
        }
    }
}

impl Into<Ident> for FormIdent {
    fn into(self) -> Ident {
        self.ident
    }
}

pub struct Form {
    ident: FormIdent,
    variants: Vec<PyArg>,
}

impl From<&PythonFile> for Form {
    fn from(python_file: &PythonFile) -> Form {
        Form {
            ident: FormIdent::from(python_file),
            variants: python_file.main_func_args.clone(),
        }
    }
}

impl Into<TokenStream2> for Form {
    fn into(self) -> TokenStream2 {
        let form_ident: Ident = self.ident.into();
        let struct_fields: Vec<TokenStream2> = self
            .variants
            .clone()
            .into_iter()
            .map(|variant| variant.into())
            .collect();

        let struct_fields_names: Vec<TokenStream2> = self
            .variants
            .iter()
            .map(|variant| {
                let literal_name = Literal::string(&variant.name);
                quote! {
                    #literal_name
                }
            })
            .collect();

        let struct_fields_values: Vec<TokenStream2> = self
            .variants
            .into_iter()
            .map(|field| {
                let variant_ident: Ident = field.into();
                quote! {
                    self.#variant_ident
                }
            })
            .collect();

        quote! {
            #[derive(rocket::form::FromForm)]
            struct #form_ident {
                #(#struct_fields),*
            }

            impl #form_ident {
                pub fn kwargs(self, py: pyo3::prelude::Python) -> &pyo3::types::PyDict {
                    use pyo3::types::IntoPyDict;
                    use pyo3::conversion::IntoPy;

                    let mut args : Vec<(&str, pyo3::Py<pyo3::PyAny>)> = vec!();

                    #(
                        let struct_field_value = #struct_fields_values;
                        let py_any : pyo3::Py<pyo3::PyAny> = struct_field_value.into_py(py);

                        if !py_any.is_none(py) {
                            args.push((#struct_fields_names, py_any));
                        }
                    )*

                    args.into_py_dict(py)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::py_arg::{PyArg, PyPrimitiveDataType};
    use std::path::PathBuf;

    #[test]
    fn test_form() {
        let py_file = PythonFile {
            file_name: "test.py".into(),
            file_stem: "test".into(),
            uuid: "27597466".into(),
            main_func_args: vec![
                PyArg {
                    name: "input".into(),
                    data_type: PyPrimitiveDataType::Str,
                    optional: false,
                },
                PyArg {
                    name: "score".into(),
                    data_type: PyPrimitiveDataType::Int,
                    optional: true,
                },
            ],
            path: PathBuf::from("test_py/test.py"),
        };

        let token_stream: TokenStream2 = Form::from(&py_file).into();

        let target_ts = quote! {
            #[derive(rocket::form::FromForm)]
            struct Form_27597466 {
                input: String,
                score: Option<usize>
            }

            impl Form_27597466 {
                pub fn kwargs(self, py: pyo3::prelude::Python) -> &pyo3::types::PyDict {
                    use pyo3::types::IntoPyDict;
                    use pyo3::conversion::IntoPy;

                    let mut args : Vec<(&str, pyo3::Py<pyo3::PyAny>)> = vec!();

                    let struct_field_value = self.input;
                    let py_any : pyo3::Py<pyo3::PyAny> = struct_field_value.into_py(py);

                    if !py_any.is_none(py) {
                        args.push(("input", py_any));
                    }

                    let struct_field_value = self.score;
                    let py_any : pyo3::Py<pyo3::PyAny> = struct_field_value.into_py(py);

                    if !py_any.is_none(py) {
                        args.push(("score", py_any));
                    }

                    args.into_py_dict(py)
                }
            }
        };

        assert_eq!(token_stream.to_string(), target_ts.to_string());
    }
}
