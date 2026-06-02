//! # Gas Guardian
//!
//! Gas budget enforcement and conservation analysis for Foundry deployments.
//!
//! ## What it catches
//!
//! - **Gas regressions** between commits or deployment runs
//! - **Budget overruns** — a single contract consuming >80% of total gas budget
//! - **Inefficient patterns** — mapping-to-scan regressions (O(1) → O(n))
//!
//! ## Example
//!
//! Commit a3f2: `transferFrom()` costs **45,000 gas**.
//! Commit b7c1: same function costs **540,000 gas** — a 12× regression.
//! Cause: a "small optimization" replaced a `mapping` lookup with an array scan.
//! Gas Guardian caught it.

#![deny(missing_docs)]

use std::collections::BTreeMap;

/// Threshold ratio (0.0–1.0) that flags a single contract as over-consuming
/// its share of the total deployment gas budget.
pub const BUDGET_EATEN_THRESHOLD: f64 = 0.8;

/// A gas measurement for a single function.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GasMeasurement {
    /// Human-readable label, e.g. "transferFrom" or "deploy"
    pub label: String,
    /// Gas units consumed
    pub gas_used: u64,
}

/// A complete snapshot of gas usage for one deployment or commit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GasSnapshot {
    /// Commit hash or deployment identifier
    pub id: String,
    /// Per-function gas measurements
    pub measurements: Vec<GasMeasurement>,
    /// Optional total gas budget for the whole deployment
    pub total_budget: Option<u64>,
}

impl GasSnapshot {
    /// Total gas across all measurements.
    pub fn total_gas(&self) -> u64 {
        self.measurements.iter().map(|m| m.gas_used).sum()
    }

    /// Find the single function consuming the most gas.
    pub fn hottest_function(&self) -> Option<&GasMeasurement> {
        self.measurements.iter().max_by_key(|m| m.gas_used)
    }

    /// The consumption ratio (0.0–1.0) of the hottest function relative to
    /// the total budget (if a budget is set). Returns `None` when no budget
    /// is set or no measurements exist.
    pub fn budget_eaten_ratio(&self) -> Option<f64> {
        let budget = self.total_budget?;
        let hottest = self.measurements.iter().max_by_key(|m| m.gas_used)?;
        if budget == 0 {
            return None;
        }
        Some(hottest.gas_used as f64 / budget as f64)
    }

    /// Returns `true` when a single contract has eaten more than
    /// [`BUDGET_EATEN_THRESHOLD`] of the total gas budget.
    pub fn budget_overrun(&self) -> bool {
        self.budget_eaten_ratio()
            .map_or(false, |r| r > BUDGET_EATEN_THRESHOLD)
    }
}

/// A comparison between two gas snapshots (e.g. two commits).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GasRegression {
    /// Human-readable description of the regression
    pub description: String,
    /// Label of the regressed function
    pub function: String,
    /// Gas cost before
    pub before: u64,
    /// Gas cost after
    pub after: u64,
    /// Multiplier: after / before
    pub factor: f64,
}

impl GasRegression {
    /// Create a regression entry from before/after measurements.
    pub fn new(description: impl Into<String>, function: impl Into<String>, before: u64, after: u64) -> Self {
        let factor = if before == 0 {
            f64::INFINITY
        } else {
            after as f64 / before as f64
        };
        Self {
            description: description.into(),
            function: function.into(),
            before,
            after,
            factor,
        }
    }
}

/// Compare two snapshots and emit regressions.
pub fn find_regressions(before: &GasSnapshot, after: &GasSnapshot) -> Vec<GasRegression> {
    let mut regressions = Vec::new();

    // Build a lookup from label → gas_used for the "before" snapshot.
    let before_map: BTreeMap<&str, u64> = before
        .measurements
        .iter()
        .map(|m| (m.label.as_str(), m.gas_used))
        .collect();

    for measurement in &after.measurements {
        if let Some(&before_gas) = before_map.get(measurement.label.as_str()) {
            if measurement.gas_used > before_gas {
                regressions.push(GasRegression::new(
                    format!(
                        "Gas regression in {}: {} → {} ({}× increase)",
                        measurement.label,
                        before_gas,
                        measurement.gas_used,
                        measurement.gas_used as f64 / before_gas as f64
                    ),
                    measurement.label.clone(),
                    before_gas,
                    measurement.gas_used,
                ));
            }
        }
    }

    regressions
}

/// Analyze a single snapshot for budget violations and hot spots.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConservationReport {
    /// Snapshot identifier (commit hash or deployment name).
    pub snapshot_id: String,
    /// Sum of all measured gas costs.
    pub total_gas: u64,
    /// Optional total gas budget for the deployment.
    pub budget: Option<u64>,
    /// Percentage of budget consumed by the single hottest function (0.0–100.0).
    pub budget_eaten_pct: Option<f64>,
    /// Whether a single contract exceeded 80% of the budget.
    pub overrun: bool,
    /// Label of the function with the highest gas cost.
    pub hottest_function: Option<String>,
    /// Number of regressions found compared to a baseline.
    pub regression_count: usize,
}

impl ConservationReport {
    /// Produce a conservation analysis from a snapshot and optional regressions.
    pub fn from_snapshot(snapshot: &GasSnapshot, regression_count: usize) -> Self {
        Self {
            snapshot_id: snapshot.id.clone(),
            total_gas: snapshot.total_gas(),
            budget: snapshot.total_budget,
            budget_eaten_pct: snapshot.budget_eaten_ratio().map(|r| r * 100.0),
            overrun: snapshot.budget_overrun(),
            hottest_function: snapshot.hottest_function().map(|m| m.label.clone()),
            regression_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_regression_when_costs_drop() {
        let before = GasSnapshot {
            id: "a3f2".into(),
            measurements: vec![GasMeasurement {
                label: "transferFrom".into(),
                gas_used: 45_000,
            }],
            total_budget: None,
        };
        let after = GasSnapshot {
            id: "b7c1".into(),
            measurements: vec![GasMeasurement {
                label: "transferFrom".into(),
                gas_used: 30_000,
            }],
            total_budget: None,
        };
        let regressions = find_regressions(&before, &after);
        assert!(regressions.is_empty());
    }

    #[test]
    fn test_catches_twelve_x_regression() {
        let before = GasSnapshot {
            id: "a3f2".into(),
            measurements: vec![GasMeasurement {
                label: "transferFrom".into(),
                gas_used: 45_000,
            }],
            total_budget: None,
        };
        let after = GasSnapshot {
            id: "b7c1".into(),
            measurements: vec![GasMeasurement {
                label: "transferFrom".into(),
                gas_used: 540_000,
            }],
            total_budget: None,
        };
        let regressions = find_regressions(&before, &after);
        assert_eq!(regressions.len(), 1);
        assert!((regressions[0].factor - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_budget_overrun_flag() {
        let snapshot = GasSnapshot {
            id: "test-deploy".into(),
            measurements: vec![GasMeasurement {
                label: "deploy".into(),
                gas_used: 900_000,
            }],
            total_budget: Some(1_000_000),
        };
        assert!(snapshot.budget_overrun());
        let ratio = snapshot.budget_eaten_ratio().unwrap();
        assert!((ratio - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_under_budget_is_fine() {
        let snapshot = GasSnapshot {
            id: "test-deploy".into(),
            measurements: vec![
                GasMeasurement { label: "deploy".into(), gas_used: 300_000 },
                GasMeasurement { label: "transfer".into(), gas_used: 200_000 },
            ],
            total_budget: Some(1_000_000),
        };
        assert!(!snapshot.budget_overrun());
    }
}
