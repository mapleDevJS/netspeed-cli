// Unit tests for server selection logic

#[cfg(test)]
mod tests {
    use netspeed_cli::servers::{select_best_server, calculate_distance, calculate_distances};
    use netspeed_cli::types::Server;
    use netspeed_cli::error::SpeedtestError;

    fn create_test_server(id: &str, lat: f64, lon: f64) -> Server {
        Server {
            id: id.to_string(),
            url: format!("http://server{}.test.com/", id),
            name: format!("Server {}", id),
            sponsor: format!("ISP {}", id),
            country: "US".to_string(),
            lat,
            lon,
            distance: 0.0,
            latency: 0.0,
        }
    }

    #[test]
    fn test_select_best_server_empty() {
        let servers: Vec<Server> = vec![];
        let result = select_best_server(&servers);
        assert!(result.is_err());
        match result {
            Err(SpeedtestError::ServerNotFound(_)) => {},
            _ => panic!("Expected ServerNotFound error"),
        }
    }

    #[test]
    fn test_select_best_server_single() {
        let servers = vec![create_test_server("1", 40.7128, -74.0060)];
        let result = select_best_server(&servers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "1");
    }

    #[test]
    fn test_select_best_server_closest() {
        let mut servers = vec![
            create_test_server("1", 40.7128, -74.0060), // NYC
            create_test_server("2", 34.0522, -118.2437), // LA
            create_test_server("3", 41.8781, -87.6298),  // Chicago
        ];

        // Calculate distances from a point near NYC
        calculate_distances(&mut servers, 40.8, -74.1);

        let best = select_best_server(&servers).unwrap();
        // Server 1 (NYC) should be closest
        assert_eq!(best.id, "1");
    }

    #[test]
    fn test_calculate_distances_sorting() {
        let mut servers = vec![
            create_test_server("1", 34.0522, -118.2437), // LA (far from NYC)
            create_test_server("2", 40.7128, -74.0060),  // NYC (close to NYC)
            create_test_server("3", 41.8781, -87.6298),  // Chicago (medium from NYC)
        ];

        calculate_distances(&mut servers, 40.7128, -74.0060); // From NYC

        // Servers should be sorted by distance, NYC server first
        assert_eq!(servers[0].id, "2"); // NYC
        assert!(servers[0].distance < servers[1].distance);
        assert!(servers[1].distance < servers[2].distance);
    }

    #[test]
    fn test_distance_calculation_accuracy() {
        // Test that distances are reasonable
        let nyc_to_la = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!(nyc_to_la > 3500.0 && nyc_to_la < 4500.0);

        let london_to_paris = calculate_distance(51.5074, -0.1278, 48.8566, 2.3522);
        assert!(london_to_paris > 300.0 && london_to_paris < 400.0);
    }
}
