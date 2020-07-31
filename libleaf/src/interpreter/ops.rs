use crate::interpreter::{types::TypeIdAndValue, Value, ValueTypeMarker};
use std::ops::Range;

pub(crate) fn op_add(
    ls: &Value<Box<dyn ValueTypeMarker>>,
    _left_range: Range<usize>,
    rs: &Value<Box<dyn ValueTypeMarker>>,
    _right_range: Range<usize>,
) -> Value<Box<dyn ValueTypeMarker>> {
    match (
        ls.get_value().get_type_id_and_value(),
        rs.get_value().get_type_id_and_value(),
    ) {
        (TypeIdAndValue::String(left), right) => {
            Value::new(Box::new(format!("{}{}", left, right.stringify())))
        }
        (left, TypeIdAndValue::String(right)) => {
            Value::new(Box::new(format!("{}{}", left.stringify(), right)))
        }
        (TypeIdAndValue::I32(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval + *rightval)),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval as i64 + *rightval)),
            _ => {
                panic!("Cannot add {} to {}", TypeIdAndValue::I32(leftval), right);
            }
        },
        (TypeIdAndValue::I64(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval + (*rightval as i64))),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval + *rightval)),
            _ => {
                panic!("Cannot add {} to {}", TypeIdAndValue::I64(leftval), right);
            }
        },
        (TypeIdAndValue::U32(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval + *rightval)),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval as u64 + *rightval)),
            _ => {
                panic!("Cannot add {} to {}", TypeIdAndValue::U32(leftval), right);
            }
        },
        (TypeIdAndValue::U64(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval + (*rightval as u64))),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval + *rightval)),
            _ => {
                panic!("Cannot add {} to {}", TypeIdAndValue::U64(leftval), right);
            }
        },
    }
}

pub(crate) fn op_sub(
    ls: &Value<Box<dyn ValueTypeMarker>>,
    _left_range: Range<usize>,
    rs: &Value<Box<dyn ValueTypeMarker>>,
    _right_range: Range<usize>,
) -> Value<Box<dyn ValueTypeMarker>> {
    match (
        ls.get_value().get_type_id_and_value(),
        rs.get_value().get_type_id_and_value(),
    ) {
        (TypeIdAndValue::I32(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval - *rightval)),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval as i64 - *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} - {}'",
                    TypeIdAndValue::I32(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::I64(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval - (*rightval as i64))),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval - *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} - {}'",
                    TypeIdAndValue::I64(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::U32(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval - *rightval)),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval as u64 - *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} - {}'",
                    TypeIdAndValue::U32(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::U64(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval - (*rightval as u64))),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval - *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} - {}'",
                    TypeIdAndValue::U64(leftval),
                    right
                );
            }
        },
        (left, right) => panic!("Cannot perform '{} - {}'", right, left),
    }
}

pub(crate) fn op_mul(
    ls: &Value<Box<dyn ValueTypeMarker>>,
    _left_range: Range<usize>,
    rs: &Value<Box<dyn ValueTypeMarker>>,
    _right_range: Range<usize>,
) -> Value<Box<dyn ValueTypeMarker>> {
    match (
        ls.get_value().get_type_id_and_value(),
        rs.get_value().get_type_id_and_value(),
    ) {
        (TypeIdAndValue::I32(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval * *rightval)),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval as i64 * *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} * {}'",
                    TypeIdAndValue::I32(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::I64(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval * (*rightval as i64))),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval * *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} * {}'",
                    TypeIdAndValue::I64(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::U32(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval * *rightval)),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval as u64 * *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} * {}'",
                    TypeIdAndValue::U32(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::U64(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval * (*rightval as u64))),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval * *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} * {}'",
                    TypeIdAndValue::U64(leftval),
                    right
                );
            }
        },
        (left, right) => panic!("Cannot perform '{} * {}'", right, left),
    }
}

pub(crate) fn op_div(
    ls: &Value<Box<dyn ValueTypeMarker>>,
    _left_range: Range<usize>,
    rs: &Value<Box<dyn ValueTypeMarker>>,
    _right_range: Range<usize>,
) -> Value<Box<dyn ValueTypeMarker>> {
    match (
        ls.get_value().get_type_id_and_value(),
        rs.get_value().get_type_id_and_value(),
    ) {
        (TypeIdAndValue::I32(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval / *rightval)),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval as i64 / *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} / {}'",
                    TypeIdAndValue::I32(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::I64(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval / (*rightval as i64))),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval / *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} / {}'",
                    TypeIdAndValue::I64(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::U32(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval / *rightval)),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval as u64 / *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} / {}'",
                    TypeIdAndValue::U32(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::U64(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval / (*rightval as u64))),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval / *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} / {}'",
                    TypeIdAndValue::U64(leftval),
                    right
                );
            }
        },
        (left, right) => panic!("Cannot perform '{} / {}'", right, left),
    }
}

pub(crate) fn op_mod(
    ls: &Value<Box<dyn ValueTypeMarker>>,
    _left_range: Range<usize>,
    rs: &Value<Box<dyn ValueTypeMarker>>,
    _right_range: Range<usize>,
) -> Value<Box<dyn ValueTypeMarker>> {
    match (
        ls.get_value().get_type_id_and_value(),
        rs.get_value().get_type_id_and_value(),
    ) {
        (TypeIdAndValue::I32(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval % *rightval)),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval as i64 % *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} % {}'",
                    TypeIdAndValue::I32(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::I64(leftval), right) => match right {
            TypeIdAndValue::I32(rightval) => Value::new(Box::new(*leftval % (*rightval as i64))),
            TypeIdAndValue::I64(rightval) => Value::new(Box::new(*leftval % *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} % {}'",
                    TypeIdAndValue::I64(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::U32(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval % *rightval)),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval as u64 % *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} % {}'",
                    TypeIdAndValue::U32(leftval),
                    right
                );
            }
        },
        (TypeIdAndValue::U64(leftval), right) => match right {
            TypeIdAndValue::U32(rightval) => Value::new(Box::new(*leftval % (*rightval as u64))),
            TypeIdAndValue::U64(rightval) => Value::new(Box::new(*leftval % *rightval)),
            _ => {
                panic!(
                    "Cannot perform '{} % {}'",
                    TypeIdAndValue::U64(leftval),
                    right
                );
            }
        },
        (left, right) => panic!("Cannot perform '{} % {}'", right, left),
    }
}
