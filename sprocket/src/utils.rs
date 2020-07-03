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