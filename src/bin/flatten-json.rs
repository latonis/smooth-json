//! This bin target is only used for this crate's tests.
//! It is not intended for users and is not published with the library code to crates.io.

use std::fs;

use serde_json::Value;
use smooth_json;
fn main() {
    let flattener = smooth_json::Flattener::new();
    // get all files in the directory
    // for each file, read the file
    // parse the file as json
    // flatten the json

    let paths = fs::read_dir("tests/input").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let json_str = fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&json_str).unwrap();
        let _flat_json = flattener.flatten(&json);
    }
}
