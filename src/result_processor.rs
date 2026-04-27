//! Responsible for grading test results.
use crate::{grades, profiles::UserProfile, types::TestResult};

/// Trait so callers can plug‑in alternative rating engines.
pub trait ResultProcessor {
    fn process(&self, result: &mut TestResult, profile: UserProfile);
}

/// Default implementation matching the historic behaviour.
pub struct DefaultResultProcessor;

impl DefaultResultProcessor {
    // Convenience wrapper for legacy tests
    pub fn process(
        &self,
        result: &mut crate::types::TestResult,
        profile: crate::profiles::UserProfile,
    ) {
        ResultProcessor::process(self, result, profile);
    }
}

impl ResultProcessor for DefaultResultProcessor {
    fn process(&self, result: &mut TestResult, profile: UserProfile) {
        // Compute overall grade only when we have any measurement data.
        if result.ping.is_some()
            || result.jitter.is_some()
            || result.download.is_some()
            || result.upload.is_some()
        {
            let overall = grades::grade_overall(
                result.ping,
                result.jitter,
                result.download,
                result.upload,
                profile,
            );
            result.overall_grade = Some(overall.as_str().to_string());
        }
        result.download_grade = result.download.map(|d| {
            grades::grade_download(d / 1_000_000.0, profile)
                .as_str()
                .to_string()
        });
        result.upload_grade = result.upload.map(|u| {
            grades::grade_upload(u / 1_000_000.0, profile)
                .as_str()
                .to_string()
        });
        result.connection_rating =
            Some(crate::formatter::ratings::connection_rating(result).to_string());
    }
}
