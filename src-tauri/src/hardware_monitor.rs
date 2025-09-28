use anyhow::{Result, anyhow};
use sysinfo::{System, SystemExt, CpuExt, ProcessExt, PidExt};
use std::time::Duration;

#[cfg(target_os = "windows")]
use nvml_wrapper::Nvml;

use crate::SystemStatus;

pub struct HardwareMonitor {
    system: System,
    cpu_threshold: f32,
    memory_threshold: f32,
    gpu_threshold: f32,
    temperature_threshold: f32,
    consecutive_high_readings: usize,
    max_consecutive_high: usize,
    #[cfg(target_os = "windows")]
    nvml: Option<Nvml>,
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
            #[cfg(target_os = "windows")]
            nvml,
        }
    }

    pub async fn update_metrics(&mut self) -> Result<()> {
        self.system.refresh_cpu();
        self.system.refresh_memory();
        self.system.refresh_processes();
        self.system.refresh_components_list();
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
        use sysinfo::ComponentExt;

        for component in self.system.components() {
            if component.label().contains("CPU") || component.label().contains("Core") {
                return Some(component.temperature());
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

    pub async fn get_process_info(&self) -> Vec<ProcessInfo> {
        let mut processes = Vec::new();

        for (pid, process) in self.system.processes() {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory() as f32 / 1024.0 / 1024.0,
            });
        }

        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        processes.truncate(10);
        processes
    }

    pub async fn emergency_throttle(&mut self) -> Result<()> {
        println!("EMERGENCY: System resources critically high. Implementing throttling...");

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("powercfg")
                .args(&["/setactive", "a1841308-3541-4fab-bc81-f71556f20b4a"])
                .output()?;
        }

        std::thread::sleep(Duration::from_secs(5));

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}