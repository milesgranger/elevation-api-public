extern crate netcdf;
extern crate ndarray;

use std::path::Path;
use ndarray::ArrayD;
use std::f64;


fn find_min_index(array: &ArrayD<f64>, f: &f64) -> usize {

    let mut min = f64::MAX;
    let mut min_index: usize = 0;
    for (i, val) in array.iter().enumerate() {
        if (f - val).abs() < min {
            min = (f - val).abs();
            min_index = i;
        }
    }
    println!("Returning index of: {}", &min_index);
    min_index
}


fn main() {

    let lat = 22.235417;
    let lon = -159.403750;

    let path = Path::new("/home/milesg/Projects/elevation-api/data/out.nc");
    println!("File exists?: {}", path.exists());

    let file = netcdf::open(path.to_str().unwrap()).unwrap();

    for key in file.root.variables.keys() {
        println!("Variable: {}", key);
    }

    let lats: ArrayD<f64> = file.root.variables.get("lat").unwrap().as_array().unwrap();
    let lons: ArrayD<f64> = file.root.variables.get("lon").unwrap().as_array().unwrap();
    let data = file.root.variables.get("Band1").unwrap();

    let lat_index = find_min_index(&lats, &lat);
    let lon_index = find_min_index(&lons, &lon);

    let v: f64 = data.value_at(&[lat_index, lon_index]).unwrap();

    println!("Data size: {}, elevation: {}", lats.len(), v);

}
