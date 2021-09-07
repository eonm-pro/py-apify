use crate::python_file::PythonFile;
use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
use quote::quote;

use crate::form::FormIdent;

pub struct HookFunctionIdent {
    ident: Ident,
}

impl From<&PythonFile> for HookFunctionIdent {
    fn from(python_file: &PythonFile) -> HookFunctionIdent {
        HookFunctionIdent {
            ident: Ident::new(&format!("hook_{}", python_file.uuid), Span::call_site()),
        }
    }
}

impl From<HookFunctionIdent> for Ident {
    fn from(hook_function_ident: HookFunctionIdent) -> Self {
        hook_function_ident.ident
    }
}

pub struct Hook {
    ident: HookFunctionIdent,
    py_module_name: Literal,
    py_file_name: Literal,
    form_ident: FormIdent,
}

impl From<&PythonFile> for Hook {
    fn from(python_file: &PythonFile) -> Hook {
        Hook {
            ident: HookFunctionIdent::from(python_file),
            py_module_name: Literal::string(&python_file.uuid),
            py_file_name: Literal::string(&python_file.file_name),
            form_ident: FormIdent::from(python_file),
        }
    }
}

impl From<Hook> for TokenStream2 {
    fn from(hook: Hook) -> Self {
        let hook_function_ident: Ident = hook.ident.into();
        let module_name = hook.py_module_name;
        let file_name = hook.py_file_name;
        let form_ident: Ident = hook.form_ident.into();

        quote! {
            fn #hook_function_ident(py_lock: pyo3::Python, input: #form_ident) -> Result<String, PyApifyError> {
                let kwargs : &pyo3::types::PyDict = input.kwargs(py_lock);

                let nlp = pyo3::types::PyModule::import(
                    py_lock,
                    #module_name,
                )
                .expect("failed to import PyModule");

                match nlp
                    .getattr("call").map_err(|_e| PyApifyError::HookFunctionNotFound(#file_name.to_string()))?
                    .call((), Some(kwargs)) {
                        Ok(result) => Ok(result.extract().unwrap_or("{}".to_string())),
                        Err(e) => Err(PyApifyError::HookFunctionFailure(e.to_string()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_hook() {
        let py_file = PythonFile {
            file_name: "test.py".into(),
            file_stem: "test".into(),
            uuid: "27597466".into(),
            main_func_args: vec![],
            path: PathBuf::from("/test.py"),
        };

        let token_stream: TokenStream2 = Hook::from(&py_file).into();

        let target_ts = quote! {
            fn hook_27597466(py_lock: pyo3::Python, input: Form_27597466) -> String {
                let kwargs : &pyo3::types::PyDict = input.kwargs(py_lock);

                let nlp = pyo3::types::PyModule::import(
                    py_lock,
                    "27597466",
                )
                .expect("failed to import PyModule");

                match nlp
                    .getattr("call")
                    .expect(&format!("`call` function was not found in {}. Your python file must include a `call` function that returns json data:\n\ndef call(input):\n\tjson.dumps('{{'foo': 'bar'}}')\n\n", "test.py"))
                    .call((), Some(kwargs)) {
                        Ok(result) => result.extract().unwrap_or("{}".to_string()),
                        Err(e) => format!("{{\"error\": \"{}\"}}", e.to_string())
                }
            }
        };

        assert_eq!(token_stream.to_string(), target_ts.to_string());
    }
}
