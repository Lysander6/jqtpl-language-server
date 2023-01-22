use std::fs;

use chumsky::Parser;
use jqtpl_language_server::parser::parser;

fn main() -> Result<(), std::io::Error> {
    let file_path = std::env::args().nth(1).expect("Missing argument");
    let src = fs::read_to_string(file_path)?;

    let parse_result = parser().parse_recovery_verbose(src);

    println!("{:?}", parse_result);

    Ok(())
}
