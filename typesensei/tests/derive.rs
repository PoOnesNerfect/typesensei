use my_serde::{Deserialize, Serialize};
use serde as my_serde;
use typesensei::Typesense;

#[derive(Debug, Serialize, Deserialize, Typesense)]
pub struct One<S, T> {
    field0: u32,
    #[typesensei(index = false)]
    field1: String,
    #[serde(flatten)]
    two: Option<S>,
    some: Option<T>,
}

#[derive(Debug, Serialize, Deserialize, Typesense)]
pub struct Two {
    field2: u32,
    #[typesensei(index = false)]
    field3: Option<String>,
}

#[test]
fn test_derive() {
    let mut one = One::<Two, u8>::model();
    one.field0.set(123);
    one.some.unset();
    one.id.set(12);

    let two = Two {
        field2: 123,
        field3: Some("hihi".to_owned()),
    };

    one.two.replace(two.into());

    let json = serde_json::to_string(&one).unwrap();
    println!("json: {json}");

    let one: OneModel<TwoModel, u8> = serde_json::from_str(&json).unwrap();

    println!("one: {one:#?}");
}
