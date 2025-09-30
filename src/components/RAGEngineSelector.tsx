import React, { useState, useEffect } from 'react';
import { ChevronDown, CheckCircle, AlertCircle, Database, Zap } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface RAGModel {
  id: string;
  name: string;
  dimensions: number;
  size_mb: number;
  description: string;
  is_available: boolean;
}

interface RAGEngineSelectorProps {
  theme?: 'light' | 'dark';
}

const RAGEngineSelector: React.FC<RAGEngineSelectorProps> = ({ theme }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedEngine, setSelectedEngine] = useState<string>('BGE-Small-EN-V1.5');
  const [availableEngines, setAvailableEngines] = useState<RAGModel[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Get theme from data attribute if not provided
  const currentTheme = theme || document.documentElement.getAttribute('data-theme') || 'dark';

  // Theme-aware bear search icon
  const bearSearchIcon = currentTheme === 'dark'
    ? '/BEAR_search_icon_white.png'
    : '/BEAR_search_icon_black.png';

  useEffect(() => {
    loadAvailableEngines();
  }, []);

  const loadAvailableEngines = async () => {
    setLoading(true);
    try {
      const engines = await invoke<RAGModel[]>('get_available_rag_models');
      setAvailableEngines(engines);
      setError(null);
    } catch (err) {
      console.error('Failed to load RAG engines:', err);
      setError('Failed to load RAG engines');
      // Set default engines as fallback
      setAvailableEngines([
        {
          id: 'BGE-Small-EN-V1.5',
          name: 'BGE Small EN v1.5',
          dimensions: 384,
          size_mb: 133,
          description: 'Fast and efficient for corporate laptops',
          is_available: true
        }
      ]);
    } finally {
      setLoading(false);
    }
  };

  const handleEngineSelect = async (engineId: string) => {
    try {
      await invoke('select_rag_model', { modelId: engineId });
      setSelectedEngine(engineId);
      setIsOpen(false);
      setError(null);
    } catch (err) {
      console.error('Failed to select RAG engine:', err);
      setError(`Failed to select ${engineId}`);
    }
  };

  const formatSize = (sizeMb: number) => {
    if (sizeMb < 1024) {
      return `${Math.round(sizeMb)} MB`;
    }
    return `${(sizeMb / 1024).toFixed(1)} GB`;
  };

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center justify-between gap-2 px-3 py-2 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg hover:bg-[var(--hover-bg)] transition-all text-[var(--text-primary)]"
      >
        <div className="flex items-center gap-2">
          <img
            src={bearSearchIcon}
            alt="RAG Engine"
            className="w-4 h-4 object-contain"
            onError={(e) => {
              // Fallback to Database icon if image fails to load
              const target = e.target as HTMLImageElement;
              target.style.display = 'none';
            }}
          />
          <Database className="w-4 h-4" style={{ display: 'none' }} />
          <span className="text-sm font-medium">{selectedEngine}</span>
          {error && <AlertCircle className="w-3 h-3 text-yellow-500" />}
        </div>
        <ChevronDown className={`w-4 h-4 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
      </button>

      {isOpen && (
        <div className="absolute top-full mt-2 right-0 w-96 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg shadow-xl z-50">
          {/* Header with Bear Search Icon */}
          <div className="p-4 border-b border-[var(--border-primary)] bg-gradient-to-r from-blue-500/10 to-purple-500/10">
            <div className="flex items-center gap-3">
              <img
                src={bearSearchIcon}
                alt="RAG Engine"
                className="w-8 h-8 object-contain"
                onError={(e) => {
                  const target = e.target as HTMLImageElement;
                  target.style.display = 'none';
                }}
              />
              <div>
                <h3 className="text-sm font-semibold text-[var(--text-primary)]">RAG Search Engine</h3>
                <p className="text-xs text-[var(--text-secondary)]">Select embedding model for document search</p>
              </div>
            </div>
          </div>

          {error && (
            <div className="px-4 py-2 border-b border-[var(--border-primary)] bg-yellow-50 dark:bg-yellow-900/20">
              <p className="text-xs text-yellow-600 dark:text-yellow-400">{error}</p>
            </div>
          )}

          <div className="py-2 max-h-80 overflow-y-auto scrollbar-custom">
            {loading ? (
              <div className="px-4 py-8 text-center">
                <div className="animate-spin w-6 h-6 border-2 border-[var(--accent)] border-t-transparent rounded-full mx-auto mb-2"></div>
                <p className="text-sm text-[var(--text-secondary)]">Loading engines...</p>
              </div>
            ) : availableEngines.length === 0 ? (
              <div className="px-4 py-3 text-sm text-[var(--text-secondary)]">
                No RAG engines available
              </div>
            ) : (
              availableEngines.map((engine) => (
                <button
                  key={engine.id}
                  onClick={() => handleEngineSelect(engine.id)}
                  disabled={!engine.is_available}
                  className="w-full px-4 py-3 text-left hover:bg-[var(--hover-bg)] transition-colors disabled:opacity-50 disabled:cursor-not-allowed group"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-sm font-medium text-[var(--text-primary)]">
                          {engine.name}
                        </span>
                        {selectedEngine === engine.id && (
                          <CheckCircle className="w-4 h-4 text-green-500" />
                        )}
                        {!engine.is_available && (
                          <span className="text-xs px-2 py-0.5 bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400 rounded">
                            Not Downloaded
                          </span>
                        )}
                      </div>

                      <p className="text-xs text-[var(--text-secondary)] mb-2">
                        {engine.description}
                      </p>

                      <div className="flex items-center gap-3 text-xs text-[var(--text-tertiary)]">
                        <div className="flex items-center gap-1">
                          <Database className="w-3 h-3" />
                          {engine.dimensions}D
                        </div>
                        <div className="flex items-center gap-1">
                          <Zap className="w-3 h-3" />
                          {formatSize(engine.size_mb)}
                        </div>
                      </div>
                    </div>
                  </div>
                </button>
              ))
            )}
          </div>

          <div className="px-4 py-3 border-t border-[var(--border-primary)] bg-[var(--bg-tertiary)]">
            <div className="flex items-center justify-between text-xs text-[var(--text-tertiary)]">
              <span>🔒 Local processing only</span>
              <span>🚀 Optimized embeddings</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default RAGEngineSelector;
