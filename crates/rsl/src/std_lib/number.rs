use crate::primitive::Primitive;
use crate::std_lib::args;
use rsl_macros::rsl_native;

#[rsl_native]
pub fn floor(arguments: Vec<Primitive>) -> Primitive {
    let value = args!(arguments; value: Number);
    Primitive::Number(value.floor())
}
