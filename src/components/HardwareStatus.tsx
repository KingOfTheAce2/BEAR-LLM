import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Monitor, Cpu, MemoryStick, HardDrive, Zap, AlertTriangle, CheckCircle, Download, Loader2 } from 'lucide-react';

interface HardwareSpecs {
  cpu_cores: number;
  cpu_frequency: number;
  cpu_brand: string;
  total_memory: number;
  available_memory: number;
  gpu_info?: {
    name: string;
    memory_total: number;
    memory_free: number;
    driver_version: string;
  };
  system_type: string;
  performance_category: string;
}

interface ModelRecommendation {
  model_id: string;
  model_name: string;
  confidence: number;
  expected_performance: string;
  memory_usage: string;
  reasoning: string;
  compatibility_score: number;
}

const HardwareStatus: React.FC = () => {
  const [hardware, setHardware] = useState<HardwareSpecs | null>(null);
  const [recommendations, setRecommendations] = useState<ModelRecommendation[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showDetails, setShowDetails] = useState(false);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<number>(0);

  useEffect(() => {
    detectHardware();
  }, []);

  const detectHardware = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const specs = await invoke<HardwareSpecs>('detect_hardware');
      const recs = await invoke<ModelRecommendation[]>('get_model_recommendations');

      setHardware(specs);
      setRecommendations(recs);
    } catch (err) {
      setError(`Failed to detect hardware: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const getPerformanceColor = (category: string) => {
    switch (category.toLowerCase()) {
      case 'budget': return 'text-yellow-500';
      case 'standard': return 'text-blue-500';
      case 'performance': return 'text-green-500';
      case 'workstation': return 'text-purple-500';
      default: return 'text-gray-500';
    }
  };

  const getPerformanceIcon = (category: string) => {
    switch (category.toLowerCase()) {
      case 'budget': return <AlertTriangle className="w-4 h-4" />;
      case 'standard': return <CheckCircle className="w-4 h-4" />;
      case 'performance': return <Zap className="w-4 h-4" />;
      case 'workstation': return <Monitor className="w-4 h-4" />;
      default: return <Cpu className="w-4 h-4" />;
    }
  };

  const handleDownloadModel = async (modelId: string) => {
    setDownloading(modelId);
    setDownloadProgress(0);
    setError(null);

    try {
      // Simulate progress updates (TODO: Connect to actual Tauri download progress events)
      const progressInterval = setInterval(() => {
        setDownloadProgress(prev => Math.min(prev + 10, 90));
      }, 500);

      await invoke('download_model_from_huggingface', { modelId });

      clearInterval(progressInterval);
      setDownloadProgress(100);

      setTimeout(() => {
        setDownloading(null);
        setDownloadProgress(0);
      }, 1000);
    } catch (err) {
      setError(`Failed to download ${modelId}: ${err}`);
      setDownloading(null);
      setDownloadProgress(0);
    }
  };

  if (isLoading) {
    return (
      <div className="p-4 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border-primary)]">
        <div className="flex items-center gap-2 text-[var(--text-secondary)]">
          <Monitor className="w-4 h-4 animate-pulse" />
          <span className="text-sm">Detecting hardware...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800">
        <div className="flex items-center gap-2 text-red-600 dark:text-red-400">
          <AlertTriangle className="w-4 h-4" />
          <span className="text-sm">{error}</span>
        </div>
      </div>
    );
  }

  if (!hardware) return null;

  return (
    <div className="space-y-4">
      {/* Hardware Summary */}
      <div className="p-4 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border-primary)]">
        <button
          onClick={() => setShowDetails(!showDetails)}
          className="w-full flex items-center justify-between text-left"
        >
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-lg bg-gradient-to-br ${getPerformanceColor(hardware.performance_category)} bg-opacity-10`}>
              {getPerformanceIcon(hardware.performance_category)}
            </div>
            <div>
              <h3 className="font-medium text-[var(--text-primary)]">
                {hardware.system_type} • {hardware.performance_category}
              </h3>
              <p className="text-sm text-[var(--text-secondary)]">
                {hardware.cpu_cores} cores • {Math.round(hardware.total_memory / 1024)}GB RAM
                {hardware.gpu_info && ` • ${hardware.gpu_info.name}`}
              </p>
            </div>
          </div>
          <div className={`text-xs px-2 py-1 rounded-full ${getPerformanceColor(hardware.performance_category)} bg-opacity-20`}>
            {hardware.performance_category}
          </div>
        </button>

        {showDetails && (
          <div className="mt-4 pt-4 border-t border-[var(--border-primary)] space-y-3">
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="flex items-center gap-2">
                <Cpu className="w-4 h-4 text-[var(--text-tertiary)]" />
                <span className="text-[var(--text-secondary)]">
                  {hardware.cpu_brand}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <MemoryStick className="w-4 h-4 text-[var(--text-tertiary)]" />
                <span className="text-[var(--text-secondary)]">
                  {Math.round(hardware.available_memory / 1024)}GB available
                </span>
              </div>
            </div>

            {hardware.gpu_info && (
              <div className="p-3 bg-[var(--bg-tertiary)] rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <Monitor className="w-4 h-4 text-green-500" />
                  <span className="text-sm font-medium text-[var(--text-primary)]">GPU Detected</span>
                </div>
                <div className="text-xs text-[var(--text-secondary)] space-y-1">
                  <div>{hardware.gpu_info.name}</div>
                  <div>{Math.round(hardware.gpu_info.memory_total / 1024)}GB VRAM</div>
                  <div>Driver: {hardware.gpu_info.driver_version}</div>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Model Recommendations */}
      {recommendations.length > 0 && (
        <div className="space-y-2">
          <h4 className="text-sm font-medium text-[var(--text-primary)] px-1">
            Recommended Models for Your Hardware
          </h4>
          {recommendations.slice(0, 3).map((rec, index) => (
            <div
              key={rec.model_id}
              className="p-3 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border-primary)] hover:border-[var(--accent)] transition-all"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-medium text-[var(--text-primary)]">
                      {rec.model_name}
                    </span>
                    <div className="text-xs px-2 py-0.5 bg-green-100 dark:bg-green-900/20 text-green-600 dark:text-green-400 rounded-full">
                      {Math.round(rec.confidence * 100)}% match
                    </div>
                  </div>

                  <div className="text-xs text-[var(--text-secondary)] space-y-1">
                    <div className="flex items-center gap-4">
                      <span>⚡ {rec.expected_performance}</span>
                      <span>💾 {rec.memory_usage}</span>
                    </div>
                    <div className="text-[var(--text-tertiary)]">
                      {rec.reasoning}
                    </div>
                  </div>
                </div>

                <div className="flex flex-col items-end gap-2">
                  <div className="text-xs text-[var(--text-tertiary)]">
                    Score: {Math.round(rec.compatibility_score * 100)}%
                  </div>
                  {index === 0 && (
                    <div className="text-xs text-green-500">Recommended</div>
                  )}
                  {downloading === rec.model_id ? (
                    <div className="flex flex-col items-end gap-1">
                      <div className="flex items-center gap-2">
                        <Loader2 className="w-4 h-4 animate-spin text-blue-500" />
                        <span className="text-xs text-blue-500">{downloadProgress}%</span>
                      </div>
                      <div className="w-24 h-1 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-blue-500 transition-all duration-300"
                          style={{ width: `${downloadProgress}%` }}
                        />
                      </div>
                    </div>
                  ) : (
                    <button
                      onClick={() => handleDownloadModel(rec.model_id)}
                      className="flex items-center gap-1 px-3 py-1.5 bg-blue-500 hover:bg-blue-600 text-white text-xs rounded-lg transition-all hover:scale-105"
                      title="Download GGUF model"
                    >
                      <Download className="w-3 h-3" />
                      <span>Download</span>
                    </button>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default HardwareStatus;