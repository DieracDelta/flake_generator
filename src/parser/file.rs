use crate::parser::parser_utils::NixNode;
use crate::user::{SmlStr, UserMetadata};
use rnix::types::*;
use std::fs;
use std::io::Write;

pub fn string_to_node(content: String) -> Result<NixNode, String> {
    let ast = match rnix::parse(&content).as_result() {
        Ok(parsed) => parsed,
        Err(err) => {
            return Err(format!("could not parse as a nix file: {}", err));
        }
    };
    Ok(ast.root().inner().unwrap())
}

// TODO shouldn't we be concatenating the filename to the absolute path?
pub fn filename_to_node(filename: &str, full_path: &SmlStr) -> Result<NixNode, String> {
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            const IS_DIRECTORY_ERRNO: i32 = 21;
            let err_msg = if let Some(IS_DIRECTORY_ERRNO) = err.raw_os_error() {
                format!("selected path {} is a directory", full_path)
            } else if err.kind() == std::io::ErrorKind::InvalidData {
                format!(
                    "selected path {} does not contain valid UTF-8 data",
                    full_path
                )
            } else {
                format!("something is very wrong: {}", err)
            };
            return Err(err_msg);
        }
    };
    string_to_node(content)
}

pub(crate) fn write_to_node(user_data: &UserMetadata) {
    let stringified = user_data.root.as_ref().unwrap().to_string();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(user_data.clone().filename.unwrap())
        .unwrap();
    file.write_all(stringified.as_bytes()).unwrap();
}
