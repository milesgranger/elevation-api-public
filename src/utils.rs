use glob;
use serde_json;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use rayon::prelude::*;


use elevation::ElevationTile;


#[derive(Serialize, Deserialize)]
pub struct ElevationTileFileMetaData {
    file: String,
    coords: [f64; 4]
}

pub fn load_summary_file() -> Vec<ElevationTileFileMetaData> {
    let file = File::open("./data/summary.json").expect("Failed to open file.");
    let data: Vec<ElevationTileFileMetaData> = serde_json::from_reader(file).unwrap();
    data
}


pub fn make_summary_file() {
    /*
        Create a summary json file which holds the meta data around each file.
        Query this file of {"file": "file/path.nc", "coords": [lat_min, lat_max, lon_min, _lon_max]}
        to see which file to load to read elevations
    */

    // Hold meta-data info in a vector
    let mut file_data: Vec<ElevationTileFileMetaData> = Vec::new();

    // Loop through all netCDF files creating meta-data items for each one.
    for entry in glob("/home/milesg/Projects/elevation-api/data/*.nc")
        .expect("Can't read glob pattern")
        {
            match entry {
                Ok(path) => {
                    println!("Path: {:?}", path.display());

                    let elevation = ElevationTile::new(&path);

                    let file = path.into_os_string().into_string().unwrap();
                    let coords = [
                            elevation.lat_min_max.0, elevation.lat_min_max.1,
                            elevation.lon_min_max.0, elevation.lon_min_max.1
                    ];

                    let meta_data = ElevationTileFileMetaData {file, coords};
                    file_data.insert(0, meta_data);
                }
                Err(e) => println!("Error locating path: {:?}", e)
            }
    }

    // Serialize list of meta-data items into JSON string and dump to file.
    let data = serde_json::to_string(&file_data).unwrap();

    let mut summary = File::create("/home/milesg/Projects/elevation-api/data/summary.json").unwrap();
    let result = summary.write_all(&data.into_bytes());
    match result {
        Ok(_r) => println!("Wrote out summary file successfully!"),
        Err(e) => println!("Failed to write summary file: {:?}", e)
    }
}


fn get_elevation(lat: &f64, lon: &f64, meta_datas: &Vec<ElevationTileFileMetaData>) -> Option<f64> {

    for resource in meta_datas {

        // Resource has coordinates holding both these lat and lon coords
        if (lat >= &resource.coords[0] && lat <= &resource.coords[1]) &&
            (lon >= &resource.coords[2] && lon <= &resource.coords[3]) {
            let tile = ElevationTile::new(Path::new(&resource.file));
            return Some(tile.get_elevation(*lat, *lon))
        }
    }
    None // No matching tiles found
}

pub fn get_elevations(coords: Vec<(&f64, &f64)>, metas: &Vec<ElevationTileFileMetaData>) -> Vec<f64> {
    /*
    let mut elevations: Vec<Option<f64>> = Vec::new();

    coords.par_iter()
        .map(|&(lat, lon)| get_elevation(lat, lon, &metas))
        .map(|result| match result {
            Some(meters) => meters,
            None => -9999.99
        })
        .collect::<Vec<f64>>()
   */

    let max_metas = 100;
    let mut open_tiles: Vec<ElevationTile> = Vec::new();
    let mut open_files: Vec<&String> = Vec::new();
    let mut elevations: Vec<f64> = Vec::new();

    for (lat, lon) in coords {
        for resource in metas {

            // Resource has coordinates holding both these lat and lon coords
            if (lat >= &resource.coords[0] && lat <= &resource.coords[1]) &&
                (lon >= &resource.coords[2] && lon <= &resource.coords[3]) {

                // Check if it's in open tiles
                if let Some(index) = open_files.iter().position(| f| *f == &resource.file) {
                    let tile = &open_tiles[index];
                    elevations.insert(0, tile.get_elevation(*lat, *lon))

                // Not in open tiles, open it.
                } else {
                    let tile = ElevationTile::new(Path::new(&resource.file));
                    elevations.insert(0, tile.get_elevation(*lat, *lon));
                    open_files.insert(0, &resource.file);
                    open_tiles.insert(0, tile);
                }
            }
        }
    }


    elevations
}