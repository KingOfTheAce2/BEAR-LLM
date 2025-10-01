import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Shield,
  Download,
  CheckCircle,
  AlertCircle,
  Loader2,
  Cpu,
  HardDrive,
  Zap,
  Lock,
  Brain,
  FileSearch,
  Sun,
  Moon
} from 'lucide-react';

interface SetupProgress {
  step: string;
  progress: number;
  message: string;
  is_complete: boolean;
  has_error: boolean;
}

interface SetupWizardProps {
  onComplete: () => void;
  theme: 'light' | 'dark';
  onThemeToggle: () => void;
}

const SetupWizard: React.FC<SetupWizardProps> = ({ onComplete, theme, onThemeToggle }) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [isInstalling, setIsInstalling] = useState(false);
  const [progress, setProgress] = useState<SetupProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [setupConfig, setSetupConfig] = useState({
    install_presidio: false,
    install_models: false,
    model_size: 'small',
    enable_gpu: false,
  });

  useEffect(() => {
    // Check if this is first run
    const checkFirstRun = async () => {
      try {
        const isFirstRun = await invoke<boolean>('check_first_run');
        if (!isFirstRun) {
          onComplete();
        }
      } catch (err) {
        console.error('Error checking first run:', err);
      }
    };

    checkFirstRun();

    // Listen for setup progress
    const unlisten = listen<SetupProgress>('setup-progress', (event) => {
      setProgress(event.payload);

      if (event.payload.is_complete) {
        setTimeout(() => {
          onComplete();
        }, 2000);
      }

      if (event.payload.has_error) {
        setError(event.payload.message);
        setIsInstalling(false);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [onComplete]);

  const startSetup = async () => {
    // If nothing is selected, just skip to completion
    if (!setupConfig.install_presidio && !setupConfig.install_models) {
      await markSetupComplete();
      return;
    }

    setIsInstalling(true);
    setError(null);

    try {
      await invoke('run_initial_setup', { config: setupConfig });
    } catch (err) {
      setError(`Setup failed: ${err}`);
      setIsInstalling(false);
    }
  };

  const skipSetup = async () => {
    await markSetupComplete();
  };

  const markSetupComplete = async () => {
    try {
      await invoke('mark_setup_complete');
      onComplete();
    } catch (err) {
      console.error('Error marking setup complete:', err);
      onComplete(); // Complete anyway
    }
  };

  const steps = [
    {
      title: 'Welcome to BEAR AI',
      description: 'State-of-the-art legal AI with privacy protection',
      icon: Shield,
    },
    {
      title: 'Privacy Protection',
      description: 'Configure Microsoft Presidio for PII detection',
      icon: Lock,
    },
    {
      title: 'AI Models',
      description: 'Select model size and download RAG embeddings',
      icon: Brain,
    },
    {
      title: 'Installation',
      description: 'Installing components',
      icon: Download,
    },
  ];

  const modelSizes = [
    {
      size: 'small',
      name: 'Corporate Laptop - Fast',
      description: 'TinyLlama 1.1B + BGE-Small (~850MB)',
      storage: '1 GB',
      ram: '4 GB minimum',
      icon: Zap,
      details: 'Perfect for standard corporate laptops',
      recommended: true,
    },
    {
      size: 'medium',
      name: 'Corporate Laptop - Balanced',
      description: 'Phi-2 + BGE-Small (~1.8GB)',
      storage: '2 GB',
      ram: '8 GB minimum',
      icon: Cpu,
      details: 'Better quality, still laptop-friendly',
    },
    {
      size: 'large',
      name: 'Workstation - Best Quality',
      description: 'Mistral-7B + BGE-Small (~4.6GB)',
      storage: '5 GB',
      ram: '16 GB minimum',
      icon: HardDrive,
      details: 'Highest quality, needs powerful hardware',
    },
  ];

  const renderStep = () => {
    switch (currentStep) {
      case 0:
        return (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="text-center space-y-6"
          >
            <div className="mx-auto w-24 h-24 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
              <Shield className="w-12 h-12 text-white" />
            </div>

            <div>
              <h2 className="text-3xl font-bold text-[var(--text-primary)] mb-2">Welcome to BEAR AI</h2>
              <p className="text-[var(--text-secondary)]">Your secure, private legal AI assistant</p>
            </div>

            <div className="bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-xl p-6 text-left space-y-4">
              <h3 className="text-xl font-semibold text-[var(--text-primary)]">Optional Components:</h3>

              <div className="space-y-3">
                <div className="flex items-start space-x-3">
                  <Lock className="w-5 h-5 text-blue-400 mt-0.5" />
                  <div>
                    <p className="text-[var(--text-primary)] font-medium">Microsoft Presidio</p>
                    <p className="text-sm text-[var(--text-secondary)]">State-of-the-art PII detection and redaction</p>
                  </div>
                </div>

                <div className="flex items-start space-x-3">
                  <Brain className="w-5 h-5 text-purple-400 mt-0.5" />
                  <div>
                    <p className="text-[var(--text-primary)] font-medium">AI Models</p>
                    <p className="text-sm text-[var(--text-secondary)]">Local LLMs for text generation</p>
                  </div>
                </div>

                <div className="flex items-start space-x-3">
                  <FileSearch className="w-5 h-5 text-green-400 mt-0.5" />
                  <div>
                    <p className="text-[var(--text-primary)] font-medium">RAG Engine</p>
                    <p className="text-sm text-[var(--text-secondary)]">Document embeddings for semantic search</p>
                  </div>
                </div>
              </div>

              <div className="pt-4 border-t border-[var(--border-primary)]">
                <p className="text-sm text-[var(--text-tertiary)] italic">
                  All components are optional. You can install them later from the settings.
                </p>
              </div>
            </div>

            <div className="flex gap-3 justify-center">
              <button
                onClick={skipSetup}
                className="px-8 py-3 bg-[var(--bg-secondary)] border border-[var(--border-primary)] text-[var(--text-primary)] rounded-lg font-medium hover:bg-[var(--hover-bg)] transition-all"
              >
                Skip Setup
              </button>
              <button
                onClick={() => setCurrentStep(1)}
                className="px-8 py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-lg font-medium hover:from-blue-600 hover:to-purple-700 transition-all"
              >
                Configure
              </button>
            </div>
          </motion.div>
        );

      case 1:
        return (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="space-y-6"
          >
            <div className="text-center">
              <h2 className="text-2xl font-bold text-[var(--text-primary)] mb-2">Component Selection</h2>
              <p className="text-[var(--text-secondary)]">Choose which components to install (all optional)</p>
            </div>

            <div className="space-y-4">
              <label className="flex items-center justify-between p-4 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg cursor-pointer hover:bg-[var(--hover-bg)] transition-colors">
                <div className="flex items-center space-x-3">
                  <Shield className="w-5 h-5 text-blue-400" />
                  <div>
                    <p className="text-[var(--text-primary)] font-medium">Install Microsoft Presidio</p>
                    <p className="text-sm text-[var(--text-secondary)]">Enterprise-grade PII detection</p>
                  </div>
                </div>
                <input
                  type="checkbox"
                  checked={setupConfig.install_presidio}
                  onChange={(e) => setSetupConfig({ ...setupConfig, install_presidio: e.target.checked })}
                  className="w-5 h-5 text-blue-500 rounded focus:ring-blue-500"
                />
              </label>

              <label className="flex items-center justify-between p-4 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg cursor-pointer hover:bg-[var(--hover-bg)] transition-colors">
                <div className="flex items-center space-x-3">
                  <Brain className="w-5 h-5 text-purple-400" />
                  <div>
                    <p className="text-[var(--text-primary)] font-medium">Download AI Models & RAG Engine</p>
                    <p className="text-sm text-[var(--text-secondary)]">LLMs and embeddings for document processing</p>
                  </div>
                </div>
                <input
                  type="checkbox"
                  checked={setupConfig.install_models}
                  onChange={(e) => setSetupConfig({ ...setupConfig, install_models: e.target.checked })}
                  className="w-5 h-5 text-blue-500 rounded focus:ring-blue-500"
                />
              </label>

              <label className="flex items-center justify-between p-4 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg cursor-pointer hover:bg-[var(--hover-bg)] transition-colors">
                <div className="flex items-center space-x-3">
                  <Zap className="w-5 h-5 text-yellow-400" />
                  <div>
                    <p className="text-[var(--text-primary)] font-medium">Enable GPU Acceleration</p>
                    <p className="text-sm text-[var(--text-secondary)]">Faster processing with NVIDIA GPU (optional)</p>
                  </div>
                </div>
                <input
                  type="checkbox"
                  checked={setupConfig.enable_gpu}
                  onChange={(e) => setSetupConfig({ ...setupConfig, enable_gpu: e.target.checked })}
                  className="w-5 h-5 text-blue-500 rounded focus:ring-blue-500"
                />
              </label>
            </div>

            <div className="flex justify-between gap-3">
              <button
                onClick={() => setCurrentStep(0)}
                className="px-6 py-2 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
              >
                Back
              </button>
              <div className="flex gap-3">
                <button
                  onClick={skipSetup}
                  className="px-6 py-2 bg-[var(--bg-secondary)] border border-[var(--border-primary)] text-[var(--text-primary)] rounded-lg hover:bg-[var(--hover-bg)] transition-all"
                >
                  Skip
                </button>
                <button
                  onClick={() => {
                    if (setupConfig.install_models) {
                      setCurrentStep(2);
                    } else {
                      setCurrentStep(3);
                      startSetup();
                    }
                  }}
                  className="px-8 py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-lg font-medium hover:from-blue-600 hover:to-purple-700 transition-all"
                >
                  {setupConfig.install_models ? 'Continue' : 'Install'}
                </button>
              </div>
            </div>
          </motion.div>
        );

      case 2:
        return (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="space-y-6"
          >
            <div className="text-center">
              <h2 className="text-2xl font-bold text-[var(--text-primary)] mb-2">Select Model Size</h2>
              <p className="text-[var(--text-secondary)]">Choose based on your system resources</p>
            </div>

            <div className="grid gap-4">
              {modelSizes.map((model) => (
                <button
                  key={model.size}
                  onClick={() => setSetupConfig({ ...setupConfig, model_size: model.size })}
                  className={`p-4 rounded-lg border-2 transition-all ${
                    setupConfig.model_size === model.size
                      ? 'border-blue-500 bg-blue-500/10'
                      : 'border-[var(--border-primary)] hover:border-[var(--accent)] bg-[var(--bg-secondary)]'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      <model.icon className={`w-6 h-6 ${
                        setupConfig.model_size === model.size ? 'text-blue-400' : 'text-[var(--text-secondary)]'
                      }`} />
                      <div className="text-left">
                        <div className="flex items-center space-x-2">
                          <p className="text-[var(--text-primary)] font-medium">{model.name}</p>
                          {model.recommended && (
                            <span className="px-2 py-0.5 bg-green-500/20 text-green-400 text-xs rounded-full">
                              Recommended
                            </span>
                          )}
                        </div>
                        <p className="text-sm text-[var(--text-secondary)]">{model.description}</p>
                      </div>
                    </div>
                    <div className="text-right text-sm">
                      <p className="text-[var(--text-secondary)]">Storage: {model.storage}</p>
                      <p className="text-[var(--text-secondary)]">RAM: {model.ram}</p>
                    </div>
                  </div>
                </button>
              ))}
            </div>

            <div className="flex justify-between gap-3">
              <button
                onClick={() => setCurrentStep(1)}
                className="px-6 py-2 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
              >
                Back
              </button>
              <div className="flex gap-3">
                <button
                  onClick={skipSetup}
                  className="px-6 py-2 bg-[var(--bg-secondary)] border border-[var(--border-primary)] text-[var(--text-primary)] rounded-lg hover:bg-[var(--hover-bg)] transition-all"
                >
                  Skip
                </button>
                <button
                  onClick={() => {
                    setCurrentStep(3);
                    startSetup();
                  }}
                  className="px-8 py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-lg font-medium hover:from-blue-600 hover:to-purple-700 transition-all"
                >
                  Start Installation
                </button>
              </div>
            </div>
          </motion.div>
        );

      case 3:
        return (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="space-y-6"
          >
            <div className="text-center">
              <h2 className="text-2xl font-bold text-[var(--text-primary)] mb-2">
                {progress?.is_complete ? 'Setup Complete!' : 'Installing Components'}
              </h2>
              <p className="text-[var(--text-secondary)]">
                {progress?.is_complete
                  ? 'BEAR AI is ready to use'
                  : 'This may take several minutes on first run'
                }
              </p>
            </div>

            {isInstalling && !progress?.is_complete && (
              <div className="space-y-4">
                <div className="bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg p-6">
                  <div className="flex items-center justify-between mb-4">
                    <p className="text-[var(--text-primary)] font-medium">{progress?.step || 'Initializing...'}</p>
                    <span className="text-sm text-[var(--text-secondary)]">{Math.round(progress?.progress || 0)}%</span>
                  </div>

                  <div className="w-full bg-[var(--hover-bg)] rounded-full h-2 mb-4">
                    <motion.div
                      className="bg-gradient-to-r from-blue-500 to-purple-600 h-2 rounded-full"
                      initial={{ width: 0 }}
                      animate={{ width: `${progress?.progress || 0}%` }}
                      transition={{ duration: 0.5 }}
                    />
                  </div>

                  <p className="text-sm text-[var(--text-secondary)]">{progress?.message}</p>
                </div>

                <div className="flex items-center justify-center">
                  <Loader2 className="w-6 h-6 text-blue-400 animate-spin" />
                </div>
              </div>
            )}

            {progress?.is_complete && (
              <motion.div
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                className="flex flex-col items-center space-y-4"
              >
                <div className="w-20 h-20 bg-green-500/20 rounded-full flex items-center justify-center">
                  <CheckCircle className="w-10 h-10 text-green-400" />
                </div>
                <p className="text-green-400 font-medium">All components installed successfully!</p>
              </motion.div>
            )}

            {error && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="bg-red-500/10 border border-red-500/30 rounded-lg p-4"
              >
                <div className="flex items-start space-x-3">
                  <AlertCircle className="w-5 h-5 text-red-400 mt-0.5" />
                  <div>
                    <p className="text-red-400 font-medium">Setup Error</p>
                    <p className="text-sm text-[var(--text-secondary)] mt-1">{error}</p>
                    <button
                      onClick={skipSetup}
                      className="mt-3 px-4 py-2 bg-[var(--bg-secondary)] border border-[var(--border-primary)] text-[var(--text-primary)] rounded-lg hover:bg-[var(--hover-bg)] transition-all text-sm"
                    >
                      Continue Without Installation
                    </button>
                  </div>
                </div>
              </motion.div>
            )}
          </motion.div>
        );

      default:
        return null;
    }
  };

  return (
    <div className="fixed inset-0 bg-[var(--bg-primary)] flex items-center justify-center z-50">
      <div className="w-full max-w-2xl p-8">
        {/* Theme Toggle */}
        <div className="flex justify-end mb-4">
          <button
            onClick={onThemeToggle}
            className="p-2 hover:bg-[var(--hover-bg)] rounded-lg transition-all border border-[var(--border-primary)]"
            title={theme === 'light' ? 'Switch to Dark Mode' : 'Switch to Light Mode'}
          >
            {theme === 'light' ? (
              <Moon className="w-5 h-5" />
            ) : (
              <Sun className="w-5 h-5" />
            )}
          </button>
        </div>

        <AnimatePresence mode="wait">
          {renderStep()}
        </AnimatePresence>
      </div>
    </div>
  );
};

export default SetupWizard;