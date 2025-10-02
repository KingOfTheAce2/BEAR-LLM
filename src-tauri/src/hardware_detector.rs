use anyhow::{anyhow, Result};
use nvml_wrapper::Nvml;
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpecs {
    pub cpu_cores: usize,
    pub cpu_frequency: u64, // MHz
    pub cpu_brand: String,
    pub total_memory: u64,     // MB
    pub available_memory: u64, // MB
    pub gpu_info: Option<GpuInfo>,
    pub system_type: SystemType,
    pub performance_category: PerformanceCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub memory_total: u64, // MB
    pub memory_free: u64,  // MB
    pub compute_capability: Option<String>,
    pub driver_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemType {
    CorporateLaptop,
    Workstation,
    Server,
    PersonalComputer,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceCategory {
    Budget,      // < 8GB RAM, basic CPU
    Standard,    // 8-16GB RAM, mid-range CPU
    Performance, // 16-32GB RAM, high-end CPU
    Workstation, // > 32GB RAM, professional CPU
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub model_id: String,
    pub model_name: String,
    pub confidence: f64,              // 0.0-1.0
    pub expected_performance: String, // e.g., "15-20 words/sec"
    pub memory_usage: String,         // e.g., "6-8 GB"
    pub reasoning: String,
    pub compatibility_score: f64, // 0.0-1.0
}

pub struct HardwareDetector {
    system: System,
}

impl HardwareDetector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self { system }
    }

    pub fn detect_hardware(&mut self) -> Result<HardwareSpecs> {
        self.system.refresh_all();

        let cpu_cores = self.system.cpus().len();
        let cpu_frequency = self
            .system
            .cpus()
            .first()
            .map(|cpu| cpu.frequency())
            .unwrap_or(0);
        let cpu_brand = self
            .system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let total_memory = self.system.total_memory() / 1024 / 1024; // Convert to MB
        let available_memory = self.system.available_memory() / 1024 / 1024; // Convert to MB

        let gpu_info = self.detect_gpu().ok();
        let system_type = self.classify_system_type(cpu_cores, total_memory, &gpu_info);
        let performance_category = self.classify_performance(cpu_cores, total_memory, &gpu_info);

        Ok(HardwareSpecs {
            cpu_cores,
            cpu_frequency,
            cpu_brand,
            total_memory,
            available_memory,
            gpu_info,
            system_type,
            performance_category,
        })
    }

    fn detect_gpu(&self) -> Result<GpuInfo> {
        // Try NVML for NVIDIA GPUs first
        if let Ok(nvml) = Nvml::init() {
            if let Ok(device_count) = nvml.device_count() {
                if device_count > 0 {
                    if let Ok(device) = nvml.device_by_index(0) {
                        let name = device
                            .name()
                            .unwrap_or_else(|_| "Unknown NVIDIA GPU".to_string());
                        let memory_info = device.memory_info().ok();
                        let memory_total = memory_info
                            .as_ref()
                            .map(|m| m.total / 1024 / 1024)
                            .unwrap_or(0);
                        let memory_free = memory_info
                            .as_ref()
                            .map(|m| m.free / 1024 / 1024)
                            .unwrap_or(0);
                        let driver_version = nvml
                            .sys_driver_version()
                            .unwrap_or_else(|_| "Unknown".to_string());

                        return Ok(GpuInfo {
                            name,
                            memory_total,
                            memory_free,
                            compute_capability: None, // Could be detected with more advanced NVML calls
                            driver_version,
                        });
                    }
                }
            }
        }

        // Fallback to system detection for other GPUs
        Err(anyhow!("No compatible GPU detected"))
    }

    fn classify_system_type(
        &self,
        cpu_cores: usize,
        total_memory: u64,
        gpu_info: &Option<GpuInfo>,
    ) -> SystemType {
        // Heuristics to classify system type
        if total_memory < 16 * 1024 && cpu_cores <= 8 {
            if self.is_laptop_cpu() {
                SystemType::CorporateLaptop
            } else {
                SystemType::PersonalComputer
            }
        } else if total_memory >= 32 * 1024 && cpu_cores >= 8 {
            if gpu_info
                .as_ref()
                .map(|gpu| gpu.memory_total >= 8 * 1024)
                .unwrap_or(false)
            {
                SystemType::Workstation
            } else {
                SystemType::Server
            }
        } else {
            SystemType::PersonalComputer
        }
    }

    fn classify_performance(
        &self,
        cpu_cores: usize,
        total_memory: u64,
        gpu_info: &Option<GpuInfo>,
    ) -> PerformanceCategory {
        let memory_gb = total_memory / 1024;
        let has_powerful_gpu = gpu_info
            .as_ref()
            .map(|gpu| gpu.memory_total >= 8 * 1024)
            .unwrap_or(false);

        match (memory_gb, cpu_cores, has_powerful_gpu) {
            (0..=7, 0..=3, false) => PerformanceCategory::Budget,
            (8..=15, 4..=7, _) => PerformanceCategory::Standard,
            (16..=31, 6..=15, _) => PerformanceCategory::Performance,
            (32.., 8.., _) => PerformanceCategory::Workstation,
            _ => PerformanceCategory::Standard,
        }
    }

    fn is_laptop_cpu(&self) -> bool {
        let cpu_brand = self
            .system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_lowercase())
            .unwrap_or_default();

        // Common laptop CPU indicators
        cpu_brand.contains("mobile") ||
        cpu_brand.contains("u") ||  // Intel U-series
        cpu_brand.contains("h") ||  // Intel H-series
        cpu_brand.contains("hs") || // AMD HS-series
        cpu_brand.contains("ryzen") && (cpu_brand.contains("4000") || cpu_brand.contains("5000") || cpu_brand.contains("6000"))
    }

    pub fn recommend_models(&self, hardware: &HardwareSpecs) -> Vec<ModelRecommendation> {
        let mut recommendations = Vec::new();

        match hardware.performance_category {
            PerformanceCategory::Budget => {
                recommendations.push(ModelRecommendation {
                    model_id: "TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string(),
                    model_name: "TinyLlama 1.1B Chat".to_string(),
                    confidence: 0.95,
                    expected_performance: "12-18 words/sec".to_string(),
                    memory_usage: "3-4 GB".to_string(),
                    reasoning: "Optimized for budget systems with limited RAM".to_string(),
                    compatibility_score: 0.92,
                });

                recommendations.push(ModelRecommendation {
                    model_id: "microsoft/DialoGPT-small".to_string(),
                    model_name: "DialoGPT Small".to_string(),
                    confidence: 0.88,
                    expected_performance: "15-22 words/sec".to_string(),
                    memory_usage: "2-3 GB".to_string(),
                    reasoning: "Lightweight conversational model for resource-constrained systems"
                        .to_string(),
                    compatibility_score: 0.95,
                });

                recommendations.push(ModelRecommendation {
                    model_id: "distilbert-base-uncased".to_string(),
                    model_name: "DistilBERT Base".to_string(),
                    confidence: 0.82,
                    expected_performance: "25-35 words/sec".to_string(),
                    memory_usage: "2-3 GB".to_string(),
                    reasoning: "Efficient BERT model for text understanding tasks".to_string(),
                    compatibility_score: 0.90,
                });
            }

            PerformanceCategory::Standard => {
                recommendations.push(ModelRecommendation {
                    model_id: "microsoft/DialoGPT-medium".to_string(),
                    model_name: "DialoGPT Medium".to_string(),
                    confidence: 0.92,
                    expected_performance: "18-26 words/sec".to_string(),
                    memory_usage: "6-8 GB".to_string(),
                    reasoning: "Balanced performance for standard corporate laptops".to_string(),
                    compatibility_score: 0.94,
                });

                recommendations.push(ModelRecommendation {
                    model_id: "microsoft/phi-2".to_string(),
                    model_name: "Phi-2".to_string(),
                    confidence: 0.87,
                    expected_performance: "15-22 words/sec".to_string(),
                    memory_usage: "8-12 GB".to_string(),
                    reasoning: "Small language model with strong reasoning capabilities"
                        .to_string(),
                    compatibility_score: 0.88,
                });

                recommendations.push(ModelRecommendation {
                    model_id: "TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string(),
                    model_name: "TinyLlama 1.1B Chat".to_string(),
                    confidence: 0.85,
                    expected_performance: "20-28 words/sec".to_string(),
                    memory_usage: "3-4 GB".to_string(),
                    reasoning: "Efficient option with room for other applications".to_string(),
                    compatibility_score: 0.91,
                });
            }

            PerformanceCategory::Performance => {
                if hardware.gpu_info.is_some() {
                    recommendations.push(ModelRecommendation {
                        model_id: "microsoft/DialoGPT-large".to_string(),
                        model_name: "DialoGPT Large".to_string(),
                        confidence: 0.89,
                        expected_performance: "12-18 words/sec".to_string(),
                        memory_usage: "12-16 GB".to_string(),
                        reasoning: "Large model for high-performance systems with GPU acceleration"
                            .to_string(),
                        compatibility_score: 0.85,
                    });
                }

                recommendations.push(ModelRecommendation {
                    model_id: "microsoft/phi-2".to_string(),
                    model_name: "Phi-2".to_string(),
                    confidence: 0.94,
                    expected_performance: "20-30 words/sec".to_string(),
                    memory_usage: "8-12 GB".to_string(),
                    reasoning: "Excellent reasoning capabilities for complex tasks".to_string(),
                    compatibility_score: 0.93,
                });

                recommendations.push(ModelRecommendation {
                    model_id: "microsoft/DialoGPT-medium".to_string(),
                    model_name: "DialoGPT Medium".to_string(),
                    confidence: 0.91,
                    expected_performance: "25-35 words/sec".to_string(),
                    memory_usage: "6-8 GB".to_string(),
                    reasoning: "Fast performance with available system resources".to_string(),
                    compatibility_score: 0.96,
                });
            }

            PerformanceCategory::Workstation => {
                recommendations.push(ModelRecommendation {
                    model_id: "microsoft/DialoGPT-large".to_string(),
                    model_name: "DialoGPT Large".to_string(),
                    confidence: 0.95,
                    expected_performance: "15-25 words/sec".to_string(),
                    memory_usage: "12-16 GB".to_string(),
                    reasoning: "Premium model for workstation-class hardware".to_string(),
                    compatibility_score: 0.92,
                });

                recommendations.push(ModelRecommendation {
                    model_id: "microsoft/phi-2".to_string(),
                    model_name: "Phi-2".to_string(),
                    confidence: 0.93,
                    expected_performance: "25-40 words/sec".to_string(),
                    memory_usage: "8-12 GB".to_string(),
                    reasoning: "High-performance reasoning model with excellent speed".to_string(),
                    compatibility_score: 0.97,
                });

                // Add more advanced models for workstations
                recommendations.push(ModelRecommendation {
                    model_id: "CodeLlama-7b-Instruct-hf".to_string(),
                    model_name: "CodeLlama 7B Instruct".to_string(),
                    confidence: 0.88,
                    expected_performance: "12-20 words/sec".to_string(),
                    memory_usage: "14-18 GB".to_string(),
                    reasoning: "Specialized code generation and analysis model".to_string(),
                    compatibility_score: 0.85,
                });
            }
        }

        // Sort by compatibility score and confidence
        recommendations.sort_by(|a, b| {
            let score_a = (a.compatibility_score + a.confidence) / 2.0;
            let score_b = (b.compatibility_score + b.confidence) / 2.0;
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top 3 recommendations
        recommendations.truncate(3);
        recommendations
    }

    pub fn get_system_summary(&self, hardware: &HardwareSpecs) -> String {
        let gpu_info = hardware
            .gpu_info
            .as_ref()
            .map(|gpu| format!("GPU: {} ({} MB VRAM)", gpu.name, gpu.memory_total))
            .unwrap_or_else(|| "GPU: None (CPU-only mode)".to_string());

        format!(
            "System Type: {:?}\nPerformance Category: {:?}\nCPU: {} ({} cores, {} MHz)\nRAM: {} MB total, {} MB available\n{}",
            hardware.system_type,
            hardware.performance_category,
            hardware.cpu_brand,
            hardware.cpu_cores,
            hardware.cpu_frequency,
            hardware.total_memory,
            hardware.available_memory,
            gpu_info
        )
    }

    pub fn estimate_model_performance(
        &self,
        hardware: &HardwareSpecs,
        model_size_gb: f64,
    ) -> String {
        let _memory_gb = hardware.total_memory as f64 / 1024.0;
        let available_gb = hardware.available_memory as f64 / 1024.0;

        if model_size_gb > available_gb * 0.8 {
            return "Not recommended - insufficient memory".to_string();
        }

        let base_performance = match hardware.performance_category {
            PerformanceCategory::Budget => 10.0,
            PerformanceCategory::Standard => 20.0,
            PerformanceCategory::Performance => 35.0,
            PerformanceCategory::Workstation => 50.0,
        };

        // Adjust for GPU
        let gpu_multiplier = if hardware.gpu_info.is_some() {
            1.5
        } else {
            1.0
        };

        // Adjust for model size
        let size_factor = if model_size_gb < 2.0 {
            1.2
        } else if model_size_gb > 10.0 {
            0.7
        } else {
            1.0
        };

        let estimated_words_per_sec = base_performance * gpu_multiplier * size_factor;

        format!(
            "{:.0}-{:.0} words/sec",
            estimated_words_per_sec * 0.8,
            estimated_words_per_sec * 1.2
        )
    }
}

impl Default for HardwareDetector {
    fn default() -> Self {
        Self::new()
    }
}
