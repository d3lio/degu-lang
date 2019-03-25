use libc::c_char;

pub const EMPTY_C_STR: *const c_char = b"\0".as_ptr() as *const _;
