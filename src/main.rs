#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
pub extern crate rocket;
pub extern crate rocket_contrib;
pub extern crate netcdf;
pub extern crate ndarray;
pub extern crate glob;
pub extern crate serde;
#[macro_use] pub extern crate serde_json;
#[macro_use] pub extern crate serde_derive;

use glob::glob;
use rocket_contrib::Json;

// Local mods
#[cfg(test)]
mod tests;
mod elevation;
mod json_structs;
use json_structs::{Points, Message};


// Sanity check
#[get("/")]
fn index() -> &'static str {
    "Up and running!"
}



// Main API for 90m resolution
#[get("/api/elevations/90m?<points>")]
fn get_elevations(points: Option<Points>) -> Json<Message> {

    match points {
        Some(points) => {
            let metas = elevation::load_summary_file();
            let elevations = elevation::get_elevations(points.points.0, &metas);
            Json(Message::Success{message: elevations})
        },
        None => {
            Json(Message::Error{message: "Failed to parse request".to_string()})
        }
    }

}


fn main() {
    rocket::ignite()
        .mount("/", routes![index, get_elevations])
        .launch();

}
