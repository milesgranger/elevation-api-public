#[macro_use] pub extern crate serde_derive;
#[macro_use] pub extern crate log;
pub extern crate netcdf;
pub extern crate ndarray;
pub extern crate glob;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate actix_web;
pub extern crate env_logger;
extern crate clap;
extern crate actix;
#[macro_use]
extern crate tera;


use actix_web::{
    http, middleware, server, App, Error, HttpResponse, HttpRequest, Query, Responder, State, fs, Json, Result
};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::path::{Path, PathBuf};

use clap::{Arg, App as ClapApp, SubCommand};
use glob::glob;


// Local mods
#[cfg(test)]
mod tests;
mod elevation;
mod json_structs;
use json_structs::{Points};


struct AppState {
    template: tera::Tera,
}

// Sanity check
fn index((state, _query): (State<AppState>, Query<HashMap<String, String>>)) -> Result<HttpResponse, Error> {
    let mut context = HashMap::new();
    context.insert("title".to_string(), "Free Elevation API".to_string());

    let s = state
        .template
        .render("index.html", &context)
        .unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(s))

}

// Main API for 90m resolution
fn get_elevations(req: &HttpRequest<AppState>) -> impl Responder {

    let points_str = req.query().get("points").unwrap().to_owned();
    info!("Got the points string!");
    let metas = elevation::load_summary_file();
    let points = Points::from_str(&points_str).expect("Unable to parse points!");

    let elevations = elevation::get_elevations(points.points, &metas);
    let elevations_resp = elevation::Elevations{ elevations };

    Json(elevations_resp)


}


fn main() {

    env::set_var("RUST_LOG", "actix_web=debug");
    //env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();
    info!("Starting up!");

    let matches = ClapApp::new("Elevation API")
        .version("1.0")
        .author("Miles Granger")
        .about("Web service and utility for giving elevations for locations on earth")
        .subcommand(
            SubCommand::with_name("run-server")
                .about("Run the elevation server")
                .arg(
                    Arg::with_name("DATA-DIR")
                        .required(true)
                        .help("Full path to directory containing netcdf files for elevations")
                        .takes_value(true)
                )
        )
        .get_matches();


    if let Some(m) = matches.subcommand_matches("run-server") {
        let data_dir = m.value_of("DATA-DIR").expect("No data directory specified!");

        // check that the directory actually exists
        if !Path::new(&data_dir).exists() {
            panic!("Data directory '{}' does not exist!", &data_dir);
        }

        // Check for summary.json, if not available in that directory, create it.
        let mut summary_file_path: PathBuf = data_dir.into();
        summary_file_path.push("summary.json");
        if !summary_file_path.exists() {
            info!("Summary file not found, creating it at {}", summary_file_path.to_str().unwrap());
            elevation::make_summary_file(&data_dir)
        }

        // Set env var for location of data
        env::set_var("DATA_DIR", &data_dir);

        // Server
        let sys = actix::System::new("elevation-api");

        server::new(|| {

                let tera = compile_templates!("./templates/**/*");

                App::with_state(AppState {template: tera})

                    // Logging
                    .middleware(middleware::Logger::default())

                    // Application base route
                    .prefix("/")

                    // Static files
                    .handler(
                        "/static",
                        fs::StaticFiles::new("static")
                            .expect("Can't find static directory!")
                            .show_files_listing())

                    // Homepage
                    .resource("/", |r| r.method(http::Method::GET).with(index))

                    // Main elevation API
                    .resource("/api/elevation", |r| r.method(http::Method::GET).f(get_elevations))
            })
            .bind("0.0.0.0:8000")
            .expect("Unable to bind to 0.0.0.0:8000")
            .start();

        info!("Started server running on 0.0.0.0:8000");
        let _ = sys.run();


    } else {
        warn!("Nothing to do, exiting!");
    }
}
