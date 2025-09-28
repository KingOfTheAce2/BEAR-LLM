import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import {
  Send, Paperclip, Plus, Settings, Sun, Moon, User,
  Bot, Copy, RefreshCw, ChevronLeft, ChevronRight,
  Loader2, Check, AlertCircle, Sparkles
} from 'lucide-react';
import { useAppStore } from './stores/appStore';
import ChatMessage from './components/ChatMessage';
import ModelSelector from './components/ModelSelector';

function App() {
  const [message, setMessage] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [theme, setTheme] = useState<'light' | 'dark'>('dark');
  const [showNewChat, setShowNewChat] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const {
    messages,
    addMessage,
    clearMessages,
    selectedModel,
    conversations,
    addConversation,
    currentConversationId,
    setCurrentConversation
  } = useAppStore();

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('theme', theme);
  }, [theme]);

  useEffect(() => {
    const savedTheme = localStorage.getItem('theme') as 'light' | 'dark';
    if (savedTheme) setTheme(savedTheme);
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  useEffect(() => {
    adjustTextareaHeight();
  }, [message]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const adjustTextareaHeight = () => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  };

  const handleSendMessage = async () => {
    if (!message.trim() || isLoading) return;

    const userMessage = message.trim();
    setMessage('');
    setIsLoading(true);

    addMessage({
      role: 'user',
      content: userMessage,
      timestamp: Date.now(),
    });

    try {
      const response = await invoke<string>('send_message', {
        message: userMessage,
        modelName: selectedModel,
      });

      addMessage({
        role: 'assistant',
        content: response,
        timestamp: Date.now(),
      });
    } catch (error: any) {
      addMessage({
        role: 'system',
        content: `Error: ${error}`,
        timestamp: Date.now(),
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleFileUpload = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Documents',
          extensions: ['txt', 'pdf', 'docx', 'xlsx', 'csv', 'pptx', 'md', 'json']
        }]
      });

      if (selected) {
        setIsLoading(true);
        const fileName = selected.split(/[/\\]/).pop() || 'document';

        addMessage({
          role: 'system',
          content: `📎 Uploaded: ${fileName}`,
          timestamp: Date.now(),
        });

        const result = await invoke<any>('process_document', {
          filePath: selected,
          fileType: selected.split('.').pop()
        });

        addMessage({
          role: 'system',
          content: `✅ Document processed: ${fileName}`,
          timestamp: Date.now(),
        });
      }
    } catch (error) {
      addMessage({
        role: 'system',
        content: `Failed to process document: ${error}`,
        timestamp: Date.now(),
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleNewChat = () => {
    const newConversation = {
      id: Date.now().toString(),
      title: 'New Chat',
      timestamp: Date.now(),
      messages: []
    };
    addConversation(newConversation);
    setCurrentConversation(newConversation.id);
    clearMessages();
  };

  const toggleTheme = () => {
    setTheme(theme === 'light' ? 'dark' : 'light');
  };

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Sidebar */}
      <div className={`${sidebarOpen ? 'w-64' : 'w-0'} transition-all duration-300 bg-[var(--bg-secondary)] border-r border-[var(--border-primary)] flex flex-col overflow-hidden`}>
        <div className="p-4 border-b border-[var(--border-primary)]">
          <button
            onClick={handleNewChat}
            className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-[var(--bg-primary)] hover:bg-[var(--hover-bg)] border border-[var(--border-primary)] rounded-lg transition-all hover-lift"
          >
            <Plus className="w-4 h-4" />
            <span className="text-sm font-medium">New Chat</span>
          </button>
        </div>

        <div className="flex-1 overflow-y-auto scrollbar-custom p-3">
          <div className="space-y-1">
            {conversations.map((conv) => (
              <button
                key={conv.id}
                onClick={() => setCurrentConversation(conv.id)}
                className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-all ${
                  currentConversationId === conv.id
                    ? 'bg-[var(--hover-bg)] text-[var(--text-primary)]'
                    : 'text-[var(--text-secondary)] hover:bg-[var(--hover-bg)]'
                }`}
              >
                <div className="truncate">{conv.title}</div>
                <div className="text-xs text-[var(--text-tertiary)] mt-0.5">
                  {new Date(conv.timestamp).toLocaleDateString()}
                </div>
              </button>
            ))}
          </div>
        </div>

        <div className="p-4 border-t border-[var(--border-primary)] space-y-2">
          <ModelSelector />
          <button
            onClick={toggleTheme}
            className="w-full flex items-center justify-center gap-2 px-3 py-2 hover:bg-[var(--hover-bg)] rounded-lg transition-all"
          >
            {theme === 'light' ? (
              <>
                <Moon className="w-4 h-4" />
                <span className="text-sm">Dark Mode</span>
              </>
            ) : (
              <>
                <Sun className="w-4 h-4" />
                <span className="text-sm">Light Mode</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col bg-[var(--bg-primary)]">
        {/* Header */}
        <header className="h-14 border-b border-[var(--border-primary)] flex items-center justify-between px-4 bg-[var(--bg-primary)]">
          <div className="flex items-center gap-3">
            <button
              onClick={() => setSidebarOpen(!sidebarOpen)}
              className="p-2 hover:bg-[var(--hover-bg)] rounded-lg transition-all"
            >
              {sidebarOpen ? <ChevronLeft className="w-5 h-5" /> : <ChevronRight className="w-5 h-5" />}
            </button>
            <div className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                <Sparkles className="w-5 h-5 text-white" />
              </div>
              <span className="font-semibold text-lg">BEAR AI Assistant</span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs px-2 py-1 bg-[var(--bg-secondary)] text-[var(--text-secondary)] rounded-full">
              {selectedModel}
            </span>
          </div>
        </header>

        {/* Messages Area */}
        <div className="flex-1 overflow-y-auto scrollbar-custom">
          <div className="max-w-4xl mx-auto">
            {messages.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full p-8 text-center animate-fadeIn">
                <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center mb-6">
                  <Sparkles className="w-12 h-12 text-white" />
                </div>
                <h2 className="text-3xl font-bold mb-2 text-[var(--text-primary)]">
                  Welcome to BEAR AI
                </h2>
                <p className="text-[var(--text-secondary)] mb-8 max-w-md">
                  Your secure, private AI assistant for legal and professional tasks
                </p>
                <div className="grid grid-cols-2 gap-4 max-w-2xl">
                  <div className="p-4 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border-primary)] hover:border-[var(--accent)] transition-all cursor-pointer hover-lift">
                    <h3 className="font-medium mb-2">📚 Document Analysis</h3>
                    <p className="text-sm text-[var(--text-secondary)]">
                      Upload and analyze legal documents with AI
                    </p>
                  </div>
                  <div className="p-4 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border-primary)] hover:border-[var(--accent)] transition-all cursor-pointer hover-lift">
                    <h3 className="font-medium mb-2">🔒 Privacy First</h3>
                    <p className="text-sm text-[var(--text-secondary)]">
                      All processing happens locally on your device
                    </p>
                  </div>
                  <div className="p-4 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border-primary)] hover:border-[var(--accent)] transition-all cursor-pointer hover-lift">
                    <h3 className="font-medium mb-2">⚡ Fast Response</h3>
                    <p className="text-sm text-[var(--text-secondary)]">
                      Get instant AI-powered insights
                    </p>
                  </div>
                  <div className="p-4 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border-primary)] hover:border-[var(--accent)] transition-all cursor-pointer hover-lift">
                    <h3 className="font-medium mb-2">🎯 Specialized Models</h3>
                    <p className="text-sm text-[var(--text-secondary)]">
                      Choose from multiple AI models
                    </p>
                  </div>
                </div>
              </div>
            ) : (
              <div className="py-8">
                {messages.map((msg, index) => (
                  <ChatMessage key={index} message={msg} />
                ))}
                {isLoading && (
                  <div className="flex items-start gap-3 px-6 py-4 animate-fadeIn">
                    <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center flex-shrink-0">
                      <Bot className="w-5 h-5 text-white" />
                    </div>
                    <div className="flex items-center gap-2 px-4 py-2 bg-[var(--bg-secondary)] rounded-lg">
                      <Loader2 className="w-4 h-4 animate-spin" />
                      <span className="text-sm text-[var(--text-secondary)]">Thinking</span>
                      <span className="animate-pulse-dots"></span>
                    </div>
                  </div>
                )}
                <div ref={messagesEndRef} />
              </div>
            )}
          </div>
        </div>

        {/* Input Area */}
        <div className="border-t border-[var(--border-primary)] bg-[var(--bg-primary)]">
          <div className="max-w-4xl mx-auto p-4">
            <div className="flex items-end gap-3">
              <button
                onClick={handleFileUpload}
                className="p-2.5 hover:bg-[var(--hover-bg)] rounded-lg transition-all"
                disabled={isLoading}
              >
                <Paperclip className="w-5 h-5 text-[var(--text-secondary)]" />
              </button>

              <div className="flex-1 relative">
                <textarea
                  ref={textareaRef}
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      handleSendMessage();
                    }
                  }}
                  placeholder="Message BEAR AI..."
                  className="w-full px-4 py-3 bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg resize-none focus:outline-none focus:border-[var(--accent)] transition-all text-[var(--text-primary)] placeholder-[var(--text-tertiary)]"
                  rows={1}
                  disabled={isLoading}
                  style={{ minHeight: '48px', maxHeight: '200px' }}
                />
              </div>

              <button
                onClick={handleSendMessage}
                className={`p-2.5 rounded-lg transition-all ${
                  message.trim() && !isLoading
                    ? 'bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white'
                    : 'bg-[var(--bg-secondary)] text-[var(--text-tertiary)] cursor-not-allowed'
                }`}
                disabled={!message.trim() || isLoading}
              >
                {isLoading ? (
                  <Loader2 className="w-5 h-5 animate-spin" />
                ) : (
                  <Send className="w-5 h-5" />
                )}
              </button>
            </div>
            <div className="mt-2 text-xs text-center text-[var(--text-tertiary)]">
              BEAR AI uses local models. Your data never leaves your device.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;