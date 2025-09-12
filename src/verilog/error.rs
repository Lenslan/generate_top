use thiserror::Error;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("Wire {0} [{1}] has no driver")]
    UnDriver(String, usize),
    #[error("Wire {0} [{1}] has no load")]
    Unload(String, usize),
}

#[derive(Debug)]
pub enum PortError {
    Error1(
        String,
        Vec<WireError>
    )
}


#[cfg(test)]
mod test {
    use crate::verilog::error::{PortError, WireError};
    use crate::verilog::error::WireError::UnDriver;

    #[test]
    fn test_error_delver() {
        fn raise_wire_error() -> Result<(), WireError> {
            Err(UnDriver("test_wire".to_string(), 0))
        }
        fn test_main() -> Result<(), PortError> {
            let mut err_collector = Vec::new();
            raise_wire_error().map_err(|e|err_collector.push(e)).unwrap();
            Ok(())
        }
        match test_main() {
            Ok(_) => {println!("Ok")}
            Err(e) => {println!("err is {:?}", e)}
        }

        println!("Over")
    }
}