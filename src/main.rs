#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
pub extern crate rocket;
#[macro_use] pub extern crate rocket_contrib;
pub extern crate netcdf;
pub extern crate ndarray;
pub extern crate glob;
pub extern crate serde;
pub extern crate serde_json;
#[macro_use] pub extern crate serde_derive;

use std::path::Path;
use std::str::FromStr;
use std::str::Utf8Error;
use glob::glob;
use rocket_contrib::Json;
use rocket::request;
use rocket::http;

#[cfg(test)]
mod tests;
mod elevation;

#[get("/")]
fn index() -> &'static str {
    "Up and running!"
}


/// Struct to represent a JSON query parameter for a given location
#[derive(FromForm, Serialize, Deserialize)]
pub struct Points {
    points: CoordinateList
}

#[derive(Serialize, Deserialize)]
pub struct CoordinateList(Vec<(f64, f64)>);

impl FromStr for CoordinateList {
    type Err = Utf8Error;

    // "?points=(45.9,34.8)(34.3,28.9)".split('(') -> ["45.9,34.8)", "34.3,28.9)"].map(
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let points: Vec<&str> = s.split('(').collect::<Vec<&str>>();
        let points: Vec<&str> = points.iter().map(|v| v.trim_matches(')')).collect();
        let points: Vec<Vec<f64>> = points.iter()
            .map(|v| v.split(',')
            .map(|v| v.parse::<f64>().expect("Failed to parse string to f64")).collect())
            .collect();
        let points: Vec<(f64, f64)> = points.iter().map(|v| (v[0], v[1])).collect();
        Ok(CoordinateList(points))
    }
}


impl<'v> request::FromFormValue<'v> for CoordinateList {
    type Error = &'v http::RawStr;

    fn from_form_value(form_value: &'v http::RawStr) -> Result<CoordinateList, &'v http::RawStr> {
        match form_value.parse::<CoordinateList>() {
            Ok(points) => Ok(points),
            _ => Err(form_value)
        }
    }
}


#[get("/v1/elevations?<points>")]
fn get_elevations(points: Points) -> Json<Points> {
    Json(points)
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .launch();
}
