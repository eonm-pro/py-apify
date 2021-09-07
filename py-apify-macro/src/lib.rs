#![feature(type_ascription)]

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parser, punctuated::Punctuated, LitStr, Token};

mod error;
mod file_loader;
mod form;
mod hook;
mod mount;
mod py_arg;
mod python_file;
mod request_handler;

use file_loader::PythonFileLoader;
use form::Form;
use hook::Hook;
use mount::RocketMount;
use request_handler::RequestHandler;

#[proc_macro]
pub fn apify(item: TokenStream) -> TokenStream {
    let error : TokenStream2 = error::gen_error().into();

    let args: Vec<String> = Punctuated::<LitStr, Token![,]>::parse_terminated
        .parse(item)
        .expect("invalid arguments")
        .into_iter()
        .map(|e| e.value())
        .collect();

    let python_files = python_file::get_py_files(args);

    let loaders: Vec<TokenStream2> = python_files
        .iter()
        .map(|file| PythonFileLoader::from(file).into())
        .collect();

    let hooks: Vec<TokenStream2> = python_files
        .iter()
        .map(|file| Hook::from(file).into())
        .collect();

    let routes: Vec<TokenStream2> = python_files
        .iter()
        .map(|file| RequestHandler::from(file).into())
        .collect();

    let mount: TokenStream2 = RocketMount::from(&python_files).into();

    let forms: Vec<TokenStream2> = python_files
        .iter()
        .map(|file| Form::from(file).into())
        .collect();

    return quote! {
        #error
        use rocket::form::{Form, Strict};
        use pyo3::prelude::*;

        pyo3::prepare_freethreaded_python();
        pyo3::Python::with_gil(|py| {
            #(#forms)*
            #(#loaders)*
            #(#routes)*
            #(#hooks)*
            #mount
        })
    }
    .into();
}
