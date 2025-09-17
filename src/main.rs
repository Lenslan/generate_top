use std::path::PathBuf;
use clap::{Parser, Subcommand};
use crate::excel::reader::ExcelReader;
use crate::excel::writer::ExcelWriter;

mod excel;
mod utils;
mod verilog;


fn main() {
    simple_logger::init_with_env().unwrap();
    let args = Args::parse();
    let module_path = PathBuf::from(args.top.clone());
    if !module_path.is_dir() {
        log::error!("the path {} is not a directory", args.top);
        std::process::exit(1);
    }

    match args.command {
        Commands::gen_excel => {
            gen_excel(module_path);
        }
        Commands::from_file => {
            from_file(module_path);
        }
        Commands::from_excel => {
            from_excel(module_path);
        }
    }
}

#[derive(Parser, Debug)]
#[command(version = "1.0")]
#[command(about = "used to generate verilog top module")]
struct Args {
    #[command(subcommand)]
    command: Commands,
    
    /// indicate the top module directory
    #[arg(short, long)]
    top: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// generate or update excel file from verilog source file
    gen_excel,

    /// generate verilog-top by `gen_excel` & `from_excel`
    from_file,

    /// generate verilog-top from excel
    from_excel
}

fn gen_excel(path: PathBuf) {
    let mut writer = ExcelWriter::new(path);
    writer.traverse_v();
    writer.generate_or_update();
}

fn from_excel(path: PathBuf) {
    let parent_path = path.parent().expect("Could not get parent path");
    let module_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .expect("Could not get module name");
    let excel_name = parent_path.join(format!("{}.xlsx", module_name));
    ExcelReader::new(excel_name).generate_v();
}

fn from_file(path: PathBuf) {
    gen_excel(path.clone());
    from_excel(path);
}