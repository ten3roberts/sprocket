use std::env;
use std::ffi::CString;

/// Null terminates all str:s in collection
pub fn vec_to_null_terminated(collection: &[&str]) -> Vec<CString> {
    collection
        .iter()
        .map(|e| CString::new(*e).expect("Failed to element layer to CString"))
        .collect()
}

/// Returns a vector containing pointers to all elements
/// Input vector needs to be valid as long as output is used
pub fn vec_to_carray(collection: &[CString]) -> Vec<*const i8> {
    collection.iter().map(|e| e.as_ptr()).collect()
}

/// Sets the working dir to the directory of the executable, independent of how it was launched
/// # Panics
/// if current executable cannot be retrieved
/// if current executable cannot be converted to valid UTF-8
/// if working directory cannot be set
pub fn normalize_working_dir() {
    let path = env::current_exe()
        .expect("Failed to get current executable")
        .to_str()
        .expect("Failed to convert current executable to UTF-8")
        .to_owned();

    let last_delimiter = path
        .rfind("/")
        .unwrap_or(path.rfind("\\").unwrap_or(path.len() - 1));
    let path = &path[0..last_delimiter];
    env::set_current_dir(&path).expect("Failed to set working directory");
}
