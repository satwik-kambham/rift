pub mod array;
pub mod io;
pub mod number;
pub mod string;
pub mod table;
pub mod web_requests;

use crate::primitive::Primitive;
use rsl_macros::rsl_native;

macro_rules! args {
    ($args:expr; $($name:ident $( : $variant:ident )?),+ $(,)?) => {{
        let mut iter = $args.into_iter();
        let parsed = ($(
            match iter.next() {
                Some(value) => {
                    $(
                        let Primitive::$variant(value) = value else {
                            return Primitive::Error(
                                concat!("expected ", stringify!($variant)).to_string()
                            );
                        };
                    )?
                    value
                }
                _ => return Primitive::Error("missing argument".to_string()),
            }
        ),+);

        if iter.next().is_some() {
            return Primitive::Error("too many arguments".to_string());
        }

        parsed
    }};
}

pub(crate) use args;

#[rsl_native]
pub fn print(arguments: Vec<Primitive>) -> Primitive {
    let text = arguments
        .iter()
        .map(|arg| format!("{}", arg))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", text);
    Primitive::Null
}

#[rsl_native]
pub fn to_string(arguments: Vec<Primitive>) -> Primitive {
    let value = args!(arguments; value);
    Primitive::String(format!("{}", value))
}

#[rsl_native]
pub fn to_json(arguments: Vec<Primitive>) -> Primitive {
    let argument = args!(arguments; argument);
    let serialized = serde_json::to_string(&argument).unwrap();
    Primitive::String(serialized)
}

#[rsl_native]
pub fn from_json(arguments: Vec<Primitive>) -> Primitive {
    let json = args!(arguments; json: String);
    serde_json::from_str(&json).unwrap()
}
