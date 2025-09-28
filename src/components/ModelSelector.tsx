import React, { useState, useEffect } from 'react';
import { ChevronDown, Download, CheckCircle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../stores/appStore';

const ModelSelector: React.FC = () => {
  const { selectedModel, setSelectedModel, availableModels, setAvailableModels } = useAppStore();
  const [isOpen, setIsOpen] = useState(false);
  const [downloading, setDownloading] = useState<string | null>(null);

  useEffect(() => {
    loadAvailableModels();
  }, []);

  const loadAvailableModels = async () => {
    try {
      const models = await invoke<string[]>('list_available_models');
      setAvailableModels(models);
    } catch (error) {
      // Silently handle error - fallback to empty models list
      setAvailableModels([]);
    }
  };

  const handleModelSelect = (model: string) => {
    setSelectedModel(model);
    setIsOpen(false);
  };

  const handleDownloadModel = async (model: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setDownloading(model);
    try {
      await invoke('download_model', { modelName: model });
      setDownloading(null);
    } catch (error) {
      // Silently handle download failure - reset downloading state
      setDownloading(null);
    }
  };

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center space-x-2 px-4 py-2 bg-legal-secondary rounded-lg hover:bg-legal-accent transition-colors"
      >
        <span>{selectedModel}</span>
        <ChevronDown className={`w-4 h-4 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
      </button>

      {isOpen && (
        <div className="absolute top-full mt-2 right-0 w-64 bg-legal-secondary rounded-lg shadow-xl z-50">
          <div className="py-2">
            {availableModels.map((model) => (
              <button
                key={model}
                onClick={() => handleModelSelect(model)}
                className="w-full px-4 py-2 text-left hover:bg-legal-accent transition-colors flex items-center justify-between"
              >
                <span>{model}</span>
                {selectedModel === model ? (
                  <CheckCircle className="w-4 h-4 text-green-400" />
                ) : downloading === model ? (
                  <Download className="w-4 h-4 animate-pulse" />
                ) : (
                  <button
                    onClick={(e) => handleDownloadModel(model, e)}
                    className="hover:text-legal-accent"
                  >
                    <Download className="w-4 h-4" />
                  </button>
                )}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default ModelSelector;