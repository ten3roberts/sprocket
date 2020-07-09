#[macro_use]
macro_rules! errfmt {
    ($($arg:tt)*) => ({
        Err(format!($($arg)*).into())
    })
}

#[macro_use]
macro_rules! unwrap_or_return {
    ($m:expr, $e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return errfmt!("{} '{}'", $m, e),
        }
    };
}

#[macro_use]
/// Takes a result
/// If Ok(v), returns Ok(v),
/// If Err(e) formats the error with message and returns
macro_rules! unwrap_and_return {
    ($m:expr, $e:expr) => {
        match $e {
            Ok(v) => Ok(v),
            Err(e) => return errfmt!("{} '{}'", $m, e),
        }
    };
}

#[macro_use]
/// Prints an error and returns if Err or unwraps Ok variant
/// Like expect() but doesn't panic
macro_rules! iferr {
    ($m:expr, $e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                error!("{} '{}'", $m, e);
                return;
            }
        }
    };
}
