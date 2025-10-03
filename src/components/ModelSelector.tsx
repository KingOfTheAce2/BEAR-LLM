import React, { useState, useEffect } from 'react';
import { ChevronDown, Download, CheckCircle, AlertCircle, Globe, Laptop, MemoryStick, HardDrive, Zap } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore, HuggingFaceModel } from '../stores/appStore';
import HuggingFaceBrowser from './HuggingFaceBrowser';
import HardwareStatus from './HardwareStatus';


const ModelSelector: React.FC = () => {
  const {
    selectedModel,
    setSelectedModel,
    availableModels,
    setAvailableModels,
    corporateModels
  } = useAppStore();
  const [isOpen, setIsOpen] = useState(false);
  const [showBrowser, setShowBrowser] = useState(false);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showHardware, setShowHardware] = useState(false);

  useEffect(() => {
    loadAvailableModels();
  }, []);

  const loadAvailableModels = async () => {
    try {
      const models = await invoke<string[]>('list_available_models');
      setAvailableModels(models);
      setError(null);
    } catch (error) {
      console.warn('Failed to load models from backend, using corporate models');
      // Use corporate model IDs when backend is not available
      const corporateModelIds = corporateModels.map(model => model.id);
      setAvailableModels(corporateModelIds);
      setError('Using corporate models - backend not connected');
    }
  };

  const handleModelSelect = (model: string | HuggingFaceModel) => {
    if (typeof model === 'string') {
      setSelectedModel(model);
    } else {
      setSelectedModel(model.id);
    }
    setIsOpen(false);
    setShowBrowser(false);
  };

  const handleBrowseModels = () => {
    setShowBrowser(true);
    setIsOpen(false);
  };

  const getModelInfo = (modelId: string) => {
    return corporateModels.find(model => model.id === modelId);
  };

  const getPerformanceColor = (performance: string) => {
    switch (performance) {
      case 'low': return 'text-green-500';
      case 'medium': return 'text-yellow-500';
      case 'high': return 'text-red-500';
      default: return 'text-gray-500';
    }
  };

  const handleDownloadModel = async (model: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setDownloading(model);
    try {
      await invoke('download_model', { modelName: model });
      setDownloading(null);
      setError(null);
    } catch (error) {
      console.error('Failed to download model:', error);
      setDownloading(null);
      setError(`Failed to download ${model}`);
    }
  };

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center justify-between w-full px-4 py-2 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg hover:bg-[var(--hover-bg)] transition-all text-[var(--text-primary)]"
      >
        <div className="flex items-center gap-2">
          <span className="text-sm">{selectedModel}</span>
          {error && <AlertCircle className="w-3 h-3 text-yellow-500" />}
        </div>
        <ChevronDown className={`w-4 h-4 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
      </button>

      {isOpen && (
        <div className="absolute top-full mt-2 right-0 w-80 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg shadow-xl z-[100]">
          {error && (
            <div className="px-4 py-2 border-b border-[var(--border-primary)] bg-yellow-50 dark:bg-yellow-900/20">
              <p className="text-xs text-yellow-600 dark:text-yellow-400">{error}</p>
            </div>
          )}

          {/* Hardware Recommendations */}
          <div className="p-3 border-b border-[var(--border-primary)]">
            <button
              onClick={() => setShowHardware(!showHardware)}
              className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-[var(--text-primary)] hover:bg-opacity-90 text-[var(--bg-primary)] rounded-lg transition-all mb-2"
            >
              <Zap className="w-4 h-4" />
              <span className="font-medium">Hardware Recommendations</span>
            </button>

            <button
              onClick={handleBrowseModels}
              className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-[var(--bg-tertiary)] hover:bg-[var(--hover-bg)] border border-[var(--border-primary)] text-[var(--text-primary)] rounded-lg transition-all"
            >
              <Globe className="w-4 h-4" />
              <span className="font-medium">Browse HuggingFace Models</span>
            </button>
          </div>

          <div className="py-2 max-h-72 overflow-y-auto scrollbar-custom">
            {availableModels.length === 0 ? (
              <div className="px-4 py-3 text-sm text-[var(--text-secondary)]">
                No models available
              </div>
            ) : (
              availableModels.map((modelId) => {
                const modelInfo = getModelInfo(modelId);
                return (
                  <button
                    key={modelId}
                    onClick={() => handleModelSelect(modelId)}
                    className="w-full px-4 py-3 text-left hover:bg-[var(--hover-bg)] transition-colors group"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-sm font-medium text-[var(--text-primary)]">
                            {modelInfo?.name || modelId}
                          </span>
                          {selectedModel === modelId && (
                            <CheckCircle className="w-4 h-4 text-green-500" />
                          )}
                        </div>

                        {modelInfo && (
                          <>
                            <div className="text-xs text-[var(--text-secondary)] mb-1">
                              by {modelInfo.author}
                            </div>

                            <div className="flex items-center gap-3 text-xs text-[var(--text-tertiary)]">
                              {modelInfo.size && (
                                <div className="flex items-center gap-1">
                                  <HardDrive className="w-3 h-3" />
                                  {modelInfo.size}
                                </div>
                              )}
                              {modelInfo.systemRequirements && (
                                <>
                                  <div className="flex items-center gap-1">
                                    <MemoryStick className="w-3 h-3" />
                                    {modelInfo.systemRequirements.recommendedRam}
                                  </div>
                                  <div className={`flex items-center gap-1 ${getPerformanceColor(modelInfo.systemRequirements.performance)}`}>
                                    <Laptop className="w-3 h-3" />
                                    {modelInfo.systemRequirements.performance}
                                  </div>
                                </>
                              )}
                            </div>
                          </>
                        )}
                      </div>

                      <div className="flex items-center gap-2 ml-2">
                        {downloading === modelId ? (
                          <div className="flex items-center gap-1">
                            <Download className="w-4 h-4 animate-pulse text-blue-500" />
                            <span className="text-xs text-blue-500">Downloading...</span>
                          </div>
                        ) : (
                          modelInfo?.isLocal || (
                            <button
                              onClick={(e) => handleDownloadModel(modelId, e)}
                              className="p-1 rounded hover:bg-[var(--accent)] hover:text-white transition-colors opacity-0 group-hover:opacity-100"
                              title="Download model"
                            >
                              <Download className="w-3 h-3" />
                            </button>
                          )
                        )}
                      </div>
                    </div>
                  </button>
                );
              })
            )}
          </div>

          <div className="px-4 py-2 border-t border-[var(--border-primary)] bg-[var(--bg-tertiary)]">
            <div className="flex items-center justify-between text-xs text-[var(--text-tertiary)]">
              <span>💡 Privacy: Models run locally</span>
              <span>🏢 Corporate optimized</span>
            </div>
          </div>
        </div>
      )}

      <HuggingFaceBrowser
        isOpen={showBrowser}
        onClose={() => setShowBrowser(false)}
        onModelSelect={handleModelSelect}
      />

      {/* Hardware Status Modal */}
      {showHardware && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
          <div className="bg-[var(--bg-primary)] rounded-lg border border-[var(--border-primary)] max-w-2xl w-full max-h-[80vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold text-[var(--text-primary)]">Hardware Analysis & Recommendations</h2>
                <button
                  onClick={() => setShowHardware(false)}
                  className="p-2 hover:bg-[var(--hover-bg)] rounded-lg transition-all"
                >
                  ✕
                </button>
              </div>
              <HardwareStatus />
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ModelSelector;