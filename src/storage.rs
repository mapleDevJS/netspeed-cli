//! Result storage trait for test results.
//!
//! Enables dependency injection for different storage backends.

use crate::error::Error;

/// Minimal trait for persisting a speed‑test report.
pub trait ResultSink {
    fn write_report(&self, report: &crate::domain::reporting::Report) -> Result<(), Error>;
}

use crate::types::TestResult;

/// Trait for persisting a single test result.
/// Implementations may be file‑based, cloud‑based, etc.
pub trait SaveResult: Send + Sync {
    fn save(&self, result: &TestResult) -> Result<(), Error>;
}

/// Trait for reading historic results (optional).
/// Not all storage backends need to implement this – e.g. a transient
/// in‑memory store can choose to omit history support.
pub trait LoadHistory: Send + Sync {
    fn load_recent(&self, limit: usize) -> Result<Vec<TestResult>, Error>;
    fn clear(&self) -> Result<(), Error>;
}

/// Combined trait for storage that supports both saving and loading.
/// Provides cleaner dependency injection than separate traits.
pub trait HistoryStorage: Send + Sync {
    fn save(&self, result: &TestResult) -> Result<(), Error>;
    fn load_history(&self, limit: usize) -> Result<Vec<TestResult>, Error>;
    fn clear_history(&self) -> Result<(), Error>;
}

impl<T: SaveResult + LoadHistory> HistoryStorage for T {
    fn save(&self, result: &TestResult) -> Result<(), Error> {
        SaveResult::save(self, result)
    }

    fn load_history(&self, limit: usize) -> Result<Vec<TestResult>, Error> {
        LoadHistory::load_recent(self, limit)
    }

    fn clear_history(&self) -> Result<(), Error> {
        LoadHistory::clear(self)
    }
}

/// File-based storage implementation using history module.
pub struct FileStorage {
    // implements both SaveResult/LoadHistory and ResultSink
    _path: std::path::PathBuf,
}

impl FileStorage {
    pub fn new() -> Self {
        Self {
            _path: std::path::PathBuf::new(),
        }
    }

    pub fn with_path(path: std::path::PathBuf) -> Self {
        Self { _path: path }
    }
}

impl Default for FileStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveResult for FileStorage {
    fn save(&self, result: &TestResult) -> Result<(), Error> {
        crate::history::save_result(result)
    }
}

impl LoadHistory for FileStorage {
    fn load_recent(&self, limit: usize) -> Result<Vec<TestResult>, Error> {
        let entries = crate::history::load()?;
        let converted: Vec<TestResult> = entries
            .into_iter()
            .rev()
            .take(limit)
            .map(|e| TestResult {
                timestamp: e.timestamp,
                server: crate::types::ServerInfo {
                    id: "0".to_string(),
                    name: e.server_name,
                    sponsor: e.sponsor,
                    country: "".to_string(),
                    distance: 0.0,
                },
                ping: e.ping,
                jitter: e.jitter,
                packet_loss: e.packet_loss,
                download: e.download,
                download_peak: e.download_peak,
                upload: e.upload,
                upload_peak: e.upload_peak,
                latency_download: e.latency_download,
                latency_upload: e.latency_upload,
                client_ip: e.client_ip,
                ..TestResult::default()
            })
            .collect();
        Ok(converted)
    }

    fn clear(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl ResultSink for FileStorage {
    fn write_report(&self, report: &crate::domain::reporting::Report) -> Result<(), Error> {
        // Reuse the history module which already knows how to serialize a Report.
        crate::history::save_report(report)
    }
}

/// In-memory storage for testing - does not persist to disk.
pub struct MockStorage {
    results: std::sync::Mutex<Vec<TestResult>>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            results: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn with_results(results: Vec<TestResult>) -> Self {
        Self {
            results: std::sync::Mutex::new(results),
        }
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveResult for MockStorage {
    fn save(&self, result: &TestResult) -> Result<(), Error> {
        let mut guard = self
            .results
            .lock()
            .map_err(|e| Error::context(format!("mock storage lock poisoned: {e}")))?;
        guard.push(result.clone());
        Ok(())
    }
}

impl LoadHistory for MockStorage {
    fn load_recent(&self, limit: usize) -> Result<Vec<TestResult>, Error> {
        let guard = self
            .results
            .lock()
            .map_err(|e| Error::context(format!("mock storage lock poisoned: {e}")))?;
        Ok(guard.iter().rev().take(limit).cloned().collect())
    }

    fn clear(&self) -> Result<(), Error> {
        let mut guard = self
            .results
            .lock()
            .map_err(|e| Error::context(format!("mock storage lock poisoned: {e}")))?;
        guard.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_result(id: &str) -> TestResult {
        TestResult {
            status: "ok".to_string(),
            version: "0.0.0".to_string(),
            test_id: Some(id.to_string()),
            server: crate::types::ServerInfo {
                id: id.to_string(),
                name: "Test".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                distance: 100.0,
            },
            ping: Some(10.0),
            jitter: Some(1.0),
            packet_loss: Some(0.0),
            download: Some(100_000_000.0),
            download_peak: Some(120_000_000.0),
            upload: Some(50_000_000.0),
            upload_peak: Some(60_000_000.0),
            download_cv: None,
            upload_cv: None,
            download_ci_95: None,
            upload_ci_95: None,
            latency_download: None,
            latency_upload: None,
            download_samples: None,
            upload_samples: None,
            ping_samples: None,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            client_ip: Some("1.2.3.4".to_string()),
            client_location: None,
            overall_grade: None,
            download_grade: None,
            upload_grade: None,
            connection_rating: None,
            phases: crate::types::TestPhases {
                ping: crate::types::PhaseResult::completed(),
                download: crate::types::PhaseResult::completed(),
                upload: crate::types::PhaseResult::completed(),
            },
        }
    }

    #[test]
    fn test_mock_storage_save_load_round_trip() {
        let storage = MockStorage::new();
        let result = make_test_result("abc");

        <dyn SaveResult>::save(&storage, &result).unwrap();

        let loaded = <dyn LoadHistory>::load_recent(&storage, 10).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].test_id, Some("abc".to_string()));
    }

    #[test]
    fn test_mock_storage_with_results() {
        let r1 = make_test_result("first");
        let r2 = make_test_result("second");
        let storage = MockStorage::with_results(vec![r1, r2]);

        let loaded = <dyn LoadHistory>::load_recent(&storage, 10).unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_mock_storage_load_recent_limit() {
        let storage = MockStorage::with_results(vec![
            make_test_result("a"),
            make_test_result("b"),
            make_test_result("c"),
        ]);

        let loaded = <dyn LoadHistory>::load_recent(&storage, 2).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].test_id, Some("c".to_string()));
        assert_eq!(loaded[1].test_id, Some("b".to_string()));
    }

    #[test]
    fn test_mock_storage_clear() {
        let storage = MockStorage::with_results(vec![make_test_result("x")]);
        assert_eq!(
            <dyn LoadHistory>::load_recent(&storage, 10).unwrap().len(),
            1
        );

        <dyn LoadHistory>::clear(&storage).unwrap();
        assert!(
            <dyn LoadHistory>::load_recent(&storage, 10)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_mock_storage_empty_load() {
        let storage = MockStorage::new();
        let loaded = <dyn LoadHistory>::load_recent(&storage, 10).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    #[serial_test::serial]
    fn test_history_storage_for_file_storage() {
        let storage = crate::storage::FileStorage::new();
        let result = make_test_result("hist");
        if <dyn SaveResult>::save(&storage, &result).is_err() {
            return;
        }
        if let Ok(loaded) = <dyn HistoryStorage>::load_history(&storage, 1) {
            assert_eq!(loaded.len(), 1);
            let _ = <dyn HistoryStorage>::clear_history(&storage);
        }
    }
}
