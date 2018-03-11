pub extern crate netcdf;
pub extern crate ndarray;
pub extern crate glob;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate rayon;
#[macro_use] pub extern crate serde_derive;

use std::path::Path;
use glob::glob;

mod elevation;
mod utils;

fn main() {

    //let lat = 22.235417;
    //let lon = -159.403751;

    // Make summary file if it doesn't exist
    if !Path::new("./data/summary.json").exists() {
        utils::make_summary_file();
    }

    let meta_data = utils::load_summary_file();

    // Build collection of testing coordinates
    let lats = ndarray::Array::linspace(45.08904, 55.08904, 50);
    let lons = ndarray::Array::linspace(80.85938, 85.85938, 50);

    let mut coords = Vec::with_capacity(lats.len() * lons.len() as usize);

    for lat in lats.iter() {
        for lon in lons.iter() {
            coords.insert(0, (lat, lon));
        }
    }

    println!("Getting coords for {} locations", coords.len());
    let coords: Vec<(&f64, &f64)> = coords.to_vec();
    let elevations = utils::get_elevations(coords, &meta_data);
    println!("Finished!");
}
