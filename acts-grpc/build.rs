// use std::{env, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["acts.proto"], &["proto"])?;
    // tonic_build::compile_protos("proto/acts.proto")?;
    Ok(())
}
