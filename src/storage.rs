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
    path: Option<std::path::PathBuf>,
}

impl FileStorage {
    pub fn new() -> Self {
        Self { path: None }
    }

    pub fn with_path(path: std::path::PathBuf) -> Self {
        Self { path: Some(path) }
    }
}

impl Default for FileStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveResult for FileStorage {
    fn save(&self, result: &TestResult) -> Result<(), Error> {
        match &self.path {
            Some(path) => crate::history::save_result_to_path(result, path),
            None => crate::history::save_result(result),
        }
    }
}

impl LoadHistory for FileStorage {
    fn load_recent(&self, limit: usize) -> Result<Vec<TestResult>, Error> {
        let entries = match &self.path {
            Some(path) => crate::history::load_history_from_path(path)?,
            None => crate::history::load()?,
        };
        let converted: Vec<TestResult> = entries
            .into_iter()
            .rev()
            .take(limit)
            .map(|e| {
                e.report.unwrap_or_else(|| TestResult {
                    phases: crate::types::TestPhases {
                        ping: crate::types::PhaseResult::skipped("not recorded in legacy history"),
                        download: crate::types::PhaseResult::skipped(
                            "not recorded in legacy history",
                        ),
                        upload: crate::types::PhaseResult::skipped(
                            "not recorded in legacy history",
                        ),
                    },
                    timestamp: e.timestamp,
                    server: crate::types::ServerInfo {
                        id: String::new(),
                        name: e.server_name,
                        sponsor: e.sponsor,
                        country: "".to_string(),
                        distance: f64::INFINITY,
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
            })
            .collect();
        Ok(converted)
    }

    fn clear(&self) -> Result<(), Error> {
        match &self.path {
            Some(path) => crate::history::clear_history_from_path(path),
            None => crate::history::clear(),
        }
    }
}

impl ResultSink for FileStorage {
    fn write_report(&self, report: &crate::domain::reporting::Report) -> Result<(), Error> {
        // Reuse the history module which already knows how to serialize a Report.
        SaveResult::save(self, report)
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

    #[test]
    fn custom_history_paths_are_isolated_and_clear_removes_backups() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/history.json");
        let storage = FileStorage::with_path(path.clone());
        let other = FileStorage::with_path(dir.path().join("other.json"));
        SaveResult::save(&storage, &make_test_result("first")).unwrap();
        ResultSink::write_report(&storage, &make_test_result("second")).unwrap();
        assert!(path.exists());
        assert!(path.with_extension("json.bak").exists());
        assert_eq!(LoadHistory::load_recent(&storage, 1).unwrap().len(), 1);
        assert!(LoadHistory::load_recent(&other, 10).unwrap().is_empty());
        std::fs::write(path.with_extension("json.corrupt"), "corrupt").unwrap();
        LoadHistory::clear(&storage).unwrap();
        assert!(!path.with_extension("json.corrupt").exists());
        assert!(!path.exists());
        assert!(!path.with_extension("json.bak").exists());
        assert!(LoadHistory::load_recent(&storage, 10).unwrap().is_empty());
        LoadHistory::clear(&storage).unwrap();
        SaveResult::save(&storage, &make_test_result("after-clear")).unwrap();
        assert_eq!(LoadHistory::load_recent(&storage, 10).unwrap().len(), 1);
    }

    #[test]
    fn history_round_trips_complete_reports_and_reads_legacy_entries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        let storage = FileStorage::with_path(path.clone());
        let mut result = make_test_result("42");
        result.server.distance = f64::INFINITY;
        result.download_samples = Some(vec![1.0, 2.0, 3.0]);
        result.download_cv = Some(0.25);
        result.download_ci_95 = Some((1.0, 2.0));
        result.client_location = Some(crate::types::ClientLocation {
            lat: 45.0,
            lon: -79.0,
            city: Some("Toronto".into()),
            country: Some("CA".into()),
        });
        result.overall_grade = Some("A".into());
        result.test_id = Some("test-id".into());
        result.phases.upload = crate::types::PhaseResult::skipped("disabled by user");
        SaveResult::save(&storage, &result).unwrap();
        let loaded = LoadHistory::load_recent(&storage, 1).unwrap().remove(0);
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::to_value(&result).unwrap()
        );
        let mut legacy = serde_json::to_value(crate::history::Entry::from(&result)).unwrap();
        legacy.as_object_mut().unwrap().remove("report");
        legacy.as_object_mut().unwrap().remove("schema_version");
        std::fs::write(&path, serde_json::to_vec(&vec![legacy]).unwrap()).unwrap();
        let loaded = LoadHistory::load_recent(&storage, 1).unwrap().remove(0);
        assert_eq!(loaded.download, result.download);
        assert!(loaded.server.id.is_empty());
        assert!(loaded.server.distance.is_infinite());
        SaveResult::save(&storage, &result).unwrap();
        assert_eq!(LoadHistory::load_recent(&storage, 10).unwrap().len(), 2);
    }

    #[test]
    fn unsupported_history_schema_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        let storage = FileStorage::with_path(path.clone());
        let report = make_test_result("1");
        SaveResult::save(&storage, &report).unwrap();
        SaveResult::save(&storage, &report).unwrap();
        assert!(path.with_extension("json.bak").exists());
        let mut entries: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        entries[0]["schema_version"] = serde_json::json!(2);
        entries[0]["report"] = serde_json::json!("future report format");
        let original = serde_json::to_vec(&entries).unwrap();
        std::fs::write(&path, &original).unwrap();
        assert!(LoadHistory::load_recent(&storage, 1).is_err());
        assert!(SaveResult::save(&storage, &report).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }

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
        let dir = tempfile::tempdir().unwrap();
        let storage = FileStorage::with_path(dir.path().join("history.json"));
        <dyn SaveResult>::save(&storage, &make_test_result("hist")).unwrap();
        assert_eq!(
            <dyn HistoryStorage>::load_history(&storage, 1)
                .unwrap()
                .len(),
            1
        );
        <dyn HistoryStorage>::clear_history(&storage).unwrap();
        assert!(
            <dyn HistoryStorage>::load_history(&storage, 1)
                .unwrap()
                .is_empty()
        );
    }
}
