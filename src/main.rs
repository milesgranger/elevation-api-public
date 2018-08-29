#![feature(duration_as_u128)]

#[macro_use] pub extern crate serde_derive;
#[macro_use] pub extern crate log;
#[macro_use] pub extern crate serde_json;
#[macro_use] extern crate tera;
#[macro_use] extern crate lazy_static;
pub extern crate netcdf;
pub extern crate ndarray;
pub extern crate glob;
pub extern crate serde;
pub extern crate actix_web;
pub extern crate env_logger;
extern crate clap;
extern crate actix;


use actix_web::{
    http, middleware, server, App, Error, HttpResponse, HttpRequest, Query, Responder, State, fs, Result
};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::{Arg, App as ClapApp, SubCommand};
use glob::glob;


// Local mods
#[cfg(test)]
mod tests;
mod elevation;
mod json_structs;
use json_structs::{Points};
use elevation::ElevationTileFileMetaData;


lazy_static! {
    static ref ELEVATION_METAS: Vec<ElevationTileFileMetaData> = elevation::load_summary_file();
}


struct AppState {
    template: tera::Tera,
}

// Sanity check
fn index((state, _query): (State<AppState>, Query<HashMap<String, String>>)) -> Result<HttpResponse, Error> {

    let start = Instant::now();

    let mut context = HashMap::new();
    context.insert("title".to_string(), "Free Elevation API".to_string());

    let s = state
        .template
        .render("index.html", &context)
        .unwrap();
    info!("Handled request for homepage in {}ms", start.elapsed().as_millis());
    Ok(HttpResponse::Ok().content_type("text/html").body(s))

}

// Main API for 90m resolution
fn get_elevations(req: &HttpRequest<AppState>) -> impl Responder {

    let start = Instant::now();

    let points_str = match req.query().get("points") {
        Some(pt_str) => pt_str.to_owned(),
        None => {
            return HttpResponse::Ok()
                    .json(
                        json!({ 
                            "message": "Please provide some coordinates! ie. http://elevation-api.io/api/elevation?points=(39.90974,-106.17188),(62.52417,10.02487)"
                            }))
        }
    };

    let points = Points::from_str(&points_str);

    match points {
        Ok(pts) => {

            let n_points = pts.len();

            if n_points > 50 {
                warn!("Recieved API request for more than 50 points, got {}! Aborting request", pts.len());
                return HttpResponse::BadRequest()
                    .json(json!({"message": "Requested more than 50 locations, please reduce the request size."}))
            }

            let elevations = elevation::get_elevations(&pts.points, &ELEVATION_METAS);
            let elevations_resp = elevation::Elevations{ elevations };

            info!("Successfully processed {} points in {}ms", n_points, start.elapsed().as_millis());
            HttpResponse::Ok()
                .json(elevations_resp)
        },
        Err(_) => {
            HttpResponse::BadRequest()
                .json(json!({"message": "Unable to parse one or more of the coordinates provided!"}))
        }
    }
}


fn main() {

    env::set_var("RUST_LOG", "actix_web,elevation_api=info");
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
