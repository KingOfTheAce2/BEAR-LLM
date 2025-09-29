import { create } from 'zustand';

interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

interface Document {
  id: string;
  filename: string;
  content: string;
  pii_removed: boolean;
  metadata: any;
}

interface Conversation {
  id: string;
  title: string;
  timestamp: number;
  messages: Message[];
}

export interface HuggingFaceModel {
  id: string;
  name: string;
  author: string;
  downloads: number;
  likes: number;
  tags: string[];
  size?: string;
  description?: string;
  isLocal: boolean;
  isDownloading: boolean;
  downloadProgress?: number;
  systemRequirements?: {
    minRam: string;
    recommendedRam: string;
    diskSpace: string;
    performance: 'low' | 'medium' | 'high';
  };
}

interface AppStore {
  messages: Message[];
  documents: Document[];
  selectedModel: string;
  availableModels: string[];
  huggingFaceModels: HuggingFaceModel[];
  corporateModels: HuggingFaceModel[];
  isProcessing: boolean;
  systemHealth: any;
  conversations: Conversation[];
  currentConversationId: string;
  hfSearchQuery: string;
  hfSearchResults: HuggingFaceModel[];
  isSearchingHF: boolean;

  addMessage: (message: Message) => void;
  clearMessages: () => void;
  addDocument: (document: Document) => void;
  setSelectedModel: (model: string) => void;
  setAvailableModels: (models: string[]) => void;
  setHuggingFaceModels: (models: HuggingFaceModel[]) => void;
  setCorporateModels: (models: HuggingFaceModel[]) => void;
  setProcessing: (processing: boolean) => void;
  setSystemHealth: (health: any) => void;
  addConversation: (conversation: Conversation) => void;
  setCurrentConversation: (id: string) => void;
  setHFSearchQuery: (query: string) => void;
  setHFSearchResults: (results: HuggingFaceModel[]) => void;
  setIsSearchingHF: (searching: boolean) => void;
  updateModelDownloadProgress: (modelId: string, progress: number) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  messages: [],
  documents: [],
  selectedModel: 'microsoft/DialoGPT-medium',
  availableModels: ['microsoft/DialoGPT-medium', 'microsoft/DialoGPT-small', 'distilbert-base-uncased', 'TinyLlama/TinyLlama-1.1B-Chat-v1.0', 'microsoft/phi-2'],
  huggingFaceModels: [],
  corporateModels: [
    {
      id: 'microsoft/DialoGPT-medium',
      name: 'DialoGPT Medium',
      author: 'Microsoft',
      downloads: 2500000,
      likes: 450,
      tags: ['conversational', 'text-generation', 'corporate-friendly'],
      size: '355MB',
      description: 'Corporate-optimized conversational AI model',
      isLocal: true,
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
      description: 'Lightweight model for basic corporate laptops',
      isLocal: true,
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
      description: 'Efficient chat model for resource-constrained environments',
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
      description: 'Small language model with strong reasoning capabilities',
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
      description: 'Lightweight BERT model for text understanding',
      isLocal: true,
      isDownloading: false,
      systemRequirements: {
        minRam: '2GB',
        recommendedRam: '4GB',
        diskSpace: '300MB',
        performance: 'low'
      }
    }
  ],
  hfSearchQuery: '',
  hfSearchResults: [],
  isSearchingHF: false,
  isProcessing: false,
  systemHealth: null,
  conversations: [{
    id: '1',
    title: 'New Chat',
    timestamp: Date.now(),
    messages: []
  }],
  currentConversationId: '1',

  addMessage: (message) =>
    set((state) => ({ messages: [...state.messages, message] })),

  clearMessages: () => set({ messages: [] }),

  addDocument: (document) =>
    set((state) => ({ documents: [...state.documents, document] })),

  setSelectedModel: (model) => set({ selectedModel: model }),

  setAvailableModels: (models) => set({ availableModels: models }),

  setHuggingFaceModels: (models) => set({ huggingFaceModels: models }),

  setCorporateModels: (models) => set({ corporateModels: models }),

  setProcessing: (processing) => set({ isProcessing: processing }),

  setHFSearchQuery: (query) => set({ hfSearchQuery: query }),

  setHFSearchResults: (results) => set({ hfSearchResults: results }),

  setIsSearchingHF: (searching) => set({ isSearchingHF: searching }),

  updateModelDownloadProgress: (modelId, progress) =>
    set((state) => ({
      corporateModels: state.corporateModels.map(model =>
        model.id === modelId ? { ...model, downloadProgress: progress, isDownloading: progress < 100 } : model
      ),
      huggingFaceModels: state.huggingFaceModels.map(model =>
        model.id === modelId ? { ...model, downloadProgress: progress, isDownloading: progress < 100 } : model
      )
    })),

  setSystemHealth: (health) => set({ systemHealth: health }),

  addConversation: (conversation) =>
    set((state) => ({ conversations: [...state.conversations, conversation] })),

  setCurrentConversation: (id) => set({ currentConversationId: id }),
}));