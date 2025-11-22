pub mod array;
pub mod io;
pub mod string;
pub mod table;
pub mod web_requests;

use crate::primitive::Primitive;

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

pub fn print(arguments: Vec<Primitive>) -> Primitive {
    let text = arguments
        .iter()
        .map(|arg| format!("{}", arg))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", text);
    Primitive::Null
}

pub fn to_json(arguments: Vec<Primitive>) -> Primitive {
    let argument = args!(arguments; argument);
    let serialized = serde_json::to_string(&argument).unwrap();
    Primitive::String(serialized)
}

pub fn from_json(arguments: Vec<Primitive>) -> Primitive {
    let json = args!(arguments; json: String);
    serde_json::from_str(&json).unwrap()
}
