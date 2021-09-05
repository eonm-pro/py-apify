use glob::glob;
use std::fs::read_to_string;
use std::path::PathBuf;
use uuid::Uuid;

use crate::py_arg::{get_func_args, PyArg};

#[derive(Debug, Clone)]
pub struct PythonFile {
    pub path: PathBuf,
    pub file_name: String,
    pub file_stem: String,
    pub uuid: String,
    pub main_func_args: Vec<PyArg>,
}

impl PythonFile {
    pub fn new(input: PathBuf) -> Self {
        PythonFile {
            file_name: input.file_name().expect("").to_str().expect("").to_string(),
            file_stem: input.file_stem().expect("").to_str().expect("").to_string(),
            main_func_args: get_func_args(
                read_to_string(&input).expect("failed to read file"),
                "call",
            ),
            path: input,
            uuid: Uuid::new_v4().to_simple().to_string(),
        }
    }
}

pub fn get_py_files(input: Vec<String>) -> Vec<PythonFile> {
    let mut files_name: Vec<PathBuf> = if !input.is_empty() {
        input
            .iter()
            .flat_map(|elem| {
                glob(elem)
                    .expect("Failed to read glob pattern")
                    .filter_map(|e| e.ok())
            })
            .collect::<Vec<PathBuf>>()
    } else {
        glob("./src/*.py")
            .expect("Failed to read glob pattern")
            .filter_map(|e| e.ok())
            .collect::<Vec<PathBuf>>()
    };

    files_name.sort();
    files_name.dedup();

    let python_files = files_name
        .into_iter()
        .map(|elem| PythonFile::new(elem))
        .collect::<Vec<PythonFile>>();

    #[cfg(not(feature = "no-check"))]
    {
        // python_files.iter().for_each(|file| file.check());
    }

    return python_files;
}
