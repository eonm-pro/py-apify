use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use rustpython_parser::ast::{ExpressionType, Located, Parameter, Program, StatementType, Varargs};
use rustpython_parser::parser;

#[derive(Debug, Clone)]
pub enum PyPrimitiveDataType {
    Str,
    Float,
    Int,
    Bool,
}

impl From<String> for PyPrimitiveDataType {
    fn from(input: String) -> PyPrimitiveDataType {
        match input.as_ref() {
            "str" => PyPrimitiveDataType::Str,
            "int" => PyPrimitiveDataType::Int,
            "float" => PyPrimitiveDataType::Float,
            "bool" => PyPrimitiveDataType::Bool,
            _ => panic!("This datatype is not supported by py apify"),
        }
    }
}

impl Into<Ident> for PyPrimitiveDataType {
    fn into(self) -> Ident {
        match self {
            Self::Str => Ident::new("String", Span::call_site()),
            Self::Float => Ident::new("f64", Span::call_site()),
            Self::Int => Ident::new("usize", Span::call_site()),
            Self::Bool => Ident::new("bool", Span::call_site()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PyArg {
    pub name: String,
    pub data_type: PyPrimitiveDataType,
    pub optional: bool,
}

impl Into<Ident> for PyArg {
    fn into(self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }
}

impl Into<TokenStream2> for PyArg {
    fn into(self) -> TokenStream2 {
        let struct_field_ident: Ident = self.clone().into();
        let data_type_ident: Ident = self.data_type.into();

        let data_type = if self.optional {
            quote! {
                Option<#data_type_ident>
            }
        } else {
            quote! {
                #data_type_ident
            }
        };

        quote! {
            #struct_field_ident: #data_type
        }
    }
}

pub fn get_func_by_name<'a>(
    program: &'a Program,
    func_name: &'a str,
) -> Option<&'a Located<StatementType>> {
    program.statements.iter().find(|statement| {
        if let StatementType::FunctionDef { name, .. } = &statement.node {
            return name == func_name;
        }

        false
    })
}

pub fn collect_func_args_default_values(
    func: &Located<StatementType>,
) -> Vec<&Located<ExpressionType>> {
    let mut defaults = vec![];

    if let StatementType::FunctionDef { args, .. } = &func.node {
        defaults.extend(&args.defaults);
        defaults.extend(
            &args
                .kw_defaults
                .iter()
                .flat_map(|e| e)
                .collect::<Vec<&Located<ExpressionType>>>(),
        );

        defaults.sort_by(|a, b| {
            (a.location.column())
                .partial_cmp(&b.location.column())
                .unwrap()
        });
    }

    defaults
}

pub fn collect_func_args(func: &Located<StatementType>) -> Vec<&Parameter> {
    let mut func_args = vec![];

    if let StatementType::FunctionDef { args, .. } = &func.node {
        func_args.extend(&args.args);
        func_args.extend(&args.kwonlyargs);

        match &args.vararg {
            Varargs::Named(param) => {
                func_args.push(param);
            }
            _ => {}
        };

        func_args.sort_by(|a, b| {
            (a.location.column())
                .partial_cmp(&b.location.column())
                .unwrap()
        });
    }

    func_args
}

pub fn get_func_args(py_code: String, func_name: &str) -> Vec<PyArg> {
    let program = parser::parse_program(&py_code).unwrap();

    let call_func = get_func_by_name(&program, &func_name).expect("call function not found");
    let defaults_values = collect_func_args_default_values(&call_func);
    let func_args = collect_func_args(&call_func);

    let mut args = vec![];

    let mut peekable_args = func_args.iter().peekable();

    while let Some(arg) = peekable_args.next() {
        let mut data_type = None;

        if let Some(location) = &arg.annotation {
            if let ExpressionType::Identifier { name, .. } = &location.node {
                data_type = Some(name.to_string());
            }
        }

        let optional = defaults_values.iter().find(|e| {
            if let Some(next_elem) = peekable_args.peek() {
                e.location.column() > arg.location.column()
                    && e.location.column() < next_elem.location.column()
            } else {
                e.location.column() > arg.location.column()
            }
        });

        let data_type: PyPrimitiveDataType = match data_type {
            Some(d_type) => PyPrimitiveDataType::from(d_type),
            None => PyPrimitiveDataType::Str,
        };

        args.push(PyArg {
            name: arg.arg.to_string(),
            data_type: data_type,
            optional: optional.is_some(),
        });
    }

    args
}
