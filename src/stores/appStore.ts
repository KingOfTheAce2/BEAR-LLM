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

interface AppStore {
  messages: Message[];
  documents: Document[];
  selectedModel: string;
  availableModels: string[];
  isProcessing: boolean;
  systemHealth: any;
  conversations: Conversation[];
  currentConversationId: string;

  addMessage: (message: Message) => void;
  clearMessages: () => void;
  addDocument: (document: Document) => void;
  setSelectedModel: (model: string) => void;
  setAvailableModels: (models: string[]) => void;
  setProcessing: (processing: boolean) => void;
  setSystemHealth: (health: any) => void;
  addConversation: (conversation: Conversation) => void;
  setCurrentConversation: (id: string) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  messages: [],
  documents: [],
  selectedModel: 'llama2-7b',
  availableModels: ['llama2-7b', 'mistral-7b', 'phi-2'],
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

  setProcessing: (processing) => set({ isProcessing: processing }),

  setSystemHealth: (health) => set({ systemHealth: health }),

  addConversation: (conversation) =>
    set((state) => ({ conversations: [...state.conversations, conversation] })),

  setCurrentConversation: (id) => set({ currentConversationId: id }),
}));