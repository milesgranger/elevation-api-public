extern crate netcdf;
extern crate ndarray;

use std::path::Path;
use std::collections::HashMap;
use ndarray::ArrayD;
use std::f64;
use netcdf::variable::Variable;
use netcdf::file::File;



struct Elevation {
    data: ArrayD<f64>,
    lats: ArrayD<f64>,
    lons: ArrayD<f64>
}

impl Elevation {

    fn new(path: &Path) -> Elevation {
        // Create a new elevation resource

        let file: File = netcdf::open(path.to_str().unwrap()).unwrap();

        let lats: ArrayD<f64> = file.root.variables.get("lat").unwrap().as_array().unwrap();
        let lons: ArrayD<f64> = file.root.variables.get("lon").unwrap().as_array().unwrap();
        let data = file.root.variables.get("Band1").unwrap().as_array().unwrap();

        Elevation {
            data,
            lats,
            lons
        }
    }

    fn find_closest_index(&self, array: &ArrayD<f64>, f: &f64) -> usize {
        /*
            Locate the index in "array" where the difference is smallest
            between that element and "f"
        */
        let mut min = f64::MAX;
        let mut min_index: usize = 0;
        for (i, val) in array.iter().enumerate() {
            if (f - val).abs() < min {
                min = (f - val).abs();
                min_index = i;
            }
        }
        //println!("Returning index of: {} - {}", &min_index, array[[min_index,]]);
        min_index
    }


    fn get_elevation(&self, lat: f64, lon: f64) -> f64 {
        let lat_index = self.find_closest_index(&self.lats, &lat);
        let lon_index = self.find_closest_index(&self.lons, &lon);
        self.data[[lat_index, lon_index]]
    }
}


fn main() {

    let lat = 22.235417;
    let lon = -159.403751;

    let path = Path::new("/home/milesg/Projects/elevation-api/data/out.nc");
    println!("File exists?: {}", path.exists());

    let elevation = Elevation::new(path);
    let val = elevation.get_elevation(lat, lon);

    println!("Elevation: {}", val);

}
