use crate::python_file::PythonFile;
use crate::request_handler::RequestHandlerIdent;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

pub struct RocketMount {
    routes: Vec<RequestHandlerIdent>,
}

impl From<&Vec<PythonFile>> for RocketMount {
    fn from(python_files: &Vec<PythonFile>) -> RocketMount {
        RocketMount {
            routes: python_files.iter().map(|file| file.into()).collect(),
        }
    }
}

impl Into<TokenStream2> for RocketMount {
    fn into(self) -> TokenStream2 {
        let idents = self
            .routes
            .into_iter()
            .map(|e| e.into())
            .collect::<Vec<Ident>>();

        quote! {
            rocket::build().mount("/", routes![#(#idents),*])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mount() {
        let py_file_1 = PythonFile {
            file_name: "test.py".into(),
            file_stem: "test".into(),
            uuid: "27597466".into(),
            main_func_args: vec![],
            path: PathBuf::from("/test.py"),
        };

        let py_file_2 = PythonFile {
            file_name: "test-1.py".into(),
            file_stem: "test-1".into(),
            uuid: "41198456".into(),
            main_func_args: vec![],
            path: PathBuf::from("/test-1.py"),
        };

        let token_stream: TokenStream2 = RocketMount::from(&vec![py_file_1, py_file_2]).into();

        let target_ts = quote! {
            rocket::build().mount("/", routes![route_27597466, route_41198456])
        };

        assert_eq!(token_stream.to_string(), target_ts.to_string());
    }
}
