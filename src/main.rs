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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use glob::glob;
use rocket_contrib::{Json, Template};
use rocket::response::status::BadRequest;
use rocket::response::NamedFile;

// Local mods
#[cfg(test)]
mod tests;
mod elevation;
mod json_structs;
use json_structs::{Points};


// Sanity check
#[get("/")]
fn index() -> Template {
    let mut context = HashMap::new();
    context.insert("title".to_string(), "Free Elevation API".to_string());
    Template::render("index", &context)
}


#[get("/<file..>")]
fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}


// Main API for 90m resolution
#[get("/api/v1.0.0/90m?<points>")]
fn get_elevations_v1_0_0(points: Option<Points>) -> Result<Json<elevation::ElevationResponse>, BadRequest<String>> {

    match points {
        Some(points) => {
            let metas = elevation::load_summary_file();
            let elevations = elevation::get_elevations(points.points.0, &metas);
            let elevation_response = elevation::ElevationResponse{points: elevations};
            Ok(Json(elevation_response))
        },
        None => {
            Err(BadRequest(Some("Unable to parse coordinates. Should be in form '(lat, lon),(lat,lon),(lat,lon)'".to_string())))
        }
    }

}


fn main() {
    rocket::ignite()
        .mount("/", routes![index, get_elevations_v1_0_0, static_files])
        .attach(Template::fairing())
        .launch();

}
