#[macro_use]
extern crate cpython;
#[macro_use]
extern crate crowbar;
#[macro_use]
extern crate serde_json;

lambda!(|event, _| {
    println!("invoked with {:?}", event);

    let coords: &str = event.get("queryStringParameters").unwrap().get("extra-parameter").unwrap().as_str().unwrap();

    let body: String = format!(r#"{{ "example": "data", "to_return": 2, "values": {} }}"#, coords);

    Ok(json!({
        "headers": {
            "Access-Control-Allow-Origin": "*"
        },
        "statusCode": 200,
        "body": body
    }))
});
