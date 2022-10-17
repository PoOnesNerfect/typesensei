use serde::{Deserialize, Serialize};
use typesensei::Typesense;

#[derive(Debug, Serialize, Deserialize, Typesense)]
pub struct Something {
    id: String,
    field0: u32,
    #[typesense(index = false)]
    field1: String,
    #[serde(flatten)]
    something_else: SomethingElse,
}

#[derive(Debug, Serialize, Deserialize, Typesense)]
pub struct SomethingElse {
    id: u32,
    field2: u32,
    field3: String,
}

#[test]
fn test_derive() {
    let something = Something {
        id: "hi".to_owned(),
        field0: 1,
        field1: "hello".to_owned(),
        something_else: SomethingElse {
            id: 12,
            field2: 123,
            field3: "hello world".to_owned(),
        },
    };
    let json = serde_json::to_string(&something).unwrap();

    println!("json: {json}");
}
