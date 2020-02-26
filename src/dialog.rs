extern crate nfd;
use nfd::Response;
#[allow(unused_imports)]
use std::path::Path;

/// Open file dialog
#[allow(dead_code)]
pub fn open_file_dialog() -> String {
    let result = nfd::open_file_dialog(Some("gb*,asm"), None).unwrap();
    match result {
        Response::Okay(file_path) => return file_path,
        Response::OkayMultiple(_) => "".to_string(),
        Response::Cancel => return "".to_string(),
    }
}

#[test]
fn test_open_file_dialog() {
    let file_path = open_file_dialog();
    let mut result = false;
    let path = Path::new(&file_path);
    let extension = path.extension();
    match extension {
        Some(ext) => {
            if ext == "gb" || ext == "gbc" {
                result = true;
            } else if ext == "asm" {
                result = true;
            }
        }
        None => {}
    }
    assert!(result);
}
