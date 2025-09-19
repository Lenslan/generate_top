pub mod module;
pub mod parse;
pub mod port;
pub mod wire;
mod writer;
mod data;
pub mod parameter;
mod width;

trait VerilogBase {
    fn get_name(&self) -> String;
}