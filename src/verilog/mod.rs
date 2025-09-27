pub mod module;
pub mod parse;
pub mod port;
pub mod wire;
mod writer;
pub mod data;
pub mod parameter;
pub mod width;
mod assign;
mod expression;

trait VerilogBase {
    fn get_name(&self) -> String;
}