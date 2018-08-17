use std::env;
use std::path::Path;
use std::fs;
use std::str::FromStr;
use self::super::json_structs::{Points};
use self::super::elevation;


#[test]
fn test_points() {
    /*
        Parsing of well formed tuple list of coordinates
    */

    // Parse a query into a points object
    let points = Points::from_str("(48.35,5.3),(48.43,5.23)").unwrap();
    assert_eq!(points.points[0].0, 48.35);
    assert_eq!(points.points[0].1, 5.3);
    assert_eq!(points.points[1].0, 48.43);
    assert_eq!(points.points[1].1, 5.23);
}

#[test]
fn test_points_parse_failure() {
    /*
        Parsing a bad formed tuple of coordinates should panic
    */
    let points = Points::from_str("(48.2,54.2a");
    match points {
        Ok(pts) => panic!("Got coordinates from a bad tuple: {:?}", pts),
        Err(_) => println!("Failed to parse points as expected!")
    }
}

#[test]
fn test_points_len() {
    let points = Points::from_str("(45.2,34.2),(32.4,-12.0)").unwrap();
    assert_eq!(points.len(), 2_usize);
}

#[test]
fn test_make_summary_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let data_dir = root.join("src/tests/data/");
    let summary_file = data_dir.join("summary.json");
    if summary_file.exists(){
        fs::remove_file(&summary_file).expect("Unable to remove summary file!");
    }
    elevation::make_summary_file(&data_dir.to_str().unwrap());
    assert_eq!(summary_file.exists(), true);
}

#[test]
fn test_load_summary_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let data_dir = root.join("src/tests/data/");
    elevation::make_summary_file(&data_dir.to_str().unwrap());
    env::set_var("DATA_DIR", &data_dir);
    let metas = elevation::load_summary_file();
    println!("{:?}", &metas);

    // Ensure the file loaded matches the full path to the one in the test/data dir
    assert_eq!(&metas[0].file, &data_dir.join("n58e104.nc").into_os_string().into_string().unwrap());

    // Ensure the latitude bounds are correct
    assert_eq!(&metas[0].coords[0], &58.0);
    assert_eq!(&metas[0].coords[2], &104.0);

    // Ensure the longitudes bounds are correct
    assert_eq!(&metas[0].coords[1], &59.0);
    assert_eq!(&metas[0].coords[3], &105.0);
}

#[test]
fn test_get_elevation() {
    
    // Make summary and get metas
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let data_dir = root.join("src/tests/data/");
    elevation::make_summary_file(&data_dir.to_str().unwrap());
    env::set_var("DATA_DIR", &data_dir);
    let metas = elevation::load_summary_file();

    // Make a point
    let points = Points::from_str("(58.5,104.5)").unwrap();

    // Fetch elevation
    let elevations = elevation::get_elevations(points.points, &metas);

    assert_eq!(elevations[0].elevation, 457.0);

}