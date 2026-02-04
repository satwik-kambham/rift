use rsl::environment::Environment;
use rsl::primitive::Primitive;
use rsl::register_native_functions;
use rsl_macros::rsl_native;

#[rsl_native]
fn sample_native(_arguments: Vec<Primitive>) -> Primitive {
    Primitive::Null
}

#[test]
fn registers_native_functions() {
    let environment = Environment::new(None);
    register_native_functions(&environment);

    let value = environment.get_value("sampleNative");
    assert!(matches!(value, Primitive::Function(_)));
}
