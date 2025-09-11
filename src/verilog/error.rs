use thiserror::Error;
use crate::verilog::VerilogBase;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("Wire {0} [{1}] has no driver")]
    UnDriver(String, usize),
    #[error("Wire {0} [{1}] has no load")]
    Unload(String, usize),
}

#[derive(Debug, Error)]
pub enum PortError {
    #[error("Port Error => wire {0}")]
    Error1(
        #[from] WireError
    ),
}




#[cfg(test)]
mod test {
    use anyhow::{bail, Context};
    use crate::verilog::error::{PortError, WireError};
    use crate::verilog::error::WireError::UnDriver;

    #[test]
    fn test_error_delver() {
        fn raise_wire_error() -> anyhow::Result<()> {
            bail!(WireError::UnDriver("wire1".into(), 12))
        }
        fn test_main() -> anyhow::Result<()> {
            raise_wire_error().with_context(|e| {})?;
            Ok(())
        }
        match test_main() {
            Ok(_) => {println!("Ok")}
            Err(e) => {println!("err is {:?}", e)}
        }

        println!("Over")
    }
}