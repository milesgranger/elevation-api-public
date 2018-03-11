pub extern crate netcdf;
pub extern crate ndarray;
pub extern crate glob;
pub extern crate serde;
pub extern crate serde_json;
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

    let lats = ndarray::Array::linspace(45.08904, 55.08904, 5);
    let lons = ndarray::Array::linspace(80.85938, 85.85938, 5);

    for lat in lats.iter() {
        for lon in lons.iter() {
            let elevation = utils::get_elevation(lat, lon, &meta_data);

            match elevation {
                Some(meters) => println!("Elevation: {} meters at ({}, {})", meters, lat, lon),
                None => println!("Unknown Elevation for ({}, {})", lat, lon)
            }
        }
    }

    //let path = Path::new("/home/milesg/Projects/elevation-api/data/out.nc");
    //println!("File exists?: {}", path.exists());

    //let elevation = ElevationTile::new(path);
    //let val = elevation.get_elevation(lat, lon);

    //println!("Elevation: {} - Lat bounds: {:?}, Lon bounds: {:?}", val, elevation.lat_min_max, elevation.lon_min_max);

}
