use std::str::FromStr;
use self::super::json_structs::{CoordinateList, Points};


#[test]
fn test_v1_0_0_query() {
    /*
        Guarantee the following:

        Request: /api/v1.0.0/90m?points=(48.35,5.3),(48.35,5.23)
        Response: {
            "points": [
                {"lat": 48.35, "lon": 5.3, "elevation": 352.0},
                {"lat": 48.35, "lon": 5.23, "elevation": 314.0}
            ]
          }
    */
    use self::super::get_elevations_v1_0_0;
    use self::super::elevation::{Elevations, Elevation};

    // Parse a query into a points object
    let points = Points {
        points: CoordinateList::from_str("(48.35,5.3),(48.35,5.23)").unwrap()
    };

    // Get elevations from the function
    let elevations: Elevations = get_elevations_v1_0_0(Some(points)).ok().unwrap().0;

    // Ensure that the first element == 48.35 which guarantees the version promised format.
    let point: &Elevation = &elevations.points[0];
    assert_eq!(point.lat, 48.35);
    assert_eq!(point.lon, 5.3)
}
