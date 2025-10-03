import React, { useState, useEffect } from 'react';
import {
  Search, Download, Star, Users, HardDrive, Cpu, MemoryStick,
  Filter, SortAsc, SortDesc, CheckCircle, AlertCircle, Loader2,
  Globe, Shield, Laptop, ArrowUpDown, X, ExternalLink
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore, HuggingFaceModel } from '../stores/appStore';
import FloppyDiskIcon from './FloppyDiskIcon';
import { logger } from '../utils/logger';


interface HuggingFaceBrowserProps {
  isOpen: boolean;
  onClose: () => void;
  onModelSelect: (model: HuggingFaceModel) => void;
}

const HuggingFaceBrowser: React.FC<HuggingFaceBrowserProps> = ({
  isOpen,
  onClose,
  onModelSelect
}) => {
  const {
    corporateModels,
    hfSearchQuery,
    hfSearchResults,
    isSearchingHF,
    setHFSearchQuery,
    setHFSearchResults,
    setIsSearchingHF,
    updateModelDownloadProgress
  } = useAppStore();

  const [activeTab, setActiveTab] = useState<'corporate' | 'browse' | 'local'>('corporate');
  const [sortBy, setSortBy] = useState<'downloads' | 'likes' | 'name' | 'size'>('downloads');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [filterTags, setFilterTags] = useState<string[]>([]);
  const [performanceFilter, setPerformanceFilter] = useState<'all' | 'low' | 'medium' | 'high'>('all');

  // Sample HuggingFace models for demonstration
  const sampleHFModels: HuggingFaceModel[] = [
    {
      id: 'facebook/opt-350m',
      name: 'OPT-350M',
      author: 'Facebook',
      downloads: 1500000,
      likes: 250,
      tags: ['text-generation', 'causal-lm', 'facebook'],
      size: '700MB',
      description: 'Open Pre-trained Transformer 350M parameters',
      isLocal: false,
      isDownloading: false,
      systemRequirements: {
        minRam: '2GB',
        recommendedRam: '4GB',
        diskSpace: '800MB',
        performance: 'low'
      }
    },
    {
      id: 'google/flan-t5-small',
      name: 'FLAN-T5 Small',
      author: 'Google',
      downloads: 2200000,
      likes: 380,
      tags: ['text2text-generation', 'flan', 't5'],
      size: '242MB',
      description: 'Instruction-tuned T5 model for various tasks',
      isLocal: false,
      isDownloading: false,
      systemRequirements: {
        minRam: '2GB',
        recommendedRam: '4GB',
        diskSpace: '300MB',
        performance: 'low'
      }
    },
    {
      id: 'microsoft/CodeBERT-base',
      name: 'CodeBERT Base',
      author: 'Microsoft',
      downloads: 800000,
      likes: 160,
      tags: ['code', 'bert', 'programming'],
      size: '420MB',
      description: 'Pre-trained model for programming language understanding',
      isLocal: false,
      isDownloading: false,
      systemRequirements: {
        minRam: '3GB',
        recommendedRam: '6GB',
        diskSpace: '500MB',
        performance: 'medium'
      }
    },
    {
      id: 'sentence-transformers/all-MiniLM-L6-v2',
      name: 'MiniLM Sentence Transformer',
      author: 'Sentence Transformers',
      downloads: 15000000,
      likes: 750,
      tags: ['sentence-similarity', 'embeddings', 'miniLM'],
      size: '90MB',
      description: 'Efficient sentence embedding model',
      isLocal: false,
      isDownloading: false,
      systemRequirements: {
        minRam: '1GB',
        recommendedRam: '2GB',
        diskSpace: '120MB',
        performance: 'low'
      }
    }
  ];

  useEffect(() => {
    if (hfSearchResults.length === 0) {
      setHFSearchResults(sampleHFModels);
    }
  }, []);

  const handleSearch = async (query: string) => {
    setHFSearchQuery(query);
    if (!query.trim()) {
      setHFSearchResults(sampleHFModels);
      return;
    }

    setIsSearchingHF(true);

    try {
      // Try to call the backend search command
      const results = await invoke('search_huggingface_models', {
        query: query
      });

      // If backend returns results, use them
      if (results) {
        setHFSearchResults(results as HuggingFaceModel[]);
      } else {
        // Fallback to filtering sample models
        const filtered = sampleHFModels.filter(model =>
          model.name.toLowerCase().includes(query.toLowerCase()) ||
          model.author.toLowerCase().includes(query.toLowerCase()) ||
          model.tags.some(tag => tag.toLowerCase().includes(query.toLowerCase())) ||
          model.description?.toLowerCase().includes(query.toLowerCase())
        );
        setHFSearchResults(filtered);
      }

      setIsSearchingHF(false);
    } catch (error) {
      logger.warn('HuggingFace search failed, using local data', { error, query });
      // Fallback to sample models on error
      const filtered = sampleHFModels.filter(model =>
        model.name.toLowerCase().includes(query.toLowerCase()) ||
        model.author.toLowerCase().includes(query.toLowerCase()) ||
        model.tags.some(tag => tag.toLowerCase().includes(query.toLowerCase())) ||
        model.description?.toLowerCase().includes(query.toLowerCase())
      );
      setHFSearchResults(filtered);
      setIsSearchingHF(false);
    }
  };

  const handleDownloadModel = async (model: HuggingFaceModel) => {
    try {
      // Update UI to show downloading state
      updateModelDownloadProgress(model.id, 0);

      // Call the actual backend command to download the model
      logger.info('Downloading model', { modelId: model.id, modelName: model.name });

      // Create a models directory in the user's home folder
      const modelPath = `models/${model.id.replace('/', '_')}`;

      const result = await invoke('download_model_from_huggingface', {
        modelId: model.id
      });

      logger.info('Model download result', { result, modelId: model.id });

      // Mark as local after successful download
      const updatedModel = { ...model, isLocal: true, isDownloading: false };
      onModelSelect(updatedModel);

      // Update progress to 100%
      updateModelDownloadProgress(model.id, 100);

      // Add the downloaded model to local models list
      if (!localModels.includes(model.id)) {
        setLocalModels([...localModels, model.id]);
      }
    } catch (error) {
      logger.error('Failed to download model', error, { modelId: model.id, modelName: model.name });
      alert(`Failed to download model: ${error}`);
      updateModelDownloadProgress(model.id, 0);
    }
  };

  const getPerformanceColor = (performance: string) => {
    switch (performance) {
      case 'low': return 'text-green-500 bg-green-100 dark:bg-green-900/20';
      case 'medium': return 'text-yellow-500 bg-yellow-100 dark:bg-yellow-900/20';
      case 'high': return 'text-red-500 bg-red-100 dark:bg-red-900/20';
      default: return 'text-gray-500 bg-gray-100 dark:bg-gray-900/20';
    }
  };

  const getPerformanceIcon = (performance: string) => {
    switch (performance) {
      case 'low': return <Laptop className="w-3 h-3" />;
      case 'medium': return <Cpu className="w-3 h-3" />;
      case 'high': return <HardDrive className="w-3 h-3" />;
      default: return <AlertCircle className="w-3 h-3" />;
    }
  };

  const sortModels = (models: HuggingFaceModel[]) => {
    return [...models].sort((a, b) => {
      let comparison = 0;

      switch (sortBy) {
        case 'downloads':
          comparison = a.downloads - b.downloads;
          break;
        case 'likes':
          comparison = a.likes - b.likes;
          break;
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'size':
          const sizeA = parseInt(a.size?.replace(/[^\d]/g, '') || '0');
          const sizeB = parseInt(b.size?.replace(/[^\d]/g, '') || '0');
          comparison = sizeA - sizeB;
          break;
      }

      return sortOrder === 'desc' ? -comparison : comparison;
    });
  };

  const filterModels = (models: HuggingFaceModel[]) => {
    return models.filter(model => {
      const performanceMatch = performanceFilter === 'all' ||
        model.systemRequirements?.performance === performanceFilter;

      const tagMatch = filterTags.length === 0 ||
        filterTags.some(tag => model.tags.includes(tag));

      return performanceMatch && tagMatch;
    });
  };

  const ModelCard: React.FC<{ model: HuggingFaceModel }> = ({ model }) => (
    <div className="bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg p-4 hover:border-[var(--accent)] hover:shadow-md transition-all duration-300 transform hover:scale-[1.02]">
      <div className="flex items-start justify-between mb-3">
        <div className="flex-1">
          <h3 className="font-medium text-[var(--text-primary)] mb-1">{model.name}</h3>
          <p className="text-sm text-[var(--text-secondary)] mb-2">by {model.author}</p>
          {model.description && (
            <p className="text-xs text-[var(--text-tertiary)] mb-2 line-clamp-2">
              {model.description}
            </p>
          )}
        </div>
        <div className="flex items-center gap-2 ml-4">
          {model.isLocal ? (
            <button
              onClick={() => onModelSelect(model)}
              className="flex items-center gap-1 px-3 py-1.5 bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600 text-white rounded-lg text-xs transition-all duration-300 transform hover:scale-105 shadow-md"
            >
              <FloppyDiskIcon size="small" className="text-green-400" />
              Loaded
            </button>
          ) : model.isDownloading ? (
            <div className="flex items-center gap-1 px-3 py-1.5 bg-blue-500 text-white rounded text-xs">
              <Loader2 className="w-3 h-3 animate-spin" />
              {model.downloadProgress}%
            </div>
          ) : (
            <button
              onClick={() => handleDownloadModel(model)}
              className="flex items-center gap-1 px-3 py-1.5 bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600 text-white rounded-lg text-xs transition-all duration-300 transform hover:scale-105 shadow-md"
            >
              <FloppyDiskIcon size="small" isAnimated={true} />
              Insert Model
            </button>
          )}
        </div>
      </div>

      <div className="flex items-center gap-4 text-xs text-[var(--text-tertiary)] mb-3">
        <div className="flex items-center gap-1">
          <Users className="w-3 h-3" />
          {(model.downloads / 1000000).toFixed(1)}M downloads
        </div>
        <div className="flex items-center gap-1">
          <Star className="w-3 h-3" />
          {model.likes}
        </div>
        <div className="flex items-center gap-1">
          <HardDrive className="w-3 h-3" />
          {model.size}
        </div>
      </div>

      {model.systemRequirements && (
        <div className="flex items-center justify-between text-xs mb-3">
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1">
              <MemoryStick className="w-3 h-3" />
              <span>{model.systemRequirements.recommendedRam}</span>
            </div>
            <div className={`flex items-center gap-1 px-2 py-1 rounded-full ${getPerformanceColor(model.systemRequirements.performance)}`}>
              {getPerformanceIcon(model.systemRequirements.performance)}
              <span className="capitalize">{model.systemRequirements.performance}</span>
            </div>
          </div>
        </div>
      )}

      <div className="flex flex-wrap gap-1">
        {model.tags.slice(0, 3).map(tag => (
          <span
            key={tag}
            className="px-2 py-1 bg-[var(--bg-primary)] text-[var(--text-tertiary)] text-xs rounded-full border border-[var(--border-primary)] hover:border-[var(--accent)] transition-colors"
          >
            {tag}
          </span>
        ))}
        {model.tags.length > 3 && (
          <span className="text-xs text-[var(--text-tertiary)]">
            +{model.tags.length - 3} more
          </span>
        )}
      </div>
    </div>
  );

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-md z-50 flex items-center justify-center p-4 animate-fadeIn">
      <div className="bg-[var(--bg-primary)] border border-[var(--border-primary)] rounded-2xl shadow-2xl backdrop-blur-lg max-w-6xl w-full max-h-[90vh] overflow-hidden animate-slideUp">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-[var(--border-primary)]">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-gradient-to-br from-purple-500 to-pink-600 rounded-lg flex items-center justify-center">
              <FloppyDiskIcon size="small" theme="dark" isAnimated={true} />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-[var(--text-primary)]">
                AI Model Vault
              </h2>
              <p className="text-sm text-[var(--text-secondary)]">
                Insert the perfect AI model into your BEAR system
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-[var(--hover-bg)] rounded-lg transition-all"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-[var(--border-primary)]">
          <button
            onClick={() => setActiveTab('corporate')}
            className={`px-4 py-3 text-sm font-medium transition-colors ${
              activeTab === 'corporate'
                ? 'border-b-2 border-[var(--accent)] text-[var(--accent)]'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
            }`}
          >
            <div className="flex items-center gap-2">
              <Laptop className="w-4 h-4" />
              Corporate Optimized ({corporateModels.length})
            </div>
          </button>
          <button
            onClick={() => setActiveTab('browse')}
            className={`px-4 py-3 text-sm font-medium transition-colors ${
              activeTab === 'browse'
                ? 'border-b-2 border-[var(--accent)] text-[var(--accent)]'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
            }`}
          >
            <div className="flex items-center gap-2">
              <FloppyDiskIcon size="small" />
              Model Library ({hfSearchResults.length})
            </div>
          </button>
          <button
            onClick={() => setActiveTab('local')}
            className={`px-4 py-3 text-sm font-medium transition-colors ${
              activeTab === 'local'
                ? 'border-b-2 border-[var(--accent)] text-[var(--accent)]'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
            }`}
          >
            <div className="flex items-center gap-2">
              <Shield className="w-4 h-4" />
              Local Models
            </div>
          </button>
        </div>

        {/* Search and Filters */}
        {activeTab === 'browse' && (
          <div className="p-4 border-b border-[var(--border-primary)] space-y-3">
            <div className="flex gap-3">
              <div className="flex-1 relative">
                <Search className="w-4 h-4 absolute left-3 top-3 text-[var(--text-tertiary)]" />
                <input
                  type="text"
                  value={hfSearchQuery}
                  onChange={(e) => handleSearch(e.target.value)}
                  placeholder="Search for the perfect AI model..."
                  className="w-full pl-10 pr-4 py-2.5 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-xl focus:outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/20 text-[var(--text-primary)] transition-all"
                />
                {isSearchingHF && (
                  <Loader2 className="w-4 h-4 absolute right-3 top-3 animate-spin text-[var(--text-tertiary)]" />
                )}
              </div>

              <div className="flex gap-2">
                <select
                  value={sortBy}
                  onChange={(e) => setSortBy(e.target.value as any)}
                  className="px-3 py-2 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-xl text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-all"
                >
                  <option value="downloads">Sort by Downloads</option>
                  <option value="likes">Sort by Likes</option>
                  <option value="name">Sort by Name</option>
                  <option value="size">Sort by Size</option>
                </select>

                <button
                  onClick={() => setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')}
                  className="p-2 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-xl hover:bg-[var(--hover-bg)] hover:border-[var(--accent)] transition-all duration-200"
                >
                  {sortOrder === 'asc' ? <SortAsc className="w-4 h-4" /> : <SortDesc className="w-4 h-4" />}
                </button>
              </div>
            </div>

            <div className="flex gap-3">
              <select
                value={performanceFilter}
                onChange={(e) => setPerformanceFilter(e.target.value as any)}
                className="px-3 py-1.5 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-xl text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-all"
              >
                <option value="all">All Performance Levels</option>
                <option value="low">Low (Basic Laptops)</option>
                <option value="medium">Medium (Standard Laptops)</option>
                <option value="high">High (Powerful Workstations)</option>
              </select>
            </div>
          </div>
        )}

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4 max-h-[60vh]">
          {activeTab === 'corporate' && (
            <div className="space-y-4">
              <div className="mb-4 p-4 bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20 border border-blue-200 dark:border-purple-800 rounded-xl backdrop-blur-sm">
                <div className="flex items-center gap-2 text-blue-700 dark:text-blue-300">
                  <Laptop className="w-4 h-4" />
                  <span className="text-sm font-medium">Corporate Laptop Optimized</span>
                </div>
                <p className="text-xs text-blue-600 dark:text-blue-400 mt-1">
                  These models are specifically chosen for corporate environments with memory and performance constraints.
                </p>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                {filterModels(sortModels(corporateModels)).map(model => (
                  <ModelCard key={model.id} model={model} />
                ))}
              </div>
            </div>
          )}

          {activeTab === 'browse' && (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              {filterModels(sortModels(hfSearchResults)).map(model => (
                <ModelCard key={model.id} model={model} />
              ))}
            </div>
          )}

          {activeTab === 'local' && (
            <div className="space-y-4">
              <div className="text-center py-8">
                <Shield className="w-12 h-12 text-[var(--text-tertiary)] mx-auto mb-3" />
                <h3 className="text-lg font-medium text-[var(--text-primary)] mb-2">
                  Local Models
                </h3>
                <p className="text-[var(--text-secondary)] mb-4">
                  Downloaded models will appear here. All processing happens locally for maximum privacy.
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-[var(--border-primary)] bg-[var(--bg-secondary)]">
          <div className="flex items-center justify-between text-xs text-[var(--text-tertiary)]">
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-1">
                <Shield className="w-3 h-3" />
                <span>Privacy: All models run locally</span>
              </div>
              <div className="flex items-center gap-1">
                <ExternalLink className="w-3 h-3" />
                <span>Source: HuggingFace Hub</span>
              </div>
            </div>
            <span>Powered by BEAR AI • Insert the Future</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default HuggingFaceBrowser;