use crate::parser::parser_utils::{self, get_inputs, NixNode};
use crate::parser::parser_utils::{get_attr, kill_node_attribute, remove_input_from_output_fn};
use crate::user::{SmlStr, UserAction, UserMetadata, UserPrompt};
use rnix::types::*;
use std::fs;
use std::io::Write;

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
    let ast = match rnix::parse(&content).as_result() {
        Ok(parsed) => parsed,
        Err(err) => {
            return Err(format!(
                "could not parse {} as a nix file: {}",
                full_path, err
            ));
        }
    };
    Ok(ast.root().inner().unwrap())
}
