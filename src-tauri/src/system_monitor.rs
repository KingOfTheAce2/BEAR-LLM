use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::process::Command;
use sysinfo::{System, Components};
use nvml_wrapper::Nvml;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSpecs {
    pub gpu: GpuInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub os: String,
    pub capability_score: u32, // 0-100
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuInfo {
    pub available: bool,
    pub name: String,
    pub vram_total_mb: u64,
    pub vram_used_mb: u64,
    pub vram_free_mb: u64,
    pub temperature: f32,
    pub utilization: u32,
    pub cuda_available: bool,
    pub compute_capability: String,
    pub driver_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuInfo {
    pub brand: String,
    pub core_count: usize,
    pub frequency_mhz: u64,
    pub usage_percent: f32,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelCompatibility {
    pub model_name: String,
    pub compatibility: CompatibilityLevel,
    pub vram_required_mb: u64,
    pub ram_required_mb: u64,
    pub estimated_tokens_per_second: f32,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CompatibilityLevel {
    Excellent,      // Runs perfectly, plenty of headroom
    Good,          // Runs well with good performance
    Borderline,    // Will run but may have issues
    NotRecommended, // Will likely crash or be unusable
    Incompatible,  // Cannot run at all
}

pub struct SystemMonitor {
    system: System,
    nvml: Option<Nvml>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        // Try to initialize NVIDIA Management Library
        let nvml = Nvml::init().ok();

        SystemMonitor { system, nvml }
    }

    pub fn get_system_specs(&mut self) -> SystemSpecs {
        self.system.refresh_all();

        let gpu = self.get_gpu_info();
        let cpu = self.get_cpu_info();
        let memory = self.get_memory_info();
        let os = self.get_os_info();

        let capability_score = self.calculate_capability_score(&gpu, &cpu, &memory);

        SystemSpecs {
            gpu,
            cpu,
            memory,
            os,
            capability_score,
        }
    }

    fn get_gpu_info(&self) -> GpuInfo {
        if let Some(ref nvml) = self.nvml {
            // NVIDIA GPU detected
            if let Ok(device_count) = nvml.device_count() {
                if device_count > 0 {
                    if let Ok(device) = nvml.device_by_index(0) {
                        let name = device.name().unwrap_or_else(|_| "Unknown GPU".to_string());

                        let mem_info = device.memory_info().unwrap_or_else(|_| {
                            nvml_wrapper::struct_wrappers::device::MemoryInfo {
                                total: 0,
                                free: 0,
                                used: 0,
                                reserved: 0,
                                version: 0,
                            }
                        });

                        let temperature = device
                            .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                            .unwrap_or(0) as f32;

                        let utilization = device
                            .utilization_rates()
                            .map(|u| u.gpu)
                            .unwrap_or(0);

                        let compute_cap = device
                            .cuda_compute_capability()
                            .map(|cc| format!("{}.{}", cc.major, cc.minor))
                            .unwrap_or_else(|_| "Unknown".to_string());

                        let driver_version = nvml
                            .sys_driver_version()
                            .unwrap_or_else(|_| "Unknown".to_string());

                        return GpuInfo {
                            available: true,
                            name,
                            vram_total_mb: mem_info.total / 1_048_576,
                            vram_used_mb: mem_info.used / 1_048_576,
                            vram_free_mb: mem_info.free / 1_048_576,
                            temperature,
                            utilization,
                            cuda_available: true,
                            compute_capability: compute_cap,
                            driver_version,
                        };
                    }
                }
            }
        }

        // Check for AMD GPU using rocm-smi or Windows WMI
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            if let Ok(output) = Command::new("wmic")
                .args(&["path", "win32_VideoController", "get", "name,AdapterRAM"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("AMD") || output_str.contains("Radeon") {
                    // Parse AMD GPU info from WMI
                    // Parse GPU info from nvidia-smi output
                    return GpuInfo {
                        available: true,
                        name: "AMD GPU Detected".to_string(),
                        vram_total_mb: 8192, // Would parse from WMI
                        vram_used_mb: 0,
                        vram_free_mb: 8192,
                        temperature: 0.0,
                        utilization: 0,
                        cuda_available: false,
                        compute_capability: "ROCm".to_string(),
                        driver_version: "Unknown".to_string(),
                    };
                }
            }
        }

        // No GPU detected or CPU only
        GpuInfo {
            available: false,
            name: "No GPU detected".to_string(),
            vram_total_mb: 0,
            vram_used_mb: 0,
            vram_free_mb: 0,
            temperature: 0.0,
            utilization: 0,
            cuda_available: false,
            compute_capability: "N/A".to_string(),
            driver_version: "N/A".to_string(),
        }
    }

    fn get_cpu_info(&mut self) -> CpuInfo {
        self.system.refresh_cpu_all();

        let brand = self.system.cpus()[0].brand().to_string();
        let core_count = self.system.cpus().len();
        let frequency_mhz = self.system.cpus()[0].frequency();
        let usage_percent = self.system.global_cpu_usage();

        // Get CPU temperature using the new Components API
        let temperature = {
            let components = Components::new_with_refreshed_list();
            components
                .iter()
                .find(|c| c.label().contains("CPU") || c.label().contains("Core"))
                .and_then(|c| c.temperature())
                .unwrap_or(0.0)
        };

        CpuInfo {
            brand,
            core_count,
            frequency_mhz,
            usage_percent,
            temperature: temperature,
        }
    }

    fn get_memory_info(&mut self) -> MemoryInfo {
        self.system.refresh_memory();

        let total_mb = self.system.total_memory() / 1024;
        let used_mb = self.system.used_memory() / 1024;
        let available_mb = self.system.available_memory() / 1024;
        let usage_percent = (used_mb as f32 / total_mb as f32) * 100.0;

        MemoryInfo {
            total_mb,
            used_mb,
            available_mb,
            usage_percent,
        }
    }

    fn get_os_info(&self) -> String {
        format!(
            "{} {}",
            System::name().unwrap_or_else(|| "Unknown".to_string()),
            System::os_version().unwrap_or_else(|| "".to_string())
        )
    }

    fn calculate_capability_score(&self, gpu: &GpuInfo, cpu: &CpuInfo, memory: &MemoryInfo) -> u32 {
        let mut score = 0u32;

        // GPU scoring (0-50 points)
        if gpu.available {
            if gpu.cuda_available {
                score += 10; // CUDA available
            }

            // VRAM scoring
            score += match gpu.vram_total_mb {
                v if v >= 24576 => 40, // 24GB+ - Excellent
                v if v >= 16384 => 35, // 16GB+ - Very good
                v if v >= 12288 => 30, // 12GB+ - Good
                v if v >= 8192 => 25,  // 8GB+ - Decent
                v if v >= 6144 => 20,  // 6GB+ - Minimum for most models
                v if v >= 4096 => 15,  // 4GB+ - Limited
                _ => 5,                 // Less than 4GB
            };
        }

        // CPU scoring (0-25 points)
        score += match cpu.core_count {
            c if c >= 16 => 15,
            c if c >= 12 => 12,
            c if c >= 8 => 10,
            c if c >= 6 => 8,
            c if c >= 4 => 5,
            _ => 2,
        };

        // CPU frequency bonus
        if cpu.frequency_mhz >= 4000 {
            score += 10;
        } else if cpu.frequency_mhz >= 3500 {
            score += 7;
        } else if cpu.frequency_mhz >= 3000 {
            score += 5;
        }

        // RAM scoring (0-25 points)
        score += match memory.total_mb {
            m if m >= 65536 => 25,  // 64GB+ - Excellent
            m if m >= 32768 => 20,  // 32GB+ - Very good
            m if m >= 16384 => 15,  // 16GB+ - Good
            m if m >= 8192 => 10,   // 8GB+ - Minimum
            _ => 5,                  // Less than 8GB
        };

        score.min(100) // Cap at 100
    }

    pub fn check_model_compatibility(&mut self, model_params: &ModelParams) -> ModelCompatibility {
        let specs = self.get_system_specs();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Calculate required resources based on model parameters
        let vram_required_mb = calculate_vram_requirement(model_params);
        let ram_required_mb = calculate_ram_requirement(model_params);

        // Check if system can run the model
        let compatibility = if !specs.gpu.available {
            if model_params.param_count > 3_000_000_000 {
                recommendations.push("This model requires a GPU for acceptable performance".to_string());
                CompatibilityLevel::NotRecommended
            } else {
                warnings.push("Running on CPU only - expect slow performance".to_string());
                CompatibilityLevel::Borderline
            }
        } else if specs.gpu.vram_free_mb < vram_required_mb {
            if specs.gpu.vram_total_mb >= vram_required_mb {
                warnings.push(format!(
                    "Insufficient free VRAM. Need {}MB but only {}MB free. Close other applications.",
                    vram_required_mb, specs.gpu.vram_free_mb
                ));
                recommendations.push("Close GPU-intensive applications before loading".to_string());
                CompatibilityLevel::Borderline
            } else {
                warnings.push(format!(
                    "GPU VRAM insufficient. Need {}MB but GPU only has {}MB total.",
                    vram_required_mb, specs.gpu.vram_total_mb
                ));
                recommendations.push("Consider using quantized version or smaller model".to_string());
                CompatibilityLevel::NotRecommended
            }
        } else if specs.memory.available_mb < ram_required_mb {
            warnings.push(format!(
                "Low system RAM. Need {}MB but only {}MB available.",
                ram_required_mb, specs.memory.available_mb
            ));
            CompatibilityLevel::Borderline
        } else if specs.gpu.vram_free_mb >= vram_required_mb * 2 {
            recommendations.push("Excellent headroom for this model".to_string());
            CompatibilityLevel::Excellent
        } else {
            CompatibilityLevel::Good
        };

        // Estimate performance
        let estimated_tokens_per_second = estimate_inference_speed(&specs, model_params);

        // Add temperature warnings if running hot
        if specs.gpu.temperature > 80.0 {
            warnings.push("GPU running hot. Ensure proper cooling before loading model.".to_string());
        }

        if specs.cpu.temperature > 85.0 {
            warnings.push("CPU temperature high. May throttle during inference.".to_string());
        }

        ModelCompatibility {
            model_name: model_params.name.clone(),
            compatibility,
            vram_required_mb,
            ram_required_mb,
            estimated_tokens_per_second,
            warnings,
            recommendations,
        }
    }

    pub fn monitor_resources_realtime(&mut self) -> ResourceSnapshot {
        self.system.refresh_all();

        let gpu_snapshot = if let Some(ref nvml) = self.nvml {
            if let Ok(device) = nvml.device_by_index(0) {
                GpuSnapshot {
                    vram_used_percent: {
                        let mem = device.memory_info().unwrap();
                        (mem.used as f32 / mem.total as f32) * 100.0
                    },
                    utilization: device.utilization_rates().map(|u| u.gpu).unwrap_or(0),
                    temperature: device
                        .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                        .unwrap_or(0) as f32,
                    power_watts: device.power_usage().unwrap_or(0) / 1000,
                }
            } else {
                GpuSnapshot::default()
            }
        } else {
            GpuSnapshot::default()
        };

        ResourceSnapshot {
            timestamp: std::time::SystemTime::now(),
            gpu: gpu_snapshot,
            cpu_usage: self.system.global_cpu_usage(),
            ram_usage_percent: (self.system.used_memory() as f32 / self.system.total_memory() as f32) * 100.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelParams {
    pub name: String,
    pub param_count: u64,        // in millions (7B = 7000)
    pub quantization: Quantization,
    pub context_length: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Quantization {
    F32,    // Full precision
    F16,    // Half precision
    Q8_0,   // 8-bit quantization
    Q5KM,   // 5-bit quantization - Fixed naming
    Q4KM,   // 4-bit quantization - Fixed naming
    Q4_0,   // 4-bit quantization (older)
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GpuSnapshot {
    pub vram_used_percent: f32,
    pub utilization: u32,
    pub temperature: f32,
    pub power_watts: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub timestamp: std::time::SystemTime,
    pub gpu: GpuSnapshot,
    pub cpu_usage: f32,
    pub ram_usage_percent: f32,
}

fn calculate_vram_requirement(model: &ModelParams) -> u64 {
    // Calculate VRAM requirement in MB based on model size and quantization
    let base_size = model.param_count * match model.quantization {
        Quantization::F32 => 4,     // 4 bytes per parameter
        Quantization::F16 => 2,     // 2 bytes per parameter
        Quantization::Q8_0 => 1,    // ~1 byte per parameter
        Quantization::Q5KM => 1,    // ~0.625 bytes per parameter (simplified to 1)
        Quantization::Q4KM => 1,    // ~0.5 bytes per parameter (simplified to 1)
        Quantization::Q4_0 => 1,    // ~0.5 bytes per parameter (simplified to 1)
    };

    // Add overhead for context, activations, etc (roughly 20%)
    let with_overhead = (base_size as f64 * 1.2) as u64;

    // Add context buffer (depends on context length)
    let context_buffer = (model.context_length as u64 * 1024) / 1_048_576; // Rough estimate

    with_overhead + context_buffer
}

fn calculate_ram_requirement(model: &ModelParams) -> u64 {
    // System RAM requirement is typically 2x the model size for safe operation
    calculate_vram_requirement(model) * 2
}

fn estimate_inference_speed(specs: &SystemSpecs, model: &ModelParams) -> f32 {
    // Rough estimation of tokens per second based on hardware
    if !specs.gpu.available {
        // CPU only - very rough estimates
        match model.param_count {
            p if p <= 3_000 => 5.0,   // 3B or less
            p if p <= 7_000 => 2.0,   // 7B
            p if p <= 13_000 => 0.5,  // 13B
            _ => 0.1,                  // Larger models
        }
    } else {
        // GPU available - based on VRAM and model size
        let gpu_factor = match specs.gpu.vram_total_mb {
            v if v >= 24576 => 3.0,  // High-end GPU
            v if v >= 16384 => 2.5,  // Good GPU
            v if v >= 12288 => 2.0,  // Decent GPU
            v if v >= 8192 => 1.5,   // Entry-level GPU
            _ => 1.0,
        };

        match model.param_count {
            p if p <= 3_000 => 50.0 * gpu_factor,
            p if p <= 7_000 => 30.0 * gpu_factor,
            p if p <= 13_000 => 15.0 * gpu_factor,
            p if p <= 30_000 => 8.0 * gpu_factor,
            _ => 3.0 * gpu_factor,
        }
    }
}