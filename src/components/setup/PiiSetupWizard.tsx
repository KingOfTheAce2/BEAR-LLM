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
  pros: string[];
  cons: string[];
}

const PII_MODES: PIIMode[] = [
  {
    mode: 'builtin',
    name: 'Built-in Only',
    memory_mb: 0,
    accuracy: 85,
    speed: 'Fast',
    description: 'Fast regex-based PII detection with zero memory overhead.',
    pros: [
      'Zero memory overhead - perfect for laptops',
      'Fast detection speed',
      'No Python dependencies',
      'Works offline immediately',
    ],
    cons: [
      'Lower accuracy (85%) vs. ML models',
      'May miss some complex PII patterns',
      'Limited context awareness',
    ],
  },
  {
    mode: 'presidio_lite',
    name: 'Presidio Lite',
    memory_mb: 500,
    accuracy: 90,
    speed: 'Medium',
    description: 'Enhanced PII detection using Microsoft Presidio with spaCy NER.',
    pros: [
      'Better accuracy (90%) with NER models',
      'Moderate memory usage (~500MB)',
      'Better context understanding',
      'Recognizes more PII types',
    ],
    cons: [
      'Requires Python and dependencies',
      'Slower than built-in detection',
      'Initial setup required',
    ],
  },
  {
    mode: 'presidio_full',
    name: 'Presidio Full',
    memory_mb: 2048,
    accuracy: 95,
    speed: 'Slow',
    description: 'State-of-the-art ML-based PII detection with DeBERTa transformer models.',
    pros: [
      'Highest accuracy (95%)',
      'State-of-the-art ML models',
      'Best context understanding',
      'Handles complex PII patterns',
    ],
    cons: [
      'High memory usage (~2GB)',
      'Slowest detection speed',
      'Requires powerful hardware',
      'Large dependency download',
    ],
  },
];

export const PiiSetupWizard: React.FC<{ onComplete?: () => void }> = ({ onComplete }) => {
  const [step, setStep] = useState<number>(1);
  const [selectedMode, setSelectedMode] = useState<string>('builtin');
  const [memoryInfo, setMemoryInfo] = useState<MemoryInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [installing, setInstalling] = useState(false);

  useEffect(() => {
    detectSystemMemory();
  }, []);

  const detectSystemMemory = async () => {
    try {
      setLoading(true);
      const memInfo = await invoke<MemoryInfo>('get_memory_info');
      setMemoryInfo(memInfo);
      // Auto-select recommended mode
      setSelectedMode(memInfo.recommended_mode || 'builtin');
    } catch (error) {
      console.error('Failed to detect system memory:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleComplete = async () => {
    try {
      setInstalling(true);
      await invoke('set_pii_mode', { mode: selectedMode });

      // If Presidio mode selected, optionally trigger installation
      if (selectedMode !== 'builtin') {
        const shouldInstall = confirm(
          'Would you like to install Presidio dependencies now? This may take a few minutes.'
        );
        if (shouldInstall) {
          await invoke('install_presidio');
        }
      }

      if (onComplete) {
        onComplete();
      }
    } catch (error) {
      console.error('Setup failed:', error);
      alert('Setup failed: ' + error);
    } finally {
      setInstalling(false);
    }
  };

  const getMemoryClass = (totalGb: number): string => {
    if (totalGb < 8) return 'Low';
    if (totalGb < 16) return 'Medium';
    return 'High';
  };

  const getRecommendationText = (): string => {
    if (!memoryInfo) return '';

    const memClass = getMemoryClass(memoryInfo.total_gb);

    switch (memClass) {
      case 'Low':
        return `Your system has ${memoryInfo.total_gb.toFixed(1)}GB RAM. Built-in detection is strongly recommended to ensure your LLM runs smoothly without memory pressure.`;
      case 'Medium':
        return `Your system has ${memoryInfo.total_gb.toFixed(1)}GB RAM. Built-in detection is recommended, but Presidio Lite is available if you need better accuracy and have headroom.`;
      case 'High':
        return `Your system has ${memoryInfo.total_gb.toFixed(1)}GB RAM. All detection modes are available. Choose based on your accuracy needs.`;
      default:
        return '';
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-100 dark:bg-gray-900">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-4 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-600 dark:text-gray-400">Detecting system resources...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            PII Detection Setup
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Configure how BEAR protects personally identifiable information
          </p>
        </div>

        {/* Progress Steps */}
        <div className="flex items-center justify-center mb-8">
          <div className="flex items-center space-x-4">
            <div className={`flex items-center ${step >= 1 ? 'text-blue-600' : 'text-gray-400'}`}>
              <div
                className={`rounded-full h-8 w-8 flex items-center justify-center border-2 ${
                  step >= 1
                    ? 'border-blue-600 bg-blue-600 text-white'
                    : 'border-gray-400 text-gray-400'
                }`}
              >
                1
              </div>
              <span className="ml-2 font-medium">System Check</span>
            </div>
            <div className="h-1 w-16 bg-gray-300 dark:bg-gray-700"></div>
            <div className={`flex items-center ${step >= 2 ? 'text-blue-600' : 'text-gray-400'}`}>
              <div
                className={`rounded-full h-8 w-8 flex items-center justify-center border-2 ${
                  step >= 2
                    ? 'border-blue-600 bg-blue-600 text-white'
                    : 'border-gray-400 text-gray-400'
                }`}
              >
                2
              </div>
              <span className="ml-2 font-medium">Choose Mode</span>
            </div>
            <div className="h-1 w-16 bg-gray-300 dark:bg-gray-700"></div>
            <div className={`flex items-center ${step >= 3 ? 'text-blue-600' : 'text-gray-400'}`}>
              <div
                className={`rounded-full h-8 w-8 flex items-center justify-center border-2 ${
                  step >= 3
                    ? 'border-blue-600 bg-blue-600 text-white'
                    : 'border-gray-400 text-gray-400'
                }`}
              >
                3
              </div>
              <span className="ml-2 font-medium">Complete</span>
            </div>
          </div>
        </div>

        {/* Content */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
          {/* Step 1: System Information */}
          {step === 1 && memoryInfo && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                System Information
              </h2>

              <div className="grid grid-cols-2 gap-4 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
                <div>
                  <p className="text-sm text-gray-600 dark:text-gray-400">Total RAM</p>
                  <p className="text-2xl font-bold text-gray-900 dark:text-white">
                    {memoryInfo.total_gb.toFixed(1)} GB
                  </p>
                </div>
                <div>
                  <p className="text-sm text-gray-600 dark:text-gray-400">Available RAM</p>
                  <p className="text-2xl font-bold text-gray-900 dark:text-white">
                    {memoryInfo.available_gb.toFixed(1)} GB
                  </p>
                </div>
                <div>
                  <p className="text-sm text-gray-600 dark:text-gray-400">Memory Class</p>
                  <p className="text-2xl font-bold text-gray-900 dark:text-white">
                    {getMemoryClass(memoryInfo.total_gb)}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-gray-600 dark:text-gray-400">Current Usage</p>
                  <p className="text-2xl font-bold text-gray-900 dark:text-white">
                    {memoryInfo.usage_percentage.toFixed(1)}%
                  </p>
                </div>
              </div>

              <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                <h3 className="font-semibold text-blue-900 dark:text-blue-200 mb-2">
                  💡 Recommendation
                </h3>
                <p className="text-sm text-blue-800 dark:text-blue-300">{getRecommendationText()}</p>
              </div>

              <div className="flex justify-end">
                <button
                  onClick={() => setStep(2)}
                  className="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  Continue
                </button>
              </div>
            </div>
          )}

          {/* Step 2: Mode Selection */}
          {step === 2 && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                Choose Detection Mode
              </h2>

              <div className="space-y-4">
                {PII_MODES.map((mode) => {
                  const isRecommended = memoryInfo?.recommended_mode === mode.mode;
                  const canUse =
                    !memoryInfo ||
                    mode.memory_mb <= (memoryInfo.available_gb * 1024 - 5500 - 2048);

                  return (
                    <div
                      key={mode.mode}
                      onClick={() => canUse && setSelectedMode(mode.mode)}
                      className={`border-2 rounded-lg p-4 cursor-pointer transition-all ${
                        selectedMode === mode.mode
                          ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                          : canUse
                          ? 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
                          : 'border-gray-200 dark:border-gray-700 opacity-50 cursor-not-allowed'
                      }`}
                    >
                      <div className="flex items-start justify-between mb-3">
                        <div className="flex items-center">
                          <input
                            type="radio"
                            checked={selectedMode === mode.mode}
                            disabled={!canUse}
                            onChange={() => {}}
                            className="h-4 w-4 text-blue-600 border-gray-300 focus:ring-blue-500"
                          />
                          <h3 className="ml-3 text-lg font-semibold text-gray-900 dark:text-white">
                            {mode.name}
                            {isRecommended && (
                              <span className="ml-2 text-xs bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 px-2 py-1 rounded">
                                Recommended
                              </span>
                            )}
                          </h3>
                        </div>
                        <div className="text-right text-sm">
                          <div className="text-gray-600 dark:text-gray-400">
                            {mode.memory_mb === 0 ? '0MB' : `~${mode.memory_mb}MB`}
                          </div>
                          <div className="text-gray-900 dark:text-white font-semibold">
                            {mode.accuracy}% accuracy
                          </div>
                        </div>
                      </div>

                      <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                        {mode.description}
                      </p>

                      <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                          <p className="font-semibold text-gray-700 dark:text-gray-300 mb-1">
                            Pros:
                          </p>
                          <ul className="list-disc list-inside text-gray-600 dark:text-gray-400 space-y-1">
                            {mode.pros.map((pro, idx) => (
                              <li key={idx}>{pro}</li>
                            ))}
                          </ul>
                        </div>
                        <div>
                          <p className="font-semibold text-gray-700 dark:text-gray-300 mb-1">
                            Cons:
                          </p>
                          <ul className="list-disc list-inside text-gray-600 dark:text-gray-400 space-y-1">
                            {mode.cons.map((con, idx) => (
                              <li key={idx}>{con}</li>
                            ))}
                          </ul>
                        </div>
                      </div>

                      {!canUse && (
                        <div className="mt-3 text-sm text-red-600 dark:text-red-400">
                          ⚠️ Not enough available memory for this mode
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>

              {/* Trade-off Matrix */}
              <div className="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 overflow-x-auto">
                <h3 className="font-semibold text-gray-900 dark:text-white mb-3">
                  Quick Comparison
                </h3>
                <table className="min-w-full text-sm">
                  <thead>
                    <tr className="border-b border-gray-200 dark:border-gray-700">
                      <th className="text-left py-2 px-3 text-gray-700 dark:text-gray-300">
                        Mode
                      </th>
                      <th className="text-left py-2 px-3 text-gray-700 dark:text-gray-300">RAM</th>
                      <th className="text-left py-2 px-3 text-gray-700 dark:text-gray-300">
                        Accuracy
                      </th>
                      <th className="text-left py-2 px-3 text-gray-700 dark:text-gray-300">
                        Speed
                      </th>
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
                        <td className="py-2 px-3 text-gray-600 dark:text-gray-400">
                          {mode.accuracy}%
                        </td>
                        <td className="py-2 px-3 text-gray-600 dark:text-gray-400">
                          {mode.speed}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              <div className="flex justify-between">
                <button
                  onClick={() => setStep(1)}
                  className="px-6 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-md hover:bg-gray-300 dark:hover:bg-gray-600"
                >
                  Back
                </button>
                <button
                  onClick={() => setStep(3)}
                  className="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  Continue
                </button>
              </div>
            </div>
          )}

          {/* Step 3: Confirmation */}
          {step === 3 && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                Confirm Your Selection
              </h2>

              <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
                <h3 className="text-xl font-semibold text-blue-900 dark:text-blue-200 mb-4">
                  {PII_MODES.find((m) => m.mode === selectedMode)?.name}
                </h3>
                <p className="text-blue-800 dark:text-blue-300 mb-4">
                  {PII_MODES.find((m) => m.mode === selectedMode)?.description}
                </p>
                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div>
                    <p className="text-blue-700 dark:text-blue-400">Memory Usage</p>
                    <p className="text-lg font-bold text-blue-900 dark:text-blue-200">
                      {PII_MODES.find((m) => m.mode === selectedMode)?.memory_mb === 0
                        ? '0MB'
                        : `~${PII_MODES.find((m) => m.mode === selectedMode)?.memory_mb}MB`}
                    </p>
                  </div>
                  <div>
                    <p className="text-blue-700 dark:text-blue-400">Accuracy</p>
                    <p className="text-lg font-bold text-blue-900 dark:text-blue-200">
                      {PII_MODES.find((m) => m.mode === selectedMode)?.accuracy}%
                    </p>
                  </div>
                  <div>
                    <p className="text-blue-700 dark:text-blue-400">Speed</p>
                    <p className="text-lg font-bold text-blue-900 dark:text-blue-200">
                      {PII_MODES.find((m) => m.mode === selectedMode)?.speed}
                    </p>
                  </div>
                </div>
              </div>

              <div className="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  <strong>Note:</strong> You can change this setting later in the application
                  settings.
                  {selectedMode !== 'builtin' &&
                    ' Presidio dependencies will be installed on first use or you can install them now.'}
                </p>
              </div>

              <div className="flex justify-between">
                <button
                  onClick={() => setStep(2)}
                  disabled={installing}
                  className="px-6 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-md hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50"
                >
                  Back
                </button>
                <button
                  onClick={handleComplete}
                  disabled={installing}
                  className="px-6 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {installing ? 'Setting up...' : 'Complete Setup'}
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default PiiSetupWizard;
