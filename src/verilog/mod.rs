pub mod module;
pub mod parse;
pub mod port;
pub mod wire;
mod writer;
mod error;



trait VerilogBase {
    fn get_name(&self) -> String;
}