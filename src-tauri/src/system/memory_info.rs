//! System Memory Detection and PII Mode Recommendation
//!
//! Provides memory detection, usage tracking, and intelligent recommendations
//! for PII detection modes based on available system resources.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total system RAM in bytes
    pub total_memory: u64,
    /// Available RAM in bytes
    pub available_memory: u64,
    /// Used RAM in bytes
    pub used_memory: u64,
    /// Current process RAM usage in bytes
    pub process_memory: u64,
    /// Total RAM in GB (for display)
    pub total_gb: f64,
    /// Available RAM in GB (for display)
    pub available_gb: f64,
    /// Memory usage percentage
    pub usage_percentage: f64,
    /// Recommended PII mode based on available memory
    pub recommended_mode: String,
    /// Warning message if memory is low
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PIIMode {
    /// Built-in regex only (0MB overhead, 85% accuracy)
    Builtin,
    /// Presidio with spaCy only (~500MB overhead, 90% accuracy)
    PresidioLite,
    /// Presidio with full ML models (~2GB overhead, 95% accuracy)
    PresidioFull,
}

impl PIIMode {
    /// Get memory overhead in bytes for this mode
    pub fn memory_overhead_bytes(&self) -> u64 {
        match self {
            PIIMode::Builtin => 0,
            PIIMode::PresidioLite => 500 * 1024 * 1024,      // 500MB
            PIIMode::PresidioFull => 2 * 1024 * 1024 * 1024, // 2GB
        }
    }

    /// Get memory overhead in MB for display
    pub fn memory_overhead_mb(&self) -> u64 {
        self.memory_overhead_bytes() / (1024 * 1024)
    }

    /// Get accuracy percentage
    pub fn accuracy(&self) -> u8 {
        match self {
            PIIMode::Builtin => 85,
            PIIMode::PresidioLite => 90,
            PIIMode::PresidioFull => 95,
        }
    }

    /// Get speed rating
    pub fn speed(&self) -> &str {
        match self {
            PIIMode::Builtin => "Fast",
            PIIMode::PresidioLite => "Medium",
            PIIMode::PresidioFull => "Slow",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            PIIMode::Builtin => "Built-in Only",
            PIIMode::PresidioLite => "Presidio Lite",
            PIIMode::PresidioFull => "Presidio Full",
        }
    }

    /// Get description
    pub fn description(&self) -> &str {
        match self {
            PIIMode::Builtin => "Fast regex-based detection with 0MB overhead. Recommended for laptops.",
            PIIMode::PresidioLite => "Enhanced detection with spaCy NER. Moderate memory usage.",
            PIIMode::PresidioFull => "State-of-the-art ML detection with DeBERTa. High memory usage.",
        }
    }

    /// Convert to string for serialization
    pub fn to_string(&self) -> String {
        match self {
            PIIMode::Builtin => "builtin".to_string(),
            PIIMode::PresidioLite => "presidio_lite".to_string(),
            PIIMode::PresidioFull => "presidio_full".to_string(),
        }
    }

    /// Parse from string
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "builtin" => PIIMode::Builtin,
            "presidio_lite" => PIIMode::PresidioLite,
            "presidio_full" => PIIMode::PresidioFull,
            _ => PIIMode::Builtin, // Default to safest option
        }
    }
}

pub struct MemoryDetector {
    system: System,
}

impl MemoryDetector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }

    /// Get current memory information
    pub fn get_memory_info(&mut self) -> Result<MemoryInfo> {
        self.system.refresh_memory();

        let total_memory = self.system.total_memory();
        let available_memory = self.system.available_memory();
        let used_memory = self.system.used_memory();

        // Get current process memory
        let process_memory = self.get_process_memory();

        let total_gb = total_memory as f64 / (1024.0 * 1024.0 * 1024.0);
        let available_gb = available_memory as f64 / (1024.0 * 1024.0 * 1024.0);
        let usage_percentage = (used_memory as f64 / total_memory as f64) * 100.0;

        // Recommend mode based on available memory
        let (recommended_mode, warning) = self.recommend_mode(available_gb, total_gb);

        Ok(MemoryInfo {
            total_memory,
            available_memory,
            used_memory,
            process_memory,
            total_gb,
            available_gb,
            usage_percentage,
            recommended_mode,
            warning,
        })
    }

    /// Get current process memory usage
    fn get_process_memory(&mut self) -> u64 {
        use sysinfo::ProcessesToUpdate;

        self.system.refresh_processes(ProcessesToUpdate::All, true);

        if let Some(process) = self.system.process(sysinfo::get_current_pid().ok().unwrap()) {
            process.memory()
        } else {
            0
        }
    }

    /// Recommend PII mode based on available memory
    pub fn recommend_mode(&self, available_gb: f64, total_gb: f64) -> (String, Option<String>) {
        // Estimate LLM memory usage (4-7GB typical for local models)
        let estimated_llm_usage = 5.5; // GB
        let safe_buffer = 2.0; // GB - keep some headroom

        let available_for_pii = available_gb - estimated_llm_usage - safe_buffer;

        let mode = if total_gb < 8.0 {
            // Low memory systems: Built-in only
            (
                PIIMode::Builtin.to_string(),
                Some(format!(
                    "System has {:.1}GB total RAM. Built-in detection recommended to preserve memory for your LLM.",
                    total_gb
                ))
            )
        } else if total_gb < 16.0 {
            // Medium memory: Lite mode if enough available
            if available_for_pii >= 0.5 {
                (
                    PIIMode::PresidioLite.to_string(),
                    Some(format!(
                        "System has {:.1}GB total RAM. Presidio Lite available if you need better accuracy.",
                        total_gb
                    ))
                )
            } else {
                (
                    PIIMode::Builtin.to_string(),
                    Some(format!(
                        "Only {:.1}GB available. Built-in detection recommended to avoid memory pressure.",
                        available_gb
                    ))
                )
            }
        } else {
            // High memory: Full mode available
            if available_for_pii >= 2.0 {
                (
                    PIIMode::PresidioFull.to_string(),
                    None
                )
            } else if available_for_pii >= 0.5 {
                (
                    PIIMode::PresidioLite.to_string(),
                    Some(format!(
                        "Only {:.1}GB available for PII detection. Lite mode recommended over Full.",
                        available_for_pii
                    ))
                )
            } else {
                (
                    PIIMode::Builtin.to_string(),
                    Some(format!(
                        "High memory usage detected. Built-in detection recommended.",
                    ))
                )
            }
        };

        mode
    }

    /// Check if a specific PII mode is safe to use
    pub fn can_use_mode(&mut self, mode: &PIIMode) -> Result<(bool, Option<String>)> {
        let info = self.get_memory_info()?;
        let required_mb = mode.memory_overhead_mb();
        let available_mb = (info.available_memory / (1024 * 1024)) as u64;

        // Keep 2GB buffer for system stability
        let safe_buffer_mb = 2 * 1024;
        let available_after_mode = available_mb.saturating_sub(required_mb);

        if available_after_mode < safe_buffer_mb {
            Ok((
                false,
                Some(format!(
                    "{} requires {}MB but only {}MB available. This may cause system instability.",
                    mode.display_name(),
                    required_mb,
                    available_mb
                ))
            ))
        } else {
            Ok((true, None))
        }
    }

    /// Get memory impact estimate for switching modes
    pub fn estimate_mode_impact(&mut self, current_mode: &PIIMode, new_mode: &PIIMode) -> Result<String> {
        let current_overhead = current_mode.memory_overhead_mb();
        let new_overhead = new_mode.memory_overhead_mb();
        let delta = (new_overhead as i64) - (current_overhead as i64);

        let info = self.get_memory_info()?;
        let available_mb = (info.available_memory / (1024 * 1024)) as i64;

        let message = if delta > 0 {
            format!(
                "Switching to {} will use an additional {}MB of RAM. {} available.",
                new_mode.display_name(),
                delta,
                available_mb
            )
        } else if delta < 0 {
            format!(
                "Switching to {} will free up {}MB of RAM.",
                new_mode.display_name(),
                -delta
            )
        } else {
            format!("No memory change.")
        };

        Ok(message)
    }
}

impl Default for MemoryDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pii_mode_memory_overhead() {
        assert_eq!(PIIMode::Builtin.memory_overhead_mb(), 0);
        assert_eq!(PIIMode::PresidioLite.memory_overhead_mb(), 500);
        assert_eq!(PIIMode::PresidioFull.memory_overhead_mb(), 2048);
    }

    #[test]
    fn test_pii_mode_accuracy() {
        assert_eq!(PIIMode::Builtin.accuracy(), 85);
        assert_eq!(PIIMode::PresidioLite.accuracy(), 90);
        assert_eq!(PIIMode::PresidioFull.accuracy(), 95);
    }

    #[test]
    fn test_pii_mode_serialization() {
        assert_eq!(PIIMode::from_string("builtin"), PIIMode::Builtin);
        assert_eq!(PIIMode::from_string("presidio_lite"), PIIMode::PresidioLite);
        assert_eq!(PIIMode::from_string("presidio_full"), PIIMode::PresidioFull);
        assert_eq!(PIIMode::from_string("invalid"), PIIMode::Builtin);
    }

    #[test]
    fn test_memory_detector_creation() {
        let mut detector = MemoryDetector::new();
        let info = detector.get_memory_info().unwrap();
        assert!(info.total_memory > 0);
        assert!(info.total_gb > 0.0);
    }
}
