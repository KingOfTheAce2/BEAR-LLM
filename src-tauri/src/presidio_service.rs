use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::{Command as AsyncCommand, Child};
use std::time::Duration;

/// Production-ready Presidio HTTP service manager
///
/// Instead of spawning Python processes for each detection request,
/// this runs Presidio as a persistent FastAPI microservice and
/// communicates via HTTP. This provides:
/// - Better performance (no process spawn overhead)
/// - Connection pooling
/// - Graceful error handling
/// - Health monitoring
/// - Clean shutdown

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresidioRequest {
    pub text: String,
    pub language: String,
    pub entities: Vec<String>,
    pub score_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresidioResponse {
    pub entities: Vec<PresidioEntity>,
    pub processing_time_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresidioEntity {
    pub entity_type: String,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub score: f32,
}

pub struct PresidioService {
    service_process: Arc<RwLock<Option<Child>>>,
    service_url: String,
    python_path: Arc<RwLock<Option<PathBuf>>>,
    service_port: u16,
    is_running: Arc<RwLock<bool>>,
}

impl PresidioService {
    pub fn new(port: u16) -> Self {
        Self {
            service_process: Arc::new(RwLock::new(None)),
            service_url: format!("http://127.0.0.1:{}", port),
            python_path: Arc::new(RwLock::new(None)),
            service_port: port,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the Presidio FastAPI service
    pub async fn start(&self, python_path: PathBuf) -> Result<()> {
        // Check if already running
        if *self.is_running.read().await {
            tracing::info!("Presidio service already running");
            return Ok(());
        }

        tracing::info!("🚀 Starting Presidio HTTP service on port {}...", self.service_port);

        // Save python path
        *self.python_path.write().await = Some(python_path.clone());

        // Create FastAPI service script
        let service_script = self.create_service_script()?;
        let script_path = std::env::temp_dir().join("presidio_service.py");
        tokio::fs::write(&script_path, service_script).await?;

        // Start the service
        let child = AsyncCommand::new(&python_path)
            .arg(&script_path)
            .arg("--port")
            .arg(self.service_port.to_string())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        *self.service_process.write().await = Some(child);

        // Wait for service to be ready
        self.wait_for_service_ready().await?;

        *self.is_running.write().await = true;

        tracing::info!("✅ Presidio service started successfully at {}", self.service_url);

        Ok(())
    }

    /// Stop the Presidio service gracefully
    pub async fn stop(&self) -> Result<()> {
        if !*self.is_running.read().await {
            return Ok(());
        }

        tracing::info!("🛑 Stopping Presidio service...");

        // Send shutdown request
        let shutdown_url = format!("{}/shutdown", self.service_url);
        if let Ok(_) = reqwest::get(&shutdown_url).await {
            tracing::info!("Shutdown request sent to service");
        }

        // Wait a bit for graceful shutdown
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Force kill if still running
        let mut process = self.service_process.write().await;
        if let Some(mut child) = process.take() {
            match child.kill().await {
                Ok(_) => tracing::info!("Service process terminated"),
                Err(e) => tracing::warn!("Failed to kill service process: {}", e),
            }
        }

        *self.is_running.write().await = false;

        tracing::info!("✅ Presidio service stopped");

        Ok(())
    }

    /// Detect PII in text using the HTTP service
    pub async fn detect(&self, request: PresidioRequest) -> Result<PresidioResponse> {
        if !*self.is_running.read().await {
            return Err(anyhow!("Presidio service not running. Call start() first."));
        }

        let client = reqwest::Client::new();
        let url = format!("{}/analyze", self.service_url);

        let response = client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to call Presidio service: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Presidio service error: {}", error_text));
        }

        let presidio_response: PresidioResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse Presidio response: {}", e))?;

        Ok(presidio_response)
    }

    /// Check if the service is healthy
    pub async fn health_check(&self) -> Result<bool> {
        if !*self.is_running.read().await {
            return Ok(false);
        }

        let url = format!("{}/health", self.service_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        match client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Wait for the service to be ready
    async fn wait_for_service_ready(&self) -> Result<()> {
        let max_retries = 30; // 30 seconds
        let retry_delay = Duration::from_secs(1);

        for attempt in 1..=max_retries {
            tracing::debug!("Waiting for Presidio service to be ready (attempt {}/{})", attempt, max_retries);

            match self.health_check().await {
                Ok(true) => {
                    tracing::info!("Presidio service is ready!");
                    return Ok(());
                }
                _ => {
                    if attempt < max_retries {
                        tokio::time::sleep(retry_delay).await;
                    }
                }
            }
        }

        Err(anyhow!("Presidio service failed to start within timeout"))
    }

    /// Create the FastAPI service script
    fn create_service_script(&self) -> Result<String> {
        let script = r#"
#!/usr/bin/env python3
"""
Presidio HTTP Service for BEAR AI
Production-ready FastAPI microservice for PII detection
"""

import argparse
import sys
import time
from typing import List, Optional
from fastapi import FastAPI, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel
import uvicorn

try:
    from presidio_analyzer import AnalyzerEngine
    from presidio_anonymizer import AnonymizerEngine
except ImportError as e:
    print(f"ERROR: Failed to import Presidio: {e}", file=sys.stderr)
    print("Please install: pip install presidio-analyzer presidio-anonymizer", file=sys.stderr)
    sys.exit(1)

# Initialize Presidio engines (do this once at startup)
print("🔧 Initializing Presidio engines...")
analyzer = AnalyzerEngine()
anonymizer = AnonymizerEngine()
print("✅ Presidio engines initialized")

app = FastAPI(title="Presidio PII Detection Service", version="1.0.0")

class AnalyzeRequest(BaseModel):
    text: str
    language: str = "en"
    entities: Optional[List[str]] = None
    score_threshold: float = 0.85

class Entity(BaseModel):
    entity_type: str
    text: str
    start: int
    end: int
    score: float

class AnalyzeResponse(BaseModel):
    entities: List[Entity]
    processing_time_ms: int

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "service": "presidio"}

@app.post("/analyze", response_model=AnalyzeResponse)
async def analyze_text(request: AnalyzeRequest):
    """Analyze text for PII entities"""
    try:
        start_time = time.time()

        # Run Presidio analysis
        results = analyzer.analyze(
            text=request.text,
            language=request.language,
            entities=request.entities,
            score_threshold=request.score_threshold
        )

        # Convert to response format
        entities = []
        for result in results:
            entities.append(Entity(
                entity_type=result.entity_type,
                text=request.text[result.start:result.end],
                start=result.start,
                end=result.end,
                score=result.score
            ))

        processing_time_ms = int((time.time() - start_time) * 1000)

        return AnalyzeResponse(
            entities=entities,
            processing_time_ms=processing_time_ms
        )

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/shutdown")
async def shutdown():
    """Graceful shutdown endpoint"""
    import os
    import signal

    def shutdown_handler():
        os.kill(os.getpid(), signal.SIGTERM)

    # Schedule shutdown
    import threading
    threading.Timer(1.0, shutdown_handler).start()

    return {"message": "Shutting down gracefully"}

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8765, help="Service port")
    args = parser.parse_args()

    print(f"🚀 Starting Presidio service on port {args.port}...")

    uvicorn.run(
        app,
        host="127.0.0.1",
        port=args.port,
        log_level="warning",
        access_log=False
    )
"#;

        Ok(script.to_string())
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

// Graceful shutdown on drop
impl Drop for PresidioService {
    fn drop(&mut self) {
        // Note: This is blocking, but it's the best we can do in Drop
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let service_process = self.service_process.clone();
            runtime.spawn(async move {
                let mut process = service_process.write().await;
                if let Some(mut child) = process.take() {
                    let _ = child.kill().await;
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let service = PresidioService::new(8765);
        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_script_generation() {
        let service = PresidioService::new(8765);
        let script = service.create_service_script();
        assert!(script.is_ok());
        assert!(script.unwrap().contains("FastAPI"));
    }
}
