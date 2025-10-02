import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface MemoryInfo {
  total_memory: number;
  available_memory: number;
  used_memory: number;
  process_memory: number;
  total_gb: number;
  available_gb: number;
  usage_percentage: number;
  recommended_mode: string;
  warning: string | null;
}

interface PIIMode {
  mode: 'builtin' | 'presidio_lite' | 'presidio_full';
  name: string;
  memory_mb: number;
  accuracy: number;
  speed: string;
  description: string;
}

const PII_MODES: PIIMode[] = [
  {
    mode: 'builtin',
    name: 'Built-in Only',
    memory_mb: 0,
    accuracy: 85,
    speed: 'Fast',
    description: 'Fast regex-based detection with 0MB overhead. ✅ Recommended for laptops.',
  },
  {
    mode: 'presidio_lite',
    name: 'Presidio Lite',
    memory_mb: 500,
    accuracy: 90,
    speed: 'Medium',
    description: 'Enhanced detection with spaCy NER. Moderate memory usage (~500MB).',
  },
  {
    mode: 'presidio_full',
    name: 'Presidio Full',
    memory_mb: 2048,
    accuracy: 95,
    speed: 'Slow',
    description: 'State-of-the-art ML detection with DeBERTa. High memory usage (~2GB).',
  },
];

export const PiiDetectionSettings: React.FC = () => {
  const [selectedMode, setSelectedMode] = useState<string>('builtin');
  const [memoryInfo, setMemoryInfo] = useState<MemoryInfo | null>(null);
  const [currentMemoryUsage, setCurrentMemoryUsage] = useState<number>(0);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadSettings();
    // Refresh memory info every 5 seconds
    const interval = setInterval(refreshMemoryInfo, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const [config, memInfo] = await Promise.all([
        invoke<{ presidio_mode: string }>('get_pii_config'),
        invoke<MemoryInfo>('get_memory_info'),
      ]);
      setSelectedMode(config.presidio_mode || 'builtin');
      setMemoryInfo(memInfo);
      setCurrentMemoryUsage(memInfo.process_memory / (1024 * 1024)); // Convert to MB
    } catch (error) {
      console.error('Failed to load PII settings:', error);
    } finally {
      setLoading(false);
    }
  };

  const refreshMemoryInfo = async () => {
    try {
      const memInfo = await invoke<MemoryInfo>('get_memory_info');
      setMemoryInfo(memInfo);
      setCurrentMemoryUsage(memInfo.process_memory / (1024 * 1024));
    } catch (error) {
      console.error('Failed to refresh memory info:', error);
    }
  };

  const handleModeChange = async (mode: string) => {
    setSelectedMode(mode);
  };

  const handleSave = async () => {
    try {
      setSaving(true);
      await invoke('set_pii_mode', { mode: selectedMode });
      await loadSettings();
      alert('PII detection mode updated successfully!');
    } catch (error) {
      console.error('Failed to save PII mode:', error);
      alert('Failed to save settings: ' + error);
    } finally {
      setSaving(false);
    }
  };

  const getModeWarning = (mode: PIIMode): string | null => {
    if (!memoryInfo) return null;

    const availableMB = memoryInfo.available_gb * 1024;
    const requiredMB = mode.memory_mb;
    const llmEstimate = 5500; // Typical LLM usage in MB
    const safeBuffer = 2048; // 2GB buffer

    const availableForPII = availableMB - llmEstimate - safeBuffer;

    if (requiredMB > availableForPII) {
      return `⚠️ Warning: ${mode.name} requires ${requiredMB}MB but only ${Math.floor(availableForPII)}MB available. May cause system instability.`;
    }

    return null;
  };

  const getSystemRecommendation = (): string => {
    if (!memoryInfo) return '';

    if (memoryInfo.warning) {
      return memoryInfo.warning;
    }

    if (memoryInfo.total_gb < 8) {
      return `System has ${memoryInfo.total_gb.toFixed(1)}GB RAM. Built-in detection recommended.`;
    } else if (memoryInfo.total_gb < 16) {
      return `System has ${memoryInfo.total_gb.toFixed(1)}GB RAM. Presidio Lite available if better accuracy needed.`;
    } else {
      return `System has ${memoryInfo.total_gb.toFixed(1)}GB RAM. All modes available.`;
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6 bg-gray-50 dark:bg-gray-900 rounded-lg">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          PII Detection Settings
        </h2>
        <p className="text-sm text-gray-600 dark:text-gray-400">
          Configure how BEAR detects and protects personally identifiable information (PII).
        </p>
      </div>

      {/* Memory Status */}
      {memoryInfo && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
            System Memory Status
          </h3>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-gray-600 dark:text-gray-400">Total RAM:</span>
              <span className="ml-2 font-semibold text-gray-900 dark:text-white">
                {memoryInfo.total_gb.toFixed(1)} GB
              </span>
            </div>
            <div>
              <span className="text-gray-600 dark:text-gray-400">Available:</span>
              <span className="ml-2 font-semibold text-gray-900 dark:text-white">
                {memoryInfo.available_gb.toFixed(1)} GB
              </span>
            </div>
            <div>
              <span className="text-gray-600 dark:text-gray-400">Current Process:</span>
              <span className="ml-2 font-semibold text-gray-900 dark:text-white">
                {currentMemoryUsage.toFixed(0)} MB
              </span>
            </div>
            <div>
              <span className="text-gray-600 dark:text-gray-400">Usage:</span>
              <span className="ml-2 font-semibold text-gray-900 dark:text-white">
                {memoryInfo.usage_percentage.toFixed(1)}%
              </span>
            </div>
          </div>
          {memoryInfo.warning && (
            <div className="mt-3 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded text-sm text-yellow-800 dark:text-yellow-200">
              {memoryInfo.warning}
            </div>
          )}
        </div>
      )}

      {/* Detection Mode Selection */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Detection Mode
        </h3>

        <div className="space-y-4">
          {PII_MODES.map((mode) => {
            const warning = getModeWarning(mode);
            const isRecommended = memoryInfo?.recommended_mode === mode.mode;

            return (
              <label
                key={mode.mode}
                className={`block p-4 border-2 rounded-lg cursor-pointer transition-all ${
                  selectedMode === mode.mode
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
                } ${warning ? 'opacity-75' : ''}`}
              >
                <div className="flex items-start">
                  <input
                    type="radio"
                    name="pii_mode"
                    value={mode.mode}
                    checked={selectedMode === mode.mode}
                    onChange={(e) => handleModeChange(e.target.value)}
                    disabled={!!warning}
                    className="mt-1 h-4 w-4 text-blue-600 border-gray-300 focus:ring-blue-500"
                  />
                  <div className="ml-3 flex-1">
                    <div className="flex items-center justify-between mb-1">
                      <span className="font-semibold text-gray-900 dark:text-white">
                        {mode.name}
                        {isRecommended && (
                          <span className="ml-2 text-xs bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 px-2 py-1 rounded">
                            Recommended
                          </span>
                        )}
                      </span>
                    </div>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                      {mode.description}
                    </p>
                    <div className="grid grid-cols-3 gap-2 text-xs">
                      <div>
                        <span className="text-gray-500 dark:text-gray-500">RAM:</span>
                        <span className="ml-1 font-medium text-gray-900 dark:text-white">
                          {mode.memory_mb === 0 ? '0MB' : `~${mode.memory_mb}MB`}
                        </span>
                      </div>
                      <div>
                        <span className="text-gray-500 dark:text-gray-500">Accuracy:</span>
                        <span className="ml-1 font-medium text-gray-900 dark:text-white">
                          {mode.accuracy}%
                        </span>
                      </div>
                      <div>
                        <span className="text-gray-500 dark:text-gray-500">Speed:</span>
                        <span className="ml-1 font-medium text-gray-900 dark:text-white">
                          {mode.speed}
                        </span>
                      </div>
                    </div>
                    {warning && (
                      <div className="mt-2 text-xs text-red-600 dark:text-red-400">
                        {warning}
                      </div>
                    )}
                  </div>
                </div>
              </label>
            );
          })}
        </div>
      </div>

      {/* System Recommendation */}
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
        <h4 className="font-semibold text-blue-900 dark:text-blue-200 mb-2">
          💡 System Recommendation
        </h4>
        <p className="text-sm text-blue-800 dark:text-blue-300">
          {getSystemRecommendation()}
        </p>
      </div>

      {/* Comparison Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700 overflow-x-auto">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Mode Comparison
        </h3>
        <table className="min-w-full text-sm">
          <thead>
            <tr className="border-b border-gray-200 dark:border-gray-700">
              <th className="text-left py-2 px-3 text-gray-700 dark:text-gray-300">Mode</th>
              <th className="text-left py-2 px-3 text-gray-700 dark:text-gray-300">RAM</th>
              <th className="text-left py-2 px-3 text-gray-700 dark:text-gray-300">Accuracy</th>
              <th className="text-left py-2 px-3 text-gray-700 dark:text-gray-300">Speed</th>
            </tr>
          </thead>
          <tbody>
            {PII_MODES.map((mode) => (
              <tr key={mode.mode} className="border-b border-gray-100 dark:border-gray-800">
                <td className="py-2 px-3 font-medium text-gray-900 dark:text-white">
                  {mode.name}
                </td>
                <td className="py-2 px-3 text-gray-600 dark:text-gray-400">
                  {mode.memory_mb === 0 ? '0MB' : `${mode.memory_mb}MB`}
                </td>
                <td className="py-2 px-3 text-gray-600 dark:text-gray-400">{mode.accuracy}%</td>
                <td className="py-2 px-3 text-gray-600 dark:text-gray-400">{mode.speed}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Save Button */}
      <div className="flex justify-end space-x-3">
        <button
          onClick={loadSettings}
          className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700"
        >
          Reset
        </button>
        <button
          onClick={handleSave}
          disabled={saving}
          className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {saving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  );
};

export default PiiDetectionSettings;
