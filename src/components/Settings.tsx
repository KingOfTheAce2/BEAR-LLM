import React, { useState } from 'react';
import { Settings as SettingsIcon, Download, Database, Shield, X } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface SettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

const Settings: React.FC<SettingsProps> = ({ isOpen, onClose }) => {
  const [downloadingPresidio, setDownloadingPresidio] = useState(false);
  const [downloadingRAG, setDownloadingRAG] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleDownloadPresidio = async () => {
    setDownloadingPresidio(true);
    setError(null);
    setSuccess(null);

    try {
      await invoke('download_presidio_models');
      setSuccess('Presidio models downloaded successfully');
    } catch (err) {
      setError(`Failed to download Presidio: ${err}`);
    } finally {
      setDownloadingPresidio(false);
    }
  };

  const handleSetupRAG = async () => {
    setDownloadingRAG(true);
    setError(null);
    setSuccess(null);

    try {
      await invoke('setup_rag_embeddings');
      setSuccess('RAG embeddings engine initialized successfully');
    } catch (err) {
      setError(`Failed to setup RAG: ${err}`);
    } finally {
      setDownloadingRAG(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[200] p-4">
      <div className="bg-[var(--bg-primary)] rounded-lg border border-[var(--border-primary)] max-w-2xl w-full max-h-[80vh] overflow-y-auto">
        {/* Header */}
        <div className="p-6 border-b border-[var(--border-primary)] flex items-center justify-between sticky top-0 bg-[var(--bg-primary)]">
          <div className="flex items-center gap-2">
            <SettingsIcon className="w-5 h-5" />
            <h2 className="text-xl font-semibold text-[var(--text-primary)]">Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-[var(--hover-bg)] rounded-lg transition-all"
            aria-label="Close settings"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Status Messages */}
          {error && (
            <div className="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-600 dark:text-red-400 text-sm">
              {error}
            </div>
          )}
          {success && (
            <div className="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg text-green-600 dark:text-green-400 text-sm">
              {success}
            </div>
          )}

          {/* Presidio PII Detection */}
          <div className="p-4 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border-primary)]">
            <div className="flex items-start justify-between mb-4">
              <div>
                <div className="flex items-center gap-2 mb-2">
                  <Shield className="w-5 h-5 text-[var(--text-primary)]" />
                  <h3 className="font-medium text-[var(--text-primary)]">Microsoft Presidio</h3>
                </div>
                <p className="text-sm text-[var(--text-secondary)]">
                  PII (Personally Identifiable Information) detection models for document privacy
                </p>
              </div>
            </div>
            <button
              onClick={handleDownloadPresidio}
              disabled={downloadingPresidio}
              className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-[var(--text-primary)] hover:bg-opacity-90 text-[var(--bg-primary)] rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {downloadingPresidio ? (
                <>
                  <Download className="w-4 h-4 animate-pulse" />
                  <span>Downloading Presidio Models...</span>
                </>
              ) : (
                <>
                  <Download className="w-4 h-4" />
                  <span>Download Presidio Models</span>
                </>
              )}
            </button>
          </div>

          {/* RAG Embeddings */}
          <div className="p-4 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border-primary)]">
            <div className="flex items-start justify-between mb-4">
              <div>
                <div className="flex items-center gap-2 mb-2">
                  <Database className="w-5 h-5 text-[var(--text-primary)]" />
                  <h3 className="font-medium text-[var(--text-primary)]">RAG Embeddings Engine</h3>
                </div>
                <p className="text-sm text-[var(--text-secondary)]">
                  Document search and retrieval engine for contextual AI responses
                </p>
              </div>
            </div>
            <button
              onClick={handleSetupRAG}
              disabled={downloadingRAG}
              className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-[var(--text-primary)] hover:bg-opacity-90 text-[var(--bg-primary)] rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {downloadingRAG ? (
                <>
                  <Download className="w-4 h-4 animate-pulse" />
                  <span>Setting up RAG Embeddings...</span>
                </>
              ) : (
                <>
                  <Download className="w-4 h-4" />
                  <span>Setup RAG Embeddings</span>
                </>
              )}
            </button>
          </div>

          {/* Info */}
          <div className="p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
            <p className="text-sm text-blue-600 dark:text-blue-400">
              💡 These components can be installed anytime. Skipping them during initial setup won't affect core functionality.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Settings;
