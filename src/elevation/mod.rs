use std::env;
use std::f64;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::collections::HashMap;

use netcdf;
use ndarray::ArrayD;
use glob;
use serde_json;


/// Trait to identify min and max of ndarray array struct
pub trait MinMax {
    fn min_value(&self) -> f64;
    fn max_value(&self) -> f64;
}


/// Implement MinMax for ArrayD<64>
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


/// Tile to contain information on loaded NetCDF file
pub struct ElevationTile {
    pub data: ArrayD<f64>,
    pub lats: ArrayD<f64>,
    pub lons: ArrayD<f64>,
    pub lat_min_max: (f64, f64),
    pub lon_min_max: (f64, f64)
}


/// Implement ElevationTile
impl ElevationTile {

    fn new(path: &Path) -> ElevationTile {
        // Create a new elevation resource

        let file = netcdf::open(path.to_str()
                    .expect("Can't convert to str!"))
                    .unwrap_or_else(|_| panic!("Unable to open file for NetCDF format! {:?}", path.to_str()));

        info!("Variables: {:?}", &file.root.variables.keys());

        let lat_key = if file.root.variables.contains_key("y") { "y" } else { "lat" };
        let lon_key = if file.root.variables.contains_key("x") { "x" } else { "lon" };

        let lats: ArrayD<f64> = file.root.variables
            .get(lat_key)
            .unwrap_or_else(|| panic!("No variable called {}", lat_key))
            .as_array()
            .unwrap();
        let lons: ArrayD<f64> = file.root.variables
            .get(lon_key)
            .unwrap_or_else(|| panic!("No variable called {}", lon_key))
            .as_array()
            .unwrap();

        let data = file.root.variables["Band1"].as_array().unwrap();

        let lat_min_max = (lats.min_value(), lats.max_value());
        let lon_min_max = (lons.min_value(), lons.max_value());

        ElevationTile {
            data,
            lats,
            lons,
            lat_min_max,
            lon_min_max
        }
    }

    fn find_closest_index(&self, array: &ArrayD<f64>, f: f64) -> usize {
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
        let lat_index = self.find_closest_index(&self.lats, lat);
        let lon_index = self.find_closest_index(&self.lons, lon);
        self.data[[lat_index, lon_index]]
    }
}


/// Struct to represent a specific NetCDF file containing meta data about what coordinates
/// it holds information for and the file location.
#[derive(Serialize, Deserialize, Debug)]
pub struct ElevationTileFileMetaData {
    pub file: String,       //
    pub coords: [f64; 4]  // [min_lat, max_lat, min_lon, max_lon]
}


/// Struct to represent a geographic point and its associated elevation.
#[derive(Serialize, Deserialize, Debug)]
pub struct Elevation {
    pub lat: f64,
    pub lon: f64,
    pub elevation: f64
}

#[derive(Serialize, Debug)]
pub struct Elevations {
    pub elevations: Vec<Elevation>
}

/// Load a created summary.json file; holds information about what coordiantes belong to which file
pub fn load_summary_file() -> Vec<ElevationTileFileMetaData> {
    let source_dir = env::var("DATA_DIR")
        .unwrap_or_else(|_| {
            panic!("Need to set DATA_DIR env variable!")
        });

    let source_dir: PathBuf = source_dir.into();

    let file = File::open(source_dir.join("summary.json")).expect("Failed to open file.");
    let data: Vec<ElevationTileFileMetaData> = serde_json::from_reader(file).unwrap();

    // All files in summary.json are simply the filenames, modify the loaded meta data 'file' to point to the full path. 
    // this is given our DATA_DIR env variable
    let data: Vec<ElevationTileFileMetaData> = data.into_iter()
        .map(|v| {
            ElevationTileFileMetaData { 
                file: source_dir.join(v.file).into_os_string().into_string().unwrap(),
                coords: v.coords
             }
        })
        .collect();

    data
}

/// Create summary.json file; holds information about what coordinates belong to which file
#[allow(dead_code)]
pub fn make_summary_file(source_path: &str) {
    /*
        Create a summary json file which holds the meta data around each file.
        Query this file of {"file": "file/path.nc", "coords": [lat_min, lat_max, lon_min, _lon_max]}
        to see which file to load to read elevations
    */

    // Hold meta-data info in a vector
    let mut file_data: Vec<ElevationTileFileMetaData> = Vec::new();

    // Loop through all netCDF files creating meta-data items for each one.
    for entry in glob(&format!("{}/*.nc", &source_path))
        .expect("Can't read glob pattern")
        {
            match entry {
                Ok(path) => {
                    println!("Path: {:?}", &path.display());

                    let elevation = ElevationTile::new(&path);
                    let coords = [
                            elevation.lat_min_max.0, elevation.lat_min_max.1,
                            elevation.lon_min_max.0, elevation.lon_min_max.1
                    ];

                    let meta_data = ElevationTileFileMetaData {
                        file: path.file_name().expect("Unable to take filename!").to_os_string().into_string().unwrap(), 
                        coords
                        };
                    file_data.insert(0, meta_data);
                }
                Err(e) => println!("Error locating path: {:?}", e)
            }
    }

    // Serialize list of meta-data items into JSON string and dump to file.
    let data = serde_json::to_string(&file_data)
        .expect("Unable to serialize vector of meta data");

    let mut summary = File::create(&format!("{}/summary.json", &source_path))
        .expect("Unable to write data to summary file!");
    let result = summary.write_all(&data.into_bytes());
    match result {
        Ok(_r) => println!("Wrote out summary file successfully!"),
        Err(e) => println!("Failed to write summary file: {:?}", e)
    }
}

/// Function to grab elevations for a list of coordinates
pub fn get_elevations(coords: &[(f64, f64)], metas: &[ElevationTileFileMetaData]) -> Vec<Elevation> {
    /*
        Fetch elevations for the given coordinates.
    */
    let mut tiles: HashMap<&String, ElevationTile> = HashMap::new();
    let mut elevations: Vec<Elevation> = Vec::new();

    for (lat, lon) in coords {
        let mut found: bool = false;
        for resource in metas.iter() {
            // Resource has coordinates holding both these lat and lon coords
            if (lat >= &resource.coords[0] && lat <= &resource.coords[1]) &&
                (lon >= &resource.coords[2] && lon <= &resource.coords[3]) {

                let mut elevation;

                // ElevationTile has been opened and resides in the tiles HashMap...
                if tiles.contains_key(&resource.file) {
                    let tile = &tiles[&resource.file];
                    elevation = tile.get_elevation(*lat, *lon);

                // Create ElevationTile
                } else {
                    let tile = ElevationTile::new(Path::new(&resource.file));
                    elevation = tile.get_elevation(*lat, *lon);
                    tiles.insert(&resource.file, tile);
                }

                // Create an elevation and insert it into the result vector.
                let result = Elevation {lat: *lat, lon: *lon, elevation};
                elevations.push(result);
                found = true;

                // Elevation added, no need to continue trying other resources
                break;
            }
        }
        if !found {
            // Elevation wasn't found, probably an ocean location.
            warn!("Could not find elevation for points ({}, {})", &lat, &lon);
            let result = Elevation {lat: *lat, lon: *lon, elevation: -9999_f64};
            elevations.push(result);
        }
    }
    elevations
}
