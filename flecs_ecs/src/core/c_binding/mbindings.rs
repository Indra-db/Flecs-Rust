//! manual bindings to prevent warnings / errors from bindgen in build or tests

#![allow(clippy::all)]
#![allow(warnings)]
#[doc(hidden)]
pub mod mbindings {}

use super::*;

/// stdio state variables.
///
#[cfg_attr(doctest, doc = " ````no_test")]
/// ```
///     if (_flags&(__SLBF|__SWR)) == (__SLBF|__SWR),
///         _lbfsize is -_bf._size, else _lbfsize is 0
///     if _flags&__SRD, _w is 0
///     if _flags&__SWR, _r is 0
/// ```
///
/// This ensures that the getc and putc macros (or inline functions) never
/// try to write or read from a file that is in `read' or `write' mode.
/// (Moreover, they can, and do, automatically switch from read mode to
/// write mode, and back, on "r+" and "w+" files.)
///
/// _lbfsize is used only to make the inline line-buffered output stream
/// code as compact as possible.
///
/// _ub, _up, and _ur are used when ungetc() pushes back more characters
/// than fit in the current _bf, or when ungetc() pushes back a character
/// that does not match the previous one in _bf.  When this happens,
/// _ub._base becomes non-nil (i.e., a stream has ungetc() data iff
/// _ub._base!=NULL) and _up and _ur save the current values of _p and _r.
///
/// NB: see WARNING above before changing the layout of this structure!
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __sFILE {
    #[doc = "current position in (some) buffer"]
    pub _p: *mut ::std::os::raw::c_uchar,
    #[doc = "read space left for getc()"]
    pub _r: ::std::os::raw::c_int,
    #[doc = "write space left for putc()"]
    pub _w: ::std::os::raw::c_int,
    #[doc = "flags, below; this FILE is free if 0"]
    pub _flags: ::std::os::raw::c_short,
    #[doc = "fileno, if Unix descriptor, else -1"]
    pub _file: ::std::os::raw::c_short,
    #[doc = "the buffer (at least 1 byte, if !NULL)"]
    pub _bf: __sbuf,
    #[doc = "0 or -_bf._size, for inline putc"]
    pub _lbfsize: ::std::os::raw::c_int,
    #[doc = "cookie passed to io functions"]
    pub _cookie: *mut ::std::os::raw::c_void,
    pub _close: ::std::option::Option<
        unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void) -> ::std::os::raw::c_int,
    >,
    pub _read: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::std::os::raw::c_void,
            arg2: *mut ::std::os::raw::c_char,
            arg3: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub _seek: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::std::os::raw::c_void,
            arg2: fpos_t,
            arg3: ::std::os::raw::c_int,
        ) -> fpos_t,
    >,
    pub _write: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::std::os::raw::c_void,
            arg2: *const ::std::os::raw::c_char,
            arg3: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    #[doc = "ungetc buffer"]
    pub _ub: __sbuf,
    #[doc = "additions to FILE to not break ABI"]
    pub _extra: *mut __sFILEX,
    #[doc = "saved _r when _r is counting ungetc data"]
    pub _ur: ::std::os::raw::c_int,
    #[doc = "guarantee an ungetc() buffer"]
    pub _ubuf: [::std::os::raw::c_uchar; 3usize],
    #[doc = "guarantee a getc() buffer"]
    pub _nbuf: [::std::os::raw::c_uchar; 1usize],
    #[doc = "buffer for fgetln()"]
    pub _lb: __sbuf,
    #[doc = "stat.st_blksize (may be != _bf._size)"]
    pub _blksize: ::std::os::raw::c_int,
    #[doc = "current lseek offset (see WARNING)"]
    pub _offset: fpos_t,
}

/// stdio state variables.
///
/// The following always hold:
///
#[cfg_attr(doctest, doc = " ````no_test")]
/// ```
/// if (_flags&(__SLBF|__SWR)) == (__SLBF|__SWR),
///     _lbfsize is -_bf._size, else _lbfsize is 0
/// if _flags&__SRD, _w is 0
/// if _flags&__SWR, _r is 0
/// ```
///
/// This ensures that the getc and putc macros (or inline functions) never
/// try to write or read from a file that is in `read` or `write` mode.
/// (Moreover, they can, and do, automatically switch from read mode to
/// write mode, and back, on "r+" and "w+" files.)
///
/// _lbfsize is used only to make the inline line-buffered output stream
/// code as compact as possible.
///
/// _ub, _up, and _ur are used when ungetc() pushes back more characters
/// than fit in the current _bf, or when ungetc() pushes back a character
/// that does not match the previous one in _bf.  When this happens,
/// _ub._base becomes non-nil (i.e., a stream has ungetc() data iff
/// _ub._base!=NULL) and _up and _ur save the current values of _p and _r.
///
/// NB: see WARNING above before changing the layout of this structure!
pub type FILE = __sFILE;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ecs_alert_desc_t {
    pub _canary: i32,
    #[doc = "Entity associated with alert"]
    pub entity: ecs_entity_t,
    #[doc = "Alert query. An alert will be created for each entity that matches the\n specified query. The query must have at least one term that uses the\n $this variable (default)."]
    pub filter: ecs_filter_desc_t,
    /// Template for alert message. This string is used to generate the alert
    /// message and may refer to variables in the query result. The format for
    /// the template expressions is as specified by ecs_interpolate_string().
    ///
    /// # Examples
    ///
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// "$this has Position but not Velocity"
    /// "$this has a parent entity $parent without Position"
    /// ```
    pub message: *const ::std::os::raw::c_char,
    #[doc = "User friendly name. Will only be set if FLECS_DOC addon is enabled."]
    pub doc_name: *const ::std::os::raw::c_char,
    #[doc = "Description of alert. Will only be set if FLECS_DOC addon is enabled"]
    pub brief: *const ::std::os::raw::c_char,
    #[doc = "Metric kind. Must be EcsAlertInfo, EcsAlertWarning, EcsAlertError or\n EcsAlertCritical. Defaults to EcsAlertError."]
    pub severity: ecs_entity_t,
    #[doc = "Severity filters can be used to assign different severities to the same\n alert. This prevents having to create multiple alerts, and allows\n entities to transition between severities without resetting the\n alert duration (optional)."]
    pub severity_filters: [ecs_alert_severity_filter_t; 4usize],
    #[doc = "The retain period specifies how long an alert must be inactive before it\n is cleared. This makes it easier to track noisy alerts. While an alert is\n inactive its duration won't increase.\n When the retain period is 0, the alert will clear immediately after it no\n longer matches the alert query."]
    pub retain_period: f32,
    #[doc = "Alert when member value is out of range. Uses the warning/error ranges\n assigned to the member in the MemberRanges component (optional)."]
    pub member: ecs_entity_t,
    #[doc = "(Component) id of member to monitor. If left to 0 this will be set to\n the parent entity of the member (optional)."]
    pub id: ecs_id_t,
    #[doc = "Variable from which to fetch the member (optional). When left to NULL\n 'id' will be obtained from $this."]
    pub var: *const ::std::os::raw::c_char,
}

extern "C" {
    /// Enable/disable logging time since last log.
    ///
    /// By default, deltatime is disabled. Note that enabling timestamps introduces
    /// overhead as the logging code will need to obtain the current time.
    ///
    /// When enabled, this logs the amount of time in seconds passed since the last
    /// log, when this amount is non-zero. The format is a '+' character followed by
    /// the number of seconds:
    ///
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// +1 trace: log message
    /// ```
    ///
    /// # Parameters
    ///
    /// * `enabled` - Whether to enable tracing with timestamps.
    ///
    /// # Returns
    ///
    /// Previous timestamp setting.
    pub fn ecs_log_enable_timedelta(enabled: bool) -> bool;
}

extern "C" {
    /// Emulate a request.
    ///
    /// The request string must be a valid HTTP request. A minimal example:
    ///
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// GET /entity/flecs/core/World?label=true HTTP/1.1
    /// ```
    ///
    /// # Parameters
    ///
    /// * `srv` - The server.
    /// * `req` - The request.
    /// * `len` - The length of the request (optional).
    ///
    /// # Returns
    ///
    /// The reply.
    pub fn ecs_http_server_http_request(
        srv: *mut ecs_http_server_t,
        req: *const ::std::os::raw::c_char,
        len: ecs_size_t,
        reply_out: *mut ecs_http_reply_t,
    ) -> ::std::os::raw::c_int;
}
