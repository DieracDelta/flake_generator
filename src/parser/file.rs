use crate::parser::parser_utils::{string_to_node, NixNode};
use crate::user::{SmlStr, UserMetadata};
use anyhow::bail;
use std::fs;
use std::io::Write;

// TODO shouldn't we be concatenating the filename to the absolute path?
pub fn filename_to_node(filename: &str, full_path: &SmlStr) -> anyhow::Result<NixNode> {
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
            bail!(err_msg);
        }
    };
    string_to_node(content)
}

pub(crate) fn write_to_node(user_data: &UserMetadata) {
    let stringified = user_data.root.as_ref().unwrap().to_string();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(user_data.filename.as_ref().unwrap())
        .unwrap();
    file.write_all(stringified.as_bytes()).unwrap();
}
