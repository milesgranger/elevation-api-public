use std::str::FromStr;
use std::str::Utf8Error;
use rocket::request;
use rocket::http;

/// Struct to represent a JSON query parameter for a given location
#[derive(FromForm, Serialize, Deserialize)]
pub struct Points {
    pub points: CoordinateList
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoordinateList(pub Vec<(f64, f64)>);

impl FromStr for CoordinateList {
    type Err = Utf8Error;

    // "?points=(45.9,34.8)(34.3,28.9)".split('(') -> ["45.9,34.8)", "34.3,28.9)"].map(
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let points: Vec<&str> = s.split("(").filter(|v| v != &"").collect::<Vec<&str>>();
        let points: Vec<&str> = points.iter().map(|v| v.trim_matches(|v|v == ')' || v == ',')).collect();
        let points: Vec<Vec<f64>> = points.iter()
            .map(|v| v.split(",")
                .map(|v| f64::from_str(v).expect("Failed to parse f64")).collect()
            )
            .collect();
        let points: Vec<(f64, f64)> = points.iter().map(|v| (v[0], v[1])).collect();
        Ok(CoordinateList(points))
    }
}


impl<'v> request::FromFormValue<'v> for CoordinateList {
    type Error = &'v http::RawStr;

    fn from_form_value(form_value: &'v http::RawStr) -> Result<CoordinateList, &'v http::RawStr> {
        match form_value.parse::<CoordinateList>() {
            Ok(points) => Ok(points),
            _ => Err(form_value)
        }
    }
}
