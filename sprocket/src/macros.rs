#[macro_use]
macro_rules! errfmt {
    ($($arg:tt)*) => ({
        Err(format!($($arg)*))
    })
}
