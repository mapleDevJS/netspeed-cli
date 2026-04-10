//! Geographic distance calculations using the Haversine formula.
//!
//! Pure math — no I/O, no dependencies beyond std.

/// Calculate distance between two geographic points using the Haversine formula.
///
/// Returns distance in kilometers.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::geo::calculate_distance;
/// let dist = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
/// assert!((dist - 3944.0).abs() < 200.0); // ~3944 km, NYC to LA
/// ```
#[must_use]
pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_distance_same_location() {
        let dist = calculate_distance(40.7128, -74.0060, 40.7128, -74.0060);
        assert!(dist < 0.01);
    }

    #[test]
    fn test_calculate_distance_nyc_la() {
        let dist = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!((dist - 3944.0).abs() < 200.0);
    }

    #[test]
    fn test_calculate_distance_nyc_london() {
        let dist = calculate_distance(40.7128, -74.0060, 51.5074, -0.1278);
        assert!((dist - 5570.0).abs() < 300.0);
    }

    #[test]
    fn test_calculate_distance_sydney_tokyo() {
        let dist = calculate_distance(-33.8688, 151.2093, 35.6762, 139.6503);
        assert!((dist - 7823.0).abs() < 300.0);
    }

    #[test]
    fn test_calculate_distance_opposite_sides() {
        let dist = calculate_distance(40.7128, -74.0060, -33.8688, 151.2093);
        assert!(dist > 15_000.0);
    }

    #[test]
    fn test_calculate_distance_equator() {
        let dist = calculate_distance(0.0, 0.0, 0.0, 10.0);
        assert!((dist - 1111.0).abs() < 100.0);
    }
}
