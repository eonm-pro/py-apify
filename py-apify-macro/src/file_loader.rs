use crate::python_file::PythonFile;
use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::quote;

pub struct PythonFileLoader {
    module_path: Literal,
    module_name: Literal,
    file_name: Literal,
}

impl<'a> From<&'a PythonFile> for PythonFileLoader {
    fn from(python_file: &'a PythonFile) -> PythonFileLoader {
        PythonFileLoader {
            module_path: Literal::string(&format!(
                "{}",
                std::fs::canonicalize(&python_file.path)
                    .unwrap()
                    .to_str()
                    .unwrap()
            )),
            module_name: Literal::string(&python_file.uuid),
            file_name: Literal::string(&python_file.file_name),
        }
    }
}

impl Into<TokenStream2> for PythonFileLoader {
    fn into(self) -> TokenStream2 {
        let file_name: Literal = self.file_name;
        let module_name: Literal = self.module_name;
        let module_path: Literal = self.module_path;

        quote! {
            pyo3::types::PyModule::from_code(
                py,
                include_str!(#module_path),
                #file_name,
                #module_name,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_py_file_loader() {
        let py_file = PythonFile {
            file_name: "test.py".into(),
            file_stem: "test".into(),
            uuid: "27597466".into(),
            main_func_args: vec![],
            path: PathBuf::from("test_py/test.py"),
        };

        let token_stream: TokenStream2 = PythonFileLoader::from(&py_file).into();

        let full_file_path = Literal::string(&format!(
            "{}",
            std::fs::canonicalize(&py_file.path)
                .unwrap()
                .to_str()
                .unwrap()
        ));

        let target_ts = quote! {
            pyo3::types::PyModule::from_code(
                py,
                include_str!(#full_file_path),
                "test.py",
                "27597466",
            );
        };

        assert_eq!(token_stream.to_string(), target_ts.to_string());
    }
}
