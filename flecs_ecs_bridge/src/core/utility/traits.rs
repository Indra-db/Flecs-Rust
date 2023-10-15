use crate::core::{
    c_types::{InOutKind, OperKind},
    component_registration::CachedComponentData,
};

pub(crate) trait InOutType {
    const IN_OUT: InOutKind;
    type Type: CachedComponentData;
}

pub(crate) trait OperType {
    const OPER: OperKind;
    type Type: CachedComponentData;
}
