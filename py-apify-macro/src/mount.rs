use crate::python_file::PythonFile;
use crate::request_handler::{RequestHandlerIdent, RouteAttribute};
use proc_macro2::{Ident, Literal, TokenStream as TokenStream2};
use quote::quote;

#[derive(Clone)]
pub struct RocketMount {
    routes: Vec<RequestHandlerIdent>,
    literal_route: Vec<RouteAttribute>,
}

impl From<&Vec<PythonFile>> for RocketMount {
    fn from(python_files: &Vec<PythonFile>) -> RocketMount {
        RocketMount {
            routes: python_files.iter().map(|file| file.into()).collect(),
            literal_route: python_files.iter().map(|file| file.into()).collect(),
        }
    }
}

impl From<RocketMount> for TokenStream2 {
    fn from(rocket_mount: RocketMount) -> Self {
        let idents = rocket_mount
            .routes
            .clone()
            .into_iter()
            .map(|e| e.into())
            .collect::<Vec<Ident>>();

        let literals = rocket_mount
            .literal_route
            .into_iter()
            .map(|e| e.into())
            .collect::<Vec<Literal>>();

        quote! {
            rocket::build().mount("/", routes![#(#idents),*])
                #(.register(#literals, catchers![invalid_argument]))*
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
