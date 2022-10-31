use my_serde::{Deserialize, Serialize};
use serde as my_serde;
use typesensei::{state::FieldState, Typesense};

#[derive(Debug, Serialize, Deserialize, Typesense)]
pub struct One {
    field0: u32,
    #[typesensei(facet = true)]
    field1: String,
    #[serde(flatten)]
    json: serde_json::Value,
    some: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Typesense)]
pub struct Two {
    field2: u32,
    #[typesensei(index = false)]
    field3: Option<String>,
    #[typesensei(index = false)]
    field4: String,
}

#[tokio::test]
async fn test_derive() {
    let client = typesensei::Client::builder()
        .hostname("http://127.0.0.1:8108")
        .api_key("xyz")
        .build()
        .unwrap();

    // let res = client.collection::<One>().create().await.unwrap();

    let mut one = One::model();
    one.field0.set(123);
    one.field1.set("hello world".to_owned());
    one.some.set(Some(11));
    one.json["field2"] = 5332.into();
    one.json["field3"] = serde_json::json!("something");

    let res = client.documents::<One>().create(&one).await.unwrap();

    println!("res: {res:#?}");

    let mut query = One::query();
    query.field0.greater_or_equals(123).sort_asc();
    query.field1.query_by();
    query
        .json
        .filter_by("field2:>=5332".to_owned())
        .query_by("field3".to_owned());
    let query = query.q("hello world".to_owned());

    println!("q: {}", serde_json::to_string_pretty(&query).unwrap());

    let res = client.documents::<One>().search(&query).await.unwrap();

    println!("res: {:#?}", res);

    // let doc = client
    //     .collection::<One>()
    //     .documents()
    //     .retrieve("1")
    //     .await
    //     .unwrap();

    // println!("doc: {:?}", doc);
    // println!("schema: {:#?}", One::schema());
}
