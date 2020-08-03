use crate::interpreter::{CallExecutor, CallPool, Value, ValueTypeMarker};

use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

// #[derive(PartialOrd, PartialEq, Debug)]
pub(crate) enum TypeIdAndValue<'a> {
    I32(&'a i32),
    I64(&'a i64),
    U32(&'a u32),
    U64(&'a u64),
    Bool(&'a bool),
    String(&'a String),
    Void,
    Error,
    Vec(&'a Vec<Value<Box<dyn ValueTypeMarker>>>),
    Map(&'a HashMap<String, Value<Box<dyn ValueTypeMarker>>>),
}

impl<'a> PartialEq for TypeIdAndValue<'a> {
    fn eq(&self, other: &Self) -> bool {
        unimplemented!()
    }

    fn ne(&self, other: &Self) -> bool {
        unimplemented!()
    }
}

impl<'a> PartialOrd for TypeIdAndValue<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unimplemented!()
    }

    fn lt(&self, other: &Self) -> bool {
        unimplemented!()
    }

    fn le(&self, other: &Self) -> bool {
        unimplemented!()
    }

    fn gt(&self, other: &Self) -> bool {
        unimplemented!()
    }

    fn ge(&self, other: &Self) -> bool {
        unimplemented!()
    }
}

impl<'a> TypeIdAndValue<'a> {
    pub(crate) fn stringify(&self) -> String {
        match self {
            TypeIdAndValue::I32(v) => format!("{}", v),
            TypeIdAndValue::I64(v) => format!("{}", v),
            TypeIdAndValue::U32(v) => format!("{}", v),
            TypeIdAndValue::U64(v) => format!("{}", v),
            TypeIdAndValue::Bool(v) => format!("{}", v),
            TypeIdAndValue::String(v) => (*v).clone(),
            TypeIdAndValue::Void => "(void)".to_string(),
            TypeIdAndValue::Error => "(error)".to_string(),
            TypeIdAndValue::Vec(v) => v.stringify(),
            TypeIdAndValue::Map(v) => v.stringify(),
        }
    }

    #[inline]
    pub(crate) fn degrade(&self) -> TypeId {
        match self {
            TypeIdAndValue::I32(_) => TypeId::I32,
            TypeIdAndValue::I64(_) => TypeId::I64,
            TypeIdAndValue::U32(_) => TypeId::U32,
            TypeIdAndValue::U64(_) => TypeId::U64,
            TypeIdAndValue::Bool(_) => TypeId::Bool,
            TypeIdAndValue::String(_) => TypeId::String,
            TypeIdAndValue::Void => TypeId::Void,
            TypeIdAndValue::Error => TypeId::Error,
            TypeIdAndValue::Vec(_) => TypeId::Vec,
            TypeIdAndValue::Map(_) => TypeId::Map,
        }
    }
}

#[derive(PartialOrd, PartialEq, Debug)]
pub(crate) enum TypeId {
    I32,
    I64,
    U32,
    U64,
    Bool,
    String,
    Void,
    Error,
    Vec,
    Map,
}

impl TypeId {
    pub(crate) fn typename(&self) -> &'static str {
        match self {
            TypeId::I32 => "i32",
            TypeId::I64 => "i64",
            TypeId::U32 => "u32",
            TypeId::U64 => "u64",
            TypeId::Bool => "bool",
            TypeId::String => "string",
            TypeId::Void => "void",
            TypeId::Error => "error",
            TypeId::Vec => "vec",
            TypeId::Map => "map",
        }
    }
}

impl Display for TypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.typename())
    }
}

include!("int.rs");
include!("bool.rs");
include!("string.rs");
include!("void.rs");
include!("error.rs");
include!("vec.rs");
include!("map.rs");
