#[macro_use]
extern crate cpython;
#[macro_use]
extern crate crowbar;
#[macro_use]
extern crate serde_json;

lambda!(|event, _| {
    println!("invoked with {:?}", event);
    Ok(json!({
        "headers": {
            "Access-Control-Allow-Origin": "*"
        },
        "statusCode": 200,
        "body": "{ \"example\": \"data\", \"to_return\": 2 }"
    }))
});
