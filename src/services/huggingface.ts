import { invoke } from '@tauri-apps/api/core';
import { HuggingFaceModel } from '../stores/appStore';
import { logger } from '../utils/logger';

/**
 * HuggingFace API integration service
 * Provides local browsing and downloading of HuggingFace models
 */

export interface HFSearchOptions {
  query?: string;
  filter?: string;
  author?: string;
  task?: string;
  sort?: 'downloads' | 'likes' | 'createdAt' | 'lastModified';
  direction?: 'asc' | 'desc';
  limit?: number;
  full?: boolean;
}

export interface HFModelDownloadOptions {
  modelId: string;
  revision?: string;
  cache_dir?: string;
  force_download?: boolean;
  resume_download?: boolean;
  proxies?: Record<string, string>;
  token?: string;
}

class HuggingFaceService {
  private readonly baseUrl = 'https://huggingface.co/api';

  /**
   * Search HuggingFace models with filters
   */
  async searchModels(options: HFSearchOptions = {}): Promise<HuggingFaceModel[]> {
    try {
      // Try backend search first
      const result = await invoke<HuggingFaceModel[]>('search_huggingface_models', {
        options
      });
      return result;
    } catch (error) {
      logger.warn('Backend search failed, using fallback', { error, options });
      return this.fallbackSearch(options);
    }
  }

  /**
   * Get detailed information about a specific model
   */
  async getModelInfo(modelId: string): Promise<HuggingFaceModel | null> {
    try {
      const result = await invoke<HuggingFaceModel>('get_huggingface_model_info', {
        modelId
      });
      return result;
    } catch (error) {
      logger.warn('Failed to get model info', { modelId, error });
      return null;
    }
  }

  /**
   * Download a model from HuggingFace
   */
  async downloadModel(
    modelId: string,
    options: HFModelDownloadOptions = {},
    onProgress?: (progress: number) => void
  ): Promise<boolean> {
    try {
      // Start download with progress callback
      const result = await invoke<boolean>('download_huggingface_model', {
        modelId,
        options,
        progressCallback: onProgress ? 'hf_download_progress' : null
      });

      return result;
    } catch (error) {
      logger.error('Model download failed', error, { modelId, options });
      throw new Error(`Failed to download model ${modelId}: ${error}`);
    }
  }

  /**
   * List locally available models
   */
  async listLocalModels(): Promise<HuggingFaceModel[]> {
    try {
      const result = await invoke<HuggingFaceModel[]>('list_local_models');
      return result;
    } catch (error) {
      logger.warn('Failed to list local models', { error });
      return [];
    }
  }

  /**
   * Delete a local model
   */
  async deleteLocalModel(modelId: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>('delete_local_model', {
        modelId
      });
      return result;
    } catch (error) {
      logger.error('Failed to delete local model', error, { modelId });
      return false;
    }
  }

  /**
   * Get corporate-optimized model recommendations
   */
  getCorporateRecommendations(): HuggingFaceModel[] {
    return [
      {
        id: 'microsoft/DialoGPT-medium',
        name: 'DialoGPT Medium',
        author: 'Microsoft',
        downloads: 2500000,
        likes: 450,
        tags: ['conversational', 'text-generation', 'corporate-friendly'],
        size: '355MB',
        description: 'Corporate-optimized conversational AI model designed for business environments',
        isLocal: false,
        isDownloading: false,
        systemRequirements: {
          minRam: '4GB',
          recommendedRam: '8GB',
          diskSpace: '400MB',
          performance: 'medium'
        }
      },
      {
        id: 'microsoft/DialoGPT-small',
        name: 'DialoGPT Small',
        author: 'Microsoft',
        downloads: 1800000,
        likes: 320,
        tags: ['conversational', 'text-generation', 'lightweight'],
        size: '115MB',
        description: 'Lightweight conversational model perfect for basic corporate laptops',
        isLocal: false,
        isDownloading: false,
        systemRequirements: {
          minRam: '2GB',
          recommendedRam: '4GB',
          diskSpace: '150MB',
          performance: 'low'
        }
      },
      {
        id: 'TinyLlama/TinyLlama-1.1B-Chat-v1.0',
        name: 'TinyLlama 1.1B Chat',
        author: 'TinyLlama',
        downloads: 950000,
        likes: 180,
        tags: ['chat', 'efficient', 'small-model'],
        size: '2.2GB',
        description: 'Efficient chat model designed for resource-constrained corporate environments',
        isLocal: false,
        isDownloading: false,
        systemRequirements: {
          minRam: '3GB',
          recommendedRam: '6GB',
          diskSpace: '2.5GB',
          performance: 'medium'
        }
      },
      {
        id: 'microsoft/phi-2',
        name: 'Phi-2',
        author: 'Microsoft',
        downloads: 1200000,
        likes: 290,
        tags: ['small-model', 'efficient', 'reasoning'],
        size: '2.7GB',
        description: 'Small language model with strong reasoning capabilities for corporate tasks',
        isLocal: false,
        isDownloading: false,
        systemRequirements: {
          minRam: '4GB',
          recommendedRam: '8GB',
          diskSpace: '3GB',
          performance: 'medium'
        }
      },
      {
        id: 'distilbert-base-uncased',
        name: 'DistilBERT Base',
        author: 'Hugging Face',
        downloads: 8500000,
        likes: 680,
        tags: ['bert', 'efficient', 'nlp'],
        size: '255MB',
        description: 'Lightweight BERT model optimized for text understanding and corporate NLP tasks',
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
        id: 'sentence-transformers/all-MiniLM-L6-v2',
        name: 'MiniLM Sentence Transformer',
        author: 'Sentence Transformers',
        downloads: 15000000,
        likes: 750,
        tags: ['sentence-similarity', 'embeddings', 'miniLM'],
        size: '90MB',
        description: 'Ultra-efficient sentence embedding model for document similarity and search',
        isLocal: false,
        isDownloading: false,
        systemRequirements: {
          minRam: '1GB',
          recommendedRam: '2GB',
          diskSpace: '120MB',
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
        description: 'Pre-trained model for programming language understanding and code analysis',
        isLocal: false,
        isDownloading: false,
        systemRequirements: {
          minRam: '3GB',
          recommendedRam: '6GB',
          diskSpace: '500MB',
          performance: 'medium'
        }
      }
    ];
  }

  /**
   * Fallback search when backend is not available
   */
  private fallbackSearch(options: HFSearchOptions): HuggingFaceModel[] {
    const corporateModels = this.getCorporateRecommendations();

    if (!options.query) {
      return corporateModels;
    }

    const query = options.query.toLowerCase();
    return corporateModels.filter(model =>
      model.name.toLowerCase().includes(query) ||
      model.author.toLowerCase().includes(query) ||
      model.tags.some(tag => tag.toLowerCase().includes(query)) ||
      model.description?.toLowerCase().includes(query)
    );
  }

  /**
   * Check if a model is suitable for corporate laptops
   */
  isCorporateOptimized(model: HuggingFaceModel): boolean {
    if (!model.systemRequirements) return false;

    const ramMB = this.parseMemorySize(model.systemRequirements.recommendedRam);
    const diskMB = this.parseMemorySize(model.systemRequirements.diskSpace);

    // Corporate laptop optimized criteria:
    // - RAM requirement <= 8GB
    // - Disk space <= 5GB
    // - Performance level 'low' or 'medium'
    return (
      ramMB <= 8192 &&
      diskMB <= 5120 &&
      ['low', 'medium'].includes(model.systemRequirements.performance)
    );
  }

  /**
   * Parse memory size strings (e.g., "8GB", "500MB") to MB
   */
  private parseMemorySize(sizeStr: string): number {
    const match = sizeStr.match(/(\d+(?:\.\d+)?)\s*(GB|MB)/i);
    if (!match) return 0;

    const value = parseFloat(match[1]);
    const unit = match[2].toUpperCase();

    return unit === 'GB' ? value * 1024 : value;
  }

  /**
   * Estimate model requirements based on size
   */
  estimateRequirements(modelSize: string): HuggingFaceModel['systemRequirements'] {
    const sizeMB = this.parseMemorySize(modelSize);

    if (sizeMB < 200) {
      return {
        minRam: '1GB',
        recommendedRam: '2GB',
        diskSpace: modelSize,
        performance: 'low'
      };
    } else if (sizeMB < 1000) {
      return {
        minRam: '2GB',
        recommendedRam: '4GB',
        diskSpace: modelSize,
        performance: 'low'
      };
    } else if (sizeMB < 3000) {
      return {
        minRam: '4GB',
        recommendedRam: '8GB',
        diskSpace: modelSize,
        performance: 'medium'
      };
    } else {
      return {
        minRam: '8GB',
        recommendedRam: '16GB',
        diskSpace: modelSize,
        performance: 'high'
      };
    }
  }
}

// Export singleton instance
export const huggingFaceService = new HuggingFaceService();
export default huggingFaceService;