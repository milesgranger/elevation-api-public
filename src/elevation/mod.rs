use std::f64;
use std::path::Path;
use netcdf;
use netcdf::file::File;
use ndarray::ArrayD;


pub trait MinMax {
    fn min_value(&self) -> f64;
    fn max_value(&self) -> f64;
}


impl MinMax for ArrayD<f64> {
    
    fn min_value(&self) -> f64  {
        let mut val = &f64::MAX;
        for v in self.iter() {
            if v < val {
                val = v;
            }
        }
        *val
    }

    fn max_value(&self) -> f64  {
        let mut val = &f64::MIN;
        for v in self.iter() {
            if v > val {
                val = v;
            }
        }
        *val
    }
}


pub struct Elevation {
    pub data: ArrayD<f64>,
    pub lats: ArrayD<f64>,
    pub lons: ArrayD<f64>,
    pub lat_min_max: (f64, f64),
    pub lon_min_max: (f64, f64)
}

impl Elevation {

    pub fn new(path: &Path) -> Elevation {
        // Create a new elevation resource

        let file: File = netcdf::open(path.to_str().unwrap()).unwrap();

        let lats: ArrayD<f64> = file.root.variables.get("lat").unwrap().as_array().unwrap();
        let lons: ArrayD<f64> = file.root.variables.get("lon").unwrap().as_array().unwrap();
        let data = file.root.variables.get("Band1").unwrap().as_array().unwrap();

        let lat_min_max = (lats.min_value(), lats.max_value());
        let lon_min_max = (lons.min_value(), lons.max_value());

        Elevation {
            data,
            lats,
            lons,
            lat_min_max,
            lon_min_max
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


    pub fn get_elevation(&self, lat: f64, lon: f64) -> f64 {
        let lat_index = self.find_closest_index(&self.lats, &lat);
        let lon_index = self.find_closest_index(&self.lons, &lon);
        self.data[[lat_index, lon_index]]
    }
}