use crate::hook::HookFunctionIdent;
use crate::python_file::PythonFile;
use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
use quote::quote;

use crate::form::FormIdent;

#[derive(Clone)]
pub struct RouteAttribute {
    route_name: String,
}

impl From<&PythonFile> for RouteAttribute {
    fn from(python_file: &PythonFile) -> RouteAttribute {
        RouteAttribute {
            route_name: python_file.file_stem.clone(),
        }
    }
}

impl From<RouteAttribute> for TokenStream2 {
    fn from(route_attribute: RouteAttribute) -> Self {
        let route_attribute = Literal::string(&format!("/{}?<query..>", route_attribute.route_name));

        quote! {
            #[get(#route_attribute)]
        }
    }
}

impl From<RouteAttribute> for Literal {
    fn from(route_attribute: RouteAttribute) -> Self {
        Literal::string(&format!("/{}", route_attribute.route_name))
    }
}

#[derive(Clone)]
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

impl From<RequestHandlerIdent> for Ident {
    fn from(request_handler: RequestHandlerIdent) -> Self {
        request_handler.ident
    }
}

pub struct RequestHandler {
    ident: RequestHandlerIdent,
    route_attribute: RouteAttribute,
    hook_function_ident: HookFunctionIdent,
    form_ident: FormIdent,
}

impl From<&PythonFile> for RequestHandler {
    fn from(python_file: &PythonFile) -> RequestHandler {
        RequestHandler {
            ident: python_file.into(),
            route_attribute: RouteAttribute::from(python_file),
            hook_function_ident: python_file.into(),
            form_ident: FormIdent::from(python_file),
        }
    }
}

impl From<RequestHandler> for TokenStream2 {
    fn from(request_handler: RequestHandler) -> Self {
        let route_attribute: TokenStream2 = request_handler.route_attribute.into();
        let route_ident: Ident = request_handler.ident.into();
        let hook_function_ident: Ident = request_handler.hook_function_ident.into();
        let form_ident: Ident = request_handler.form_ident.into();

        quote! {
            #route_attribute
            fn #route_ident(query: rocket::form::Strict<#form_ident>) -> Result<rocket::response::content::Json<String>, PyApifyError> {
                Ok(rocket::response::content::Json(format!(
                    "{}",
                    #hook_function_ident(pyo3::Python::acquire_gil().python(), query.into_inner())?
                )))
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
