use std::str::FromStr;
use std::num::ParseFloatError;


/// Struct to represent a JSON query parameter for a given location
#[derive(Serialize, Deserialize, Debug)]
pub struct Points {
    pub points: Vec<(f64, f64)>
}

impl Points {
    pub fn len(&self) -> usize {
        self.points.len()
    }
}



/// Implement FromStr for CoordinateList to parse the coordinate list from the request query
impl FromStr for Points {

    type Err = ParseFloatError;

    // Take a string list of tuples ie. "(12.23,45.45),(23.3,34.5)" and form into Vec<(f64, f64)> data type
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /*
            Create a vector of (f64, f64) from a string like "(1.2,3.4),(3.4,2.9)(1.2,3.4)" which
            represents array of lat,lon coordinates.
        */
        let points: Vec<Vec<&str>> = s.split('(').filter(|v| v != &"") // Split (1.2,3.4)(1.3,2.3) into  ["1.2,3.4)", "1.3,2.3)"] less any empty strings
            .map(|v| v.trim_matches(|v|v == ')' || v == ','))    // trim matches of ")" or "," resulting in [["1.2,3.4"], ["1.3,2.3"]]
            .map(|v| v.split(',').collect())                               // map each tuple into [["1.2","3.4"], ["1.3", "2.3"]]
            .collect();

        // Go over each sub vec and for each element in the subvec, parse into f64
        // Return parse error if failed.
        let mut parsed_points: Vec<(f64, f64)> = Vec::new();
        for str_vec in &points {
            let mut parsed_vec = Vec::new();
            for s in str_vec {
                match f64::from_str(s) {
                    Ok(parsed_float) => parsed_vec.push(parsed_float),
                    Err(err) => return Err(err)
                }
            }
            parsed_points.push((parsed_vec[0], parsed_vec[1]));
        }

        Ok(Points { points: parsed_points })
    }
}
