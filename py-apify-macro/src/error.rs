use crate::TokenStream2;
use quote::quote;

pub fn gen_error() -> TokenStream2 {
    quote! {
        #[catch(404)]
        fn invalid_argument(req: &Request) -> Result<(), PyApifyError> {
            Err(PyApifyError::InvalidArguments)
        }

        use std::error;
        use std::error::Error;
        use std::fmt;
        use std::path::PathBuf;

        #[derive(Debug)]
        pub enum PyApifyError {
            HookFunctionNotFound(String),
            HookFunctionFailure(String),
            InvalidArguments,
        }

        impl fmt::Display for PyApifyError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    PyApifyError::HookFunctionNotFound(file_name) =>
                        write!(f, "Failed to call the `call` function inside your Python file {}. Your python file must contain a `call` function", file_name),
                    PyApifyError::HookFunctionFailure(error_message) => 
                        write!(f, "The hook function returned an error : {}", error_message),
                    PyApifyError::InvalidArguments =>
                        write!(f, "Invalid arguments")
                }
            }
        }

        impl Error for PyApifyError {}

        use std::io::Cursor;

        use rocket::http::Status;
        use rocket::request::Request;
        use rocket::response::{self, Response, Responder};
        use rocket::http::ContentType;

        impl<'a> Responder<'a, 'a> for PyApifyError {
            fn respond_to(self, _: &Request) -> response::Result<'a> {
                let error_message = format!("{{\"error\": \"{}\"}}", self);
                let error_messag_len = error_message.len();

                let status = match self {
                    Self::InvalidArguments => Status::BadRequest,
                    _ => Status::InternalServerError,
                };
                
                Response::build()
                    .header(ContentType::JSON)
                    .status(status)
                    .sized_body(error_messag_len, Cursor::new(error_message))
                    .ok()
            }
        }
    }
}