use std::path::PathBuf;
use walkdir::WalkDir;

pub struct ExcelReader {
    path: PathBuf,
}

impl ExcelReader {
    pub fn new(path: PathBuf) -> Self {
        ExcelReader { path }
    }

    pub fn generate_v(&self) {}
}
