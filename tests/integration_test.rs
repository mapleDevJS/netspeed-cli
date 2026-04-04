// Integration tests for netspeed-cli

#[cfg(test)]
mod tests {
    use netspeed_cli::servers::calculate_distance;

    #[test]
    fn test_calculate_distance_same_point() {
        // Distance between identical points should be zero
        let dist = calculate_distance(40.7128, -74.0060, 40.7128, -74.0060);
        assert!(dist < 0.01, "Distance should be nearly zero, got: {}", dist);
    }

    #[test]
    fn test_calculate_distance_nyc_to_la() {
        // NYC to LA should be approximately 3944 km
        let dist = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!(
            (dist - 3944.0).abs() < 100.0,
            "Distance NYC to LA should be ~3944 km, got: {}",
            dist
        );
    }

    #[test]
    fn test_calculate_distance_london_to_paris() {
        // London to Paris should be approximately 344 km
        let dist = calculate_distance(51.5074, -0.1278, 48.8566, 2.3522);
        assert!(
            (dist - 344.0).abs() < 50.0,
            "Distance London to Paris should be ~344 km, got: {}",
            dist
        );
    }

    #[test]
    fn test_calculate_distance_sydney_to_tokyo() {
        // Sydney to Tokyo should be approximately 7823 km
        let dist = calculate_distance(-33.8688, 151.2093, 35.6762, 139.6503);
        assert!(
            (dist - 7823.0).abs() < 200.0,
            "Distance Sydney to Tokyo should be ~7823 km, got: {}",
            dist
        );
    }
}
