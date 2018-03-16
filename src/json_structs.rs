use std::str::FromStr;
use std::num::ParseFloatError;
use rocket::request;
use rocket::http;


/// Struct to represent a JSON query parameter for a given location
#[derive(FromForm, Serialize, Deserialize)]
pub struct Points {
    pub points: CoordinateList
}


/// Struct to represent the core value of the Points struct
/// list of tuples representing (lat, lon) values
#[derive(Debug, Serialize, Deserialize)]
pub struct CoordinateList(pub Vec<(f64, f64)>);


/// Implement FromStr for CoordinateList to parse the coordinate list from the request query
impl FromStr for CoordinateList {

    type Err = ParseFloatError;

    // Take a string list of tuples ie. "(12.23,45.45),(23.3,34.5)" and form into Vec<(f64, f64)> data type
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let points: Vec<&str> = s.split("(").filter(|v| v != &"").collect::<Vec<&str>>();
        let points: Vec<&str> = points.iter().map(|v| v.trim_matches(|v|v == ')' || v == ',')).collect();
        let points: Vec<Vec<&str>> = points.iter().map(|v| v.split(",").collect()).collect();

        let mut parsed_points: Vec<Vec<f64>> = Vec::new();
        for str_vec in points.iter() {
            let mut parsed_vec = Vec::new();
            for s in str_vec {
                match f64::from_str(s) {
                    Ok(parsed_float) => parsed_vec.push(parsed_float),
                    Err(err) => return Err(err)
                }
            }
            parsed_points.push(parsed_vec);
        }
        let points: Vec<(f64, f64)> = parsed_points.iter().map(|v| (v[0], v[1])).collect();
        Ok(CoordinateList(points))
    }
}


/// Implement for Rocket to parse the value from the request, which will implicitly invoke the
/// FromStr impl above.
impl<'v> request::FromFormValue<'v> for CoordinateList {
    type Error = &'v http::RawStr;

    fn from_form_value(form_value: &'v http::RawStr) -> Result<CoordinateList, &'v http::RawStr> {
        match form_value.parse::<CoordinateList>() {
            Ok(points) => Ok(points),
            _ => Err(form_value)
        }
    }
}

