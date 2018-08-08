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
#[macro_use] extern crate log;
extern crate env_logger;
extern crate clap;


use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;
use clap::{Arg, App, SubCommand};
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
#[get("/api/elevation?<points>")]
fn get_elevations(points: Option<Points>) -> Result<Json<elevation::ElevationResponse>, BadRequest<String>> {

    match points {
        Some(points) => {
            let metas = elevation::load_summary_file();
            let elevations = elevation::get_elevations(points.points.0, &metas);

            let elevation_response = elevation::ElevationResponse{points: elevations};
            Ok(Json(elevation_response))
        },
        None => {
            Err(BadRequest(Some("Unable to parse coordinates. Should be in form '(lat,lon),(lat,lon),(lat,lon)'".to_string())))
        }
    }

}


fn main() {

    env_logger::init();
    info!("Starting up!");

    let matches = App::new("Elevation API")
        .version("1.0")
        .author("Miles Granger")
        .about("Web service and utility for giving elevations for locations on earth")
        .subcommand(
            SubCommand::with_name("make-summary")
                .about("Look at available NetCDF files in the directory and write out summary.json")
                .arg(
                    Arg::with_name("PATH")
                        .required(true)
                        // TODO: Accept paths ending with "/"
                        .help("Path to the folder containing NetCDF files, NOT ending with a slash")
                        .takes_value(true)
                )
        )
        .subcommand(
            SubCommand::with_name("run-server")
                .about("Run the elevation server")
                .arg(
                    Arg::with_name("SUMMARY-FILE")
                        .required(true)
                        .help("Full path location of the summary.json file")
                        .takes_value(true)
                )
        )
        .get_matches();


    if let Some(m) = matches.subcommand_matches("make-summary") {
        let path = m.value_of("PATH").expect("No path specified");
        elevation::make_summary_file(path);

    } else if let Some(m) = matches.subcommand_matches("run-server") {
        let summary_file = m.value_of("SUMMARY-FILE").expect("No path specified!");
        env::set_var("SUMMARY_FILE_PATH", summary_file);
        rocket::ignite()
            .mount("/", routes![index, get_elevations, static_files])
            .attach(Template::fairing())
            .launch();
    } else {
        warn!("Nothing to do, exiting!");
    }
}
