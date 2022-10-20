use std::{env, ffi::OsStr, path::PathBuf};
use tydi_common::error::{Error, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = PathBuf::from(&args[1]);
    let db = match file_path.extension().and_then(OsStr::to_str) {
        Some("til") => {
            let input_file = std::fs::read_to_string(&file_path).unwrap();
            til_parser::query::into_query_storage_default_with_output(input_file, &args[2])
        }
        Some("toml") => til_parser::project::from_path(file_path),
        _ => Err(Error::FileIOError(format!(
            "Expected file ending in .toml or .til, got: {}",
            &args[1]
        ))),
    }?;
    til_vhdl::canonical(&db)?;
    Ok(())
}
