use std::env;

fn main() -> tydi_common::error::Result<()> {
    let args: Vec<String> = env::args().collect();
    let input_file = std::fs::read_to_string(&args[1]).unwrap();
    let db = til_parser::query::into_query_storage_default_with_output(input_file, &args[2])?;
    til_vhdl::canonical(&db)?;
    Ok(())
}
