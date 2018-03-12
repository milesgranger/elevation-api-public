#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
pub extern crate rocket;
pub extern crate rocket_contrib;
pub extern crate netcdf;
pub extern crate ndarray;
pub extern crate glob;
pub extern crate serde;
pub extern crate serde_json;
#[macro_use] pub extern crate serde_derive;

use std::f64;
use std::str::FromStr;
use glob::glob;
use rocket_contrib::Json;

#[cfg(test)]
mod tests;
mod elevation;
mod json_structs;
use json_structs::{Points};

#[get("/")]
fn index() -> &'static str {
    "Up and running!"
}


#[get("/api/elevations/90m?<points>")]
fn get_elevations(points: Points) -> Json<Vec<elevation::Elevation>> {
    let metas = elevation::load_summary_file();
    let elevations = elevation::get_elevations(points.points.0, &metas);
    Json(elevations)
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, get_elevations])
        .launch();

}
