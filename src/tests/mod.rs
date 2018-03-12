use std::path::Path;
use ndarray;
use self::super::elevation;

#[test]
fn test_get_10k() {

    // Make summary file if it doesn't exist
    if !Path::new("./data/summary.json").exists() {
        elevation::make_summary_file();
    }

    // Load meta data about elevation files
    let metas = elevation::load_summary_file();

    // Build collection of testing coordinates
    let lats = ndarray::Array::linspace(45.08904, 55.08904, 100);
    let lons = ndarray::Array::linspace(80.85938, 85.85938, 100);
    let mut coords = Vec::with_capacity(lats.len() * lons.len() as usize);
    for lat in lats.iter() {
        for lon in lons.iter() {
            coords.insert(0, (lat, lon));
        }
    }
    let coords: Vec<(&f64, &f64)> = coords.to_vec();

    // Get elevations
    println!("Getting coords for {} locations", coords.len());
    let _elevations = elevation::get_elevations(coords, &metas);
    println!("Finished!");
}
