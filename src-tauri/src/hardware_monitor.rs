use anyhow::Result;
use std::time::Duration;
use sysinfo::{Components, ProcessesToUpdate, System};

#[cfg(target_os = "windows")]
use nvml_wrapper::Nvml;

use crate::SystemStatus;

/// Resource limits for system monitoring and enforcement
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_gpu_usage: f32,
    pub max_cpu_usage: f32,
    pub max_ram_usage: f32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_gpu_usage: 85.0,
            max_cpu_usage: 85.0,
            max_ram_usage: 90.0,
        }
    }
}

pub struct HardwareMonitor {
    system: System,
    cpu_threshold: f32,
    memory_threshold: f32,
    gpu_threshold: f32,
    temperature_threshold: f32,
    consecutive_high_readings: usize,
    max_consecutive_high: usize,
    resource_limits: ResourceLimits,
    #[cfg(target_os = "windows")]
    nvml: Option<Nvml>,
}

#[allow(dead_code)]
impl Default for HardwareMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        #[cfg(target_os = "windows")]
        let nvml = Nvml::init().ok();

        Self {
            system,
            cpu_threshold: 85.0,
            memory_threshold: 90.0,
            gpu_threshold: 85.0,
            temperature_threshold: 80.0,
            consecutive_high_readings: 0,
            max_consecutive_high: 3,
            resource_limits: ResourceLimits::default(),
            #[cfg(target_os = "windows")]
            nvml,
        }
    }

    pub async fn update_metrics(&mut self) -> Result<()> {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        // refresh_components_list() no longer exists - components refresh automatically
        Ok(())
    }

    pub async fn get_status(&self) -> Result<SystemStatus> {
        let cpu_usage = self.get_cpu_usage();
        let memory_usage = self.get_memory_usage();
        let gpu_usage = self.get_gpu_usage().await?;
        let temperature = self.get_temperature();

        let is_safe = self.check_thresholds(cpu_usage, memory_usage, gpu_usage, temperature);

        Ok(SystemStatus {
            cpu_usage,
            memory_usage,
            gpu_usage,
            temperature,
            is_safe,
        })
    }

    pub async fn check_safety(&mut self) -> Result<bool> {
        let status = self.get_status().await?;

        if !status.is_safe {
            self.consecutive_high_readings += 1;
            if self.consecutive_high_readings >= self.max_consecutive_high {
                return Ok(false);
            }
        } else {
            self.consecutive_high_readings = 0;
        }

        Ok(true)
    }

    fn get_cpu_usage(&self) -> f32 {
        let mut total = 0.0;
        let cpu_count = self.system.cpus().len();

        for cpu in self.system.cpus() {
            total += cpu.cpu_usage();
        }

        if cpu_count > 0 {
            total / cpu_count as f32
        } else {
            0.0
        }
    }

    fn get_memory_usage(&self) -> f32 {
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();

        if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        }
    }

    async fn get_gpu_usage(&self) -> Result<Option<f32>> {
        #[cfg(target_os = "windows")]
        {
            if let Some(ref nvml) = self.nvml {
                let device_count = nvml.device_count()?;
                if device_count > 0 {
                    let device = nvml.device_by_index(0)?;
                    let utilization = device.utilization_rates()?;
                    return Ok(Some(utilization.gpu as f32));
                }
            }
        }

        Ok(None)
    }

    fn get_temperature(&self) -> Option<f32> {
        // Use the new Components API from sysinfo 0.37
        let components = Components::new_with_refreshed_list();
        for component in &components {
            if component.label().contains("CPU") || component.label().contains("Core") {
                return component.temperature();
            }
        }
        None
    }

    fn check_thresholds(&self, cpu: f32, memory: f32, gpu: Option<f32>, temp: Option<f32>) -> bool {
        if cpu > self.cpu_threshold {
            return false;
        }

        if memory > self.memory_threshold {
            return false;
        }

        if let Some(gpu_usage) = gpu {
            if gpu_usage > self.gpu_threshold {
                return false;
            }
        }

        if let Some(temperature) = temp {
            if temperature > self.temperature_threshold {
                return false;
            }
        }

        true
    }

    pub fn set_thresholds(&mut self, cpu: f32, memory: f32, gpu: f32, temperature: f32) {
        self.cpu_threshold = cpu;
        self.memory_threshold = memory;
        self.gpu_threshold = gpu;
        self.temperature_threshold = temperature;
    }

    /// Set and enforce resource limits for GPU, CPU, and RAM usage
    pub fn set_resource_limits(
        &mut self,
        max_gpu_usage: f32,
        max_cpu_usage: f32,
        max_ram_usage: f32,
    ) -> Result<()> {
        // Validate limits
        if !(0.0..=100.0).contains(&max_gpu_usage) {
            return Err(anyhow::anyhow!("GPU usage limit must be between 0 and 100"));
        }
        if !(0.0..=100.0).contains(&max_cpu_usage) {
            return Err(anyhow::anyhow!("CPU usage limit must be between 0 and 100"));
        }
        if !(0.0..=100.0).contains(&max_ram_usage) {
            return Err(anyhow::anyhow!("RAM usage limit must be between 0 and 100"));
        }

        // Store the validated limits
        self.resource_limits = ResourceLimits {
            max_gpu_usage,
            max_cpu_usage,
            max_ram_usage,
        };

        // Update thresholds to match the resource limits
        self.cpu_threshold = max_cpu_usage;
        self.memory_threshold = max_ram_usage;
        self.gpu_threshold = max_gpu_usage;

        tracing::info!(
            gpu_limit = max_gpu_usage,
            cpu_limit = max_cpu_usage,
            ram_limit = max_ram_usage,
            "Resource limits updated and will be enforced"
        );

        Ok(())
    }

    /// Get current resource limits
    pub fn get_resource_limits(&self) -> ResourceLimits {
        self.resource_limits.clone()
    }

    /// Check if current resource usage is within configured limits
    pub async fn check_resource_limits(&self) -> Result<ResourceLimitStatus> {
        let cpu_usage = self.get_cpu_usage();
        let memory_usage = self.get_memory_usage();
        let gpu_usage = self.get_gpu_usage().await?;

        let cpu_exceeded = cpu_usage > self.resource_limits.max_cpu_usage;
        let ram_exceeded = memory_usage > self.resource_limits.max_ram_usage;
        let gpu_exceeded = if let Some(gpu) = gpu_usage {
            gpu > self.resource_limits.max_gpu_usage
        } else {
            false
        };

        let within_limits = !cpu_exceeded && !ram_exceeded && !gpu_exceeded;

        Ok(ResourceLimitStatus {
            within_limits,
            cpu_usage,
            cpu_limit: self.resource_limits.max_cpu_usage,
            cpu_exceeded,
            memory_usage,
            memory_limit: self.resource_limits.max_ram_usage,
            memory_exceeded: ram_exceeded,
            gpu_usage,
            gpu_limit: self.resource_limits.max_gpu_usage,
            gpu_exceeded,
        })
    }

    /// Enforce resource limits by rejecting operations that would exceed them
    pub async fn enforce_resource_limits(&self, operation_name: &str) -> Result<()> {
        let status = self.check_resource_limits().await?;

        if !status.within_limits {
            let mut exceeded = Vec::new();

            if status.cpu_exceeded {
                exceeded.push(format!(
                    "CPU usage ({:.1}%) exceeds limit ({:.1}%)",
                    status.cpu_usage, status.cpu_limit
                ));
            }

            if status.memory_exceeded {
                exceeded.push(format!(
                    "RAM usage ({:.1}%) exceeds limit ({:.1}%)",
                    status.memory_usage, status.memory_limit
                ));
            }

            if status.gpu_exceeded {
                if let Some(gpu) = status.gpu_usage {
                    exceeded.push(format!(
                        "GPU usage ({:.1}%) exceeds limit ({:.1}%)",
                        gpu, status.gpu_limit
                    ));
                }
            }

            let error_msg = format!(
                "Operation '{}' rejected: Resource limits exceeded. {}",
                operation_name,
                exceeded.join(", ")
            );

            tracing::warn!(
                operation = operation_name,
                exceeded = ?exceeded,
                "Operation rejected due to resource limit enforcement"
            );

            return Err(anyhow::anyhow!(error_msg));
        }

        Ok(())
    }

    pub async fn get_process_info(&self) -> Vec<ProcessInfo> {
        let mut processes = Vec::new();

        for (pid, process) in self.system.processes() {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory() as f32 / 1024.0 / 1024.0,
            });
        }

        // Sort by CPU usage, handling NaN values safely
        processes.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.truncate(10);
        processes
    }

    pub async fn emergency_throttle(&mut self) -> Result<()> {
        tracing::warn!("EMERGENCY: System resources critically high. Implementing throttling...");

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            use std::process::Command;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            Command::new("powercfg")
                .args(&["/setactive", "a1841308-3541-4fab-bc81-f71556f20b4a"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()?;
        }

        std::thread::sleep(Duration::from_secs(5));

        Ok(())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

/// Status of resource limits enforcement
#[derive(Debug, Clone)]
pub struct ResourceLimitStatus {
    pub within_limits: bool,
    pub cpu_usage: f32,
    pub cpu_limit: f32,
    pub cpu_exceeded: bool,
    pub memory_usage: f32,
    pub memory_limit: f32,
    pub memory_exceeded: bool,
    pub gpu_usage: Option<f32>,
    pub gpu_limit: f32,
    pub gpu_exceeded: bool,
}
