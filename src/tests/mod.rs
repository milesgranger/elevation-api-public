use std::str::FromStr;
use self::super::json_structs::{Points};


#[test]
fn test_points() {
    /*
        Guarantee the following:

        Request arg: '(48.35,5.3),(48.35,5.23)'
        Response: {
            "points": [
                {"lat": 48.35, "lon": 5.3, "elevation": 352.0},
                {"lat": 48.35, "lon": 5.23, "elevation": 314.0}
            ]
          }
    */

    // Parse a query into a points object
    let points = Points::from_str("(48.35,5.3),(48.43,5.23)").unwrap();
    assert_eq!(points.points[0].0, 48.35);
    assert_eq!(points.points[0].1, 5.3);
    assert_eq!(points.points[1].0, 48.43);
    assert_eq!(points.points[1].1, 5.23);
}
