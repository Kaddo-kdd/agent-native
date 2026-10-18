//! `SuiteReport` + per-case results (AN-019).
//!
//! Produced by `executor::run_suite`. The report is a plain data
//! structure (no `panic!`) that the test binary or future CLI can
//! inspect, serialise to JSON, or assert against. Per OQ-04,
//! `report.failed == 0` is the convention for a passing suite.

use serde::{Deserialize, Serialize};

/// Status of a single case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    Pass,
    Fail,
    Error,
}

/// Result for a single case, with human-readable evidence lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub id: String,
    pub status: CaseStatus,
    pub evidence: Vec<String>,
}

/// Aggregate report for one suite execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteReport {
    pub suite: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub errored: usize,
    pub cases: Vec<CaseResult>,
}

impl SuiteReport {
    #[must_use]
    pub fn new(suite_name: impl Into<String>) -> Self {
        Self {
            suite: suite_name.into(),
            total: 0,
            passed: 0,
            failed: 0,
            errored: 0,
            cases: Vec::new(),
        }
    }

    pub fn push(&mut self, result: CaseResult) {
        self.total += 1;
        match result.status {
            CaseStatus::Pass => self.passed += 1,
            CaseStatus::Fail => self.failed += 1,
            CaseStatus::Error => self.errored += 1,
        }
        self.cases.push(result);
    }

    /// Convenience: `report.ok()` is true iff no case failed or errored.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.failed == 0 && self.errored == 0
    }
}
