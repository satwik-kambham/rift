use crate::primitive::Primitive;
use crate::std_lib::args;

pub fn floor(arguments: Vec<Primitive>) -> Primitive {
    let value = args!(arguments; value: Number);
    Primitive::Number(value.floor())
}
