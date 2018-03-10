pub extern crate netcdf;
pub extern crate ndarray;

use std::path::Path;

mod elevation;
use elevation::Elevation;


fn main() {

    let lat = 22.235417;
    let lon = -159.403751;

    let path = Path::new("/home/milesg/Projects/elevation-api/data/out.nc");
    println!("File exists?: {}", path.exists());

    let elevation = Elevation::new(path);
    let val = elevation.get_elevation(lat, lon);

    println!("Elevation: {} - Lat bounds: {:?}, Lon bounds: {:?}", val, elevation.lat_min_max, elevation.lon_min_max);

}
