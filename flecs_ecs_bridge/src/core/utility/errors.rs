use std::fmt::{Display, Formatter};

pub struct InvalidStrFromId {
    pub id: u64,
}

impl std::fmt::Display for InvalidStrFromId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid string conversion from id: {}", self.id)
    }
}

pub enum FlecsErrorCode {
    InvalidOperation,
    InvalidParameter,
    ConstraintViolated,
    OutOfMemory,
    OutOfRange,
    Unsupported,
    InternalError,
    AlreadyDefined,
    MissingOsApi,
    OperationFailed,
    InvalidConversion,
    IdInUse,
    CycleDetected,
    LeakDetected,
    InconsistentName,
    NameInUse,
    NotAComponent,
    InvalidComponentSize,
    InvalidComponentAlignment,
    ComponentNotRegistered,
    InconsistentComponentId,
    InconsistentComponentAction,
    ModuleUndefined,
    MissingSymbol,
    AlreadyInUse,
    AccessViolation,
    ColumnIndexOutOfRange,
    ColumnIsNotShared,
    ColumnIsShared,
    ColumnTypeMismatch,
    InvalidWhileReadonly,
    LockedStorage,
    InvalidFromWorker,
}

impl Display for FlecsErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FlecsErrorCode::InvalidOperation => "ECS_INVALID_OPERATION",
                FlecsErrorCode::InvalidParameter => "ECS_INVALID_PARAMETER",
                FlecsErrorCode::ConstraintViolated => "ECS_CONSTRAINT_VIOLATED",
                FlecsErrorCode::OutOfMemory => "ECS_OUT_OF_MEMORY",
                FlecsErrorCode::OutOfRange => "ECS_OUT_OF_RANGE",
                FlecsErrorCode::Unsupported => "ECS_UNSUPPORTED",
                FlecsErrorCode::InternalError => "ECS_INTERNAL_ERROR",
                FlecsErrorCode::AlreadyDefined => "ECS_ALREADY_DEFINED",
                FlecsErrorCode::MissingOsApi => "ECS_MISSING_OS_API",
                FlecsErrorCode::OperationFailed => "ECS_OPERATION_FAILED",
                FlecsErrorCode::InvalidConversion => "ECS_INVALID_CONVERSION",
                FlecsErrorCode::IdInUse => "ECS_ID_IN_USE",
                FlecsErrorCode::CycleDetected => "ECS_CYCLE_DETECTED",
                FlecsErrorCode::LeakDetected => "ECS_LEAK_DETECTED",
                FlecsErrorCode::InconsistentName => "ECS_INCONSISTENT_NAME",
                FlecsErrorCode::NameInUse => "ECS_NAME_IN_USE",
                FlecsErrorCode::NotAComponent => "ECS_NOT_A_COMPONENT",
                FlecsErrorCode::InvalidComponentSize => "ECS_INVALID_COMPONENT_SIZE",
                FlecsErrorCode::InvalidComponentAlignment => "ECS_INVALID_COMPONENT_ALIGNMENT",
                FlecsErrorCode::ComponentNotRegistered => "ECS_COMPONENT_NOT_REGISTERED",
                FlecsErrorCode::InconsistentComponentId => "ECS_INCONSISTENT_COMPONENT_ID",
                FlecsErrorCode::InconsistentComponentAction => "ECS_INCONSISTENT_COMPONENT_ACTION",
                FlecsErrorCode::ModuleUndefined => "ECS_MODULE_UNDEFINED",
                FlecsErrorCode::MissingSymbol => "ECS_MISSING_SYMBOL",
                FlecsErrorCode::AlreadyInUse => "ECS_ALREADY_IN_USE",
                FlecsErrorCode::AccessViolation => "ECS_ACCESS_VIOLATION",
                FlecsErrorCode::ColumnIndexOutOfRange => "ECS_COLUMN_INDEX_OUT_OF_RANGE",
                FlecsErrorCode::ColumnIsNotShared => "ECS_COLUMN_IS_NOT_SHARED",
                FlecsErrorCode::ColumnIsShared => "ECS_COLUMN_IS_SHARED",
                FlecsErrorCode::ColumnTypeMismatch => "ECS_COLUMN_TYPE_MISMATCH",
                FlecsErrorCode::InvalidWhileReadonly => "ECS_INVALID_WHILE_READONLY",
                FlecsErrorCode::LockedStorage => "ECS_LOCKED_STORAGE",
                FlecsErrorCode::InvalidFromWorker => "ECS_INVALID_FROM_WORKER",
            }
        )
    }
}

#[cfg(feature = "flecs_ecs_asserts")]
#[macro_export]
macro_rules! ecs_assert {
    ($condition:expr $(,)?, $error_code:expr $(,)?) => {
        assert!($condition, "{}", $error_code);
    };
    ($condition:expr $(,)?, $error_code:expr, $msg:expr $(,)?) => {
        assert!($condition, "{}: {}", $error_code, $msg);
    };
    ($condition:expr $(,)?, $error_code:expr, $arg:ident: *const c_char $(,)?) => {
        assert!($condition, "{}: {}", $error_code,
            if $arg.is_null() {
                "<null>"
            } else {
                unsafe { CStr::from_ptr($arg).to_str().unwrap_or("<invalid>") }
            }
        );
    };
    ($condition:expr $(,)?, $error_code:expr, $fmt:expr, $($arg:tt)+) => {
        assert!($condition, "{}: {}", $error_code, format!($fmt, $($arg)+));
    };
}

#[cfg(not(feature = "flecs_ecs_asserts"))]
#[macro_export]
macro_rules! ecs_assert {
    ($($args:tt)*) => {};
}
