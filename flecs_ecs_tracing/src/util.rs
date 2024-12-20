use std::borrow::Cow;

/// Convert a C string to a Rust string
pub(crate) unsafe fn flecs_str<'a>(str: *const i8) -> Option<Cow<'a, str>> {
    match str.is_null() {
        true => None,
        false => Some(std::ffi::CStr::from_ptr(str).to_string_lossy()),
    }
}

pub(crate) fn leak_cowstr(f: Cow<'_, str>) -> &'static str {
    match f {
        // As far as I can tell, this is zero-copy if the string is already owned
        Cow::Owned(s) => String::leak(s),
        Cow::Borrowed(s) => String::leak(s.to_string()),
    }
}
