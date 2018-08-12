#[macro_use] pub extern crate serde_derive;
#[macro_use] extern crate log;
pub extern crate netcdf;
pub extern crate ndarray;
pub extern crate glob;
pub extern crate serde;
pub extern crate serde_json;
extern crate env_logger;
extern crate clap;
extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate tera;


use actix_web::{
    error, http, middleware, server, App, Error, HttpResponse, Query, State, fs
};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;

use clap::{Arg, App as ClapApp, SubCommand};
use glob::glob;


// Local mods
#[cfg(test)]
mod tests;
mod elevation;
mod json_structs;
use json_structs::{Points};


struct TeraAppState {
    template: tera::Tera,
}

// Sanity check
fn index((state, query): (State<TeraAppState>, Query<HashMap<String, String>>)) -> Result<HttpResponse, Error> {
    let mut context = HashMap::new();
    context.insert("title".to_string(), "Free Elevation API".to_string());

    let s = state
        .template
        .render("index.tera.html", &context)
        .unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(s))

}

// Main API for 90m resolution
fn get_elevations(points: Option<Points>) -> () //Result<Json<elevation::ElevationResponse>, BadRequest<String>>
{

    match points {
        Some(points) => {
            let metas = elevation::load_summary_file();
            let elevations = elevation::get_elevations(points.points.0, &metas);

            let elevation_response = elevation::ElevationResponse{points: elevations};
            //Ok(Json(elevation_response))
        },
        None => {
            println!("None!");
            //Err(BadRequest(Some("Unable to parse coordinates. Should be in form '(lat,lon),(lat,lon),(lat,lon)'".to_string())))
        }
    };

}


fn main() {

    env_logger::init();
    info!("Starting up!");

    let matches = ClapApp::new("Elevation API")
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

        // Server
        let sys = actix::System::new("elevation-api");

        server::new(|| {
                let tera = compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"));
                App::with_state(TeraAppState{template: tera})
                    .prefix("/")

                    // Static files
                    .handler(
                        "/static",
                        fs::StaticFiles::new("static")
                            .expect("Can't find static directory!")
                            .show_files_listing())
                    .resource("/", |r| r.method(http::Method::GET).with(index))
            })
            .bind("0.0.0.0:8000")
            .expect("Unable to bind to 0.0.0.0:8000")
            .workers(1)
            .start();

        info!("Started server running on 0.0.0.0:8000");
        let _ = sys.run();


    } else {
        warn!("Nothing to do, exiting!");
    }
}
