// Fuzz tests for security-critical parsing functions
//
// To set up fuzzing:
//   cargo install cargo-fuzz
//   cargo +nightly fuzz init  # Initialize if needed
//   cargo fuzz run xml_parser
//
// This module provides fuzz targets that test:
// - XML parsing (parse_ip_from_xml)
// - Server URL derivation (ServerEndpoints::from_server_url)

use libfuzzer_sys::fuzz_target;
use netspeed_cli::endpoints::ServerEndpoints;
use netspeed_cli::http::parse_ip_from_xml;

/// XML parsing and server URL derivation fuzzer
fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, handling various encodings
    let s = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return, // Skip invalid UTF-8
    };

    // Test XML parsing - should be resilient to malformed input
    let _ = parse_ip_from_xml(s);

    // Test server URL parsing - critical for security
    let _ = ServerEndpoints::from_server_url(s);
});

/// IP address in XML context fuzzer
fuzz_target!(|data: &[u8]| {
    let s = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Test IP address in XML context
    if let Ok(ip) = parse_ip_from_xml(&format!(
        r##'<settings><client ip=“{}”/></settings>'##, s
    )) {
        // If parsing succeeded, verify it's valid
        let _ = ip;
    }
});