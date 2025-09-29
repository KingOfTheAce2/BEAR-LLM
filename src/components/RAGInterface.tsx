import React, { useState, useRef } from 'react';
import { Search, Upload, Brain, FileText, Database, Zap, Loader2, CheckCircle, AlertTriangle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface RAGResult {
  answer: string;
  sources: {
    title: string;
    snippet: string;
    relevance: number;
    source: string;
  }[];
  reasoning?: string;
  confidence: number;
}

interface DocumentStatus {
  filename: string;
  status: 'processing' | 'indexed' | 'error';
  chunks?: number;
  error?: string;
}

const RAGInterface: React.FC = () => {
  const [query, setQuery] = useState('');
  const [result, setResult] = useState<RAGResult | null>(null);
  const [isSearching, setIsSearching] = useState(false);
  const [uploadedDocs, setUploadedDocs] = useState<DocumentStatus[]>([]);
  const [isUploading, setIsUploading] = useState(false);
  const [useAgenticRag, setUseAgenticRag] = useState(true);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleSearch = async () => {
    if (!query.trim()) return;

    setIsSearching(true);
    try {
      const result = await invoke<RAGResult>('rag_search', {
        query: query.trim(),
        useAgentic: useAgenticRag,
        maxResults: 5
      });

      setResult(result);
    } catch (error) {
      console.error('RAG search failed:', error);
      setResult({
        answer: 'Sorry, I encountered an error while searching the knowledge base.',
        sources: [],
        confidence: 0
      });
    } finally {
      setIsSearching(false);
    }
  };

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files || []);
    if (files.length === 0) return;

    setIsUploading(true);

    for (const file of files) {
      const docStatus: DocumentStatus = {
        filename: file.name,
        status: 'processing'
      };

      setUploadedDocs(prev => [...prev, docStatus]);

      try {
        const result = await invoke<{ chunks: number }>('upload_document', {
          filename: file.name,
          content: await file.arrayBuffer()
        });

        setUploadedDocs(prev =>
          prev.map(doc =>
            doc.filename === file.name
              ? { ...doc, status: 'indexed', chunks: result.chunks }
              : doc
          )
        );
      } catch (error) {
        setUploadedDocs(prev =>
          prev.map(doc =>
            doc.filename === file.name
              ? { ...doc, status: 'error', error: error instanceof Error ? error.message : 'Upload failed' }
              : doc
          )
        );
      }
    }

    setIsUploading(false);
  };

  const getRelevanceColor = (relevance: number) => {
    if (relevance >= 0.8) return 'text-green-600';
    if (relevance >= 0.6) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300';
    if (confidence >= 0.6) return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300';
    return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300';
  };

  const sampleQueries = [
    "What are the key terms in my employment contracts?",
    "Find all documents related to intellectual property rights",
    "Summarize the liability clauses across all agreements",
    "What are the termination conditions in the service agreements?",
    "Search for any PII data that needs to be redacted"
  ];

  return (
    <div className="bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-2">
          <Brain className="w-5 h-5 text-[var(--accent)]" />
          <h3 className="text-lg font-semibold text-[var(--text-primary)]">
            RAG Knowledge Search
          </h3>
        </div>

        <div className="flex items-center gap-4">
          <label className="flex items-center gap-2 text-sm text-[var(--text-secondary)]">
            <input
              type="checkbox"
              checked={useAgenticRag}
              onChange={(e) => setUseAgenticRag(e.target.checked)}
              className="rounded border-[var(--border-primary)]"
            />
            <Zap className="w-4 h-4" />
            AgenticRAG
          </label>

          <input
            type="file"
            ref={fileInputRef}
            onChange={handleFileUpload}
            multiple
            accept=".pdf,.docx,.doc,.txt,.md,.json,.csv,.xml,.html"
            className="hidden"
          />
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={isUploading}
            className="flex items-center gap-2 px-3 py-2 bg-[var(--accent)] hover:bg-[var(--accent-hover)] disabled:opacity-50 text-white rounded-lg transition-colors text-sm"
          >
            <Upload className="w-4 h-4" />
            {isUploading ? 'Uploading...' : 'Upload Docs'}
          </button>
        </div>
      </div>

      {/* Document Status */}
      {uploadedDocs.length > 0 && (
        <div className="mb-6 p-4 bg-[var(--bg-tertiary)] rounded-lg border border-[var(--border-secondary)]">
          <h4 className="text-sm font-medium text-[var(--text-primary)] mb-2">Document Index Status</h4>
          <div className="space-y-2 max-h-32 overflow-y-auto">
            {uploadedDocs.map((doc, index) => (
              <div key={index} className="flex items-center justify-between text-xs">
                <span className="text-[var(--text-secondary)] truncate">{doc.filename}</span>
                <div className="flex items-center gap-2">
                  {doc.status === 'processing' && (
                    <Loader2 className="w-3 h-3 animate-spin text-blue-500" />
                  )}
                  {doc.status === 'indexed' && (
                    <>
                      <CheckCircle className="w-3 h-3 text-green-500" />
                      <span className="text-green-600">{doc.chunks} chunks</span>
                    </>
                  )}
                  {doc.status === 'error' && (
                    <>
                      <AlertTriangle className="w-3 h-3 text-red-500" />
                      <span className="text-red-600">{doc.error}</span>
                    </>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Sample Queries */}
      <div className="mb-4">
        <h4 className="text-sm font-medium text-[var(--text-primary)] mb-2">Sample Queries</h4>
        <div className="flex flex-wrap gap-2">
          {sampleQueries.map((sampleQuery, index) => (
            <button
              key={index}
              onClick={() => setQuery(sampleQuery)}
              className="px-3 py-1 text-xs bg-[var(--bg-tertiary)] hover:bg-[var(--hover-bg)] border border-[var(--border-secondary)] rounded transition-colors text-[var(--text-secondary)]"
            >
              {sampleQuery.slice(0, 30)}...
            </button>
          ))}
        </div>
      </div>

      {/* Search Input */}
      <div className="mb-4">
        <div className="flex gap-2">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-[var(--text-tertiary)]" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
              placeholder="Ask a question about your documents..."
              className="w-full pl-10 pr-4 py-3 bg-[var(--bg-primary)] border border-[var(--border-primary)] rounded-lg text-[var(--text-primary)] placeholder-[var(--text-tertiary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            />
          </div>
          <button
            onClick={handleSearch}
            disabled={isSearching || !query.trim()}
            className="px-6 py-3 bg-[var(--accent)] hover:bg-[var(--accent-hover)] disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
          >
            {isSearching ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              'Search'
            )}
          </button>
        </div>
      </div>

      {/* AgenticRAG Info */}
      {useAgenticRag && (
        <div className="mb-4 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
          <div className="flex items-center gap-2">
            <Zap className="w-4 h-4 text-blue-500" />
            <p className="text-xs text-blue-700 dark:text-blue-400">
              <strong>AgenticRAG Mode:</strong> Using advanced reasoning, query rewriting, and result reranking for enhanced accuracy.
            </p>
          </div>
        </div>
      )}

      {/* Results */}
      {result && (
        <div className="space-y-4">
          {/* Answer */}
          <div className="p-4 bg-[var(--bg-primary)] border border-[var(--border-primary)] rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <h4 className="text-sm font-medium text-[var(--text-primary)]">Answer</h4>
              <span className={`px-2 py-1 text-xs rounded-full ${getConfidenceColor(result.confidence)}`}>
                {Math.round(result.confidence * 100)}% confidence
              </span>
            </div>
            <p className="text-[var(--text-secondary)] leading-relaxed">{result.answer}</p>
          </div>

          {/* AgenticRAG Reasoning */}
          {result.reasoning && (
            <div className="p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Brain className="w-4 h-4 text-blue-500" />
                <h4 className="text-sm font-medium text-blue-700 dark:text-blue-300">Reasoning Process</h4>
              </div>
              <p className="text-xs text-blue-600 dark:text-blue-400">{result.reasoning}</p>
            </div>
          )}

          {/* Sources */}
          {result.sources.length > 0 && (
            <div className="space-y-3">
              <h4 className="text-sm font-medium text-[var(--text-primary)] flex items-center gap-2">
                <FileText className="w-4 h-4" />
                Sources ({result.sources.length})
              </h4>
              {result.sources.map((source, index) => (
                <div
                  key={index}
                  className="p-3 bg-[var(--bg-tertiary)] border border-[var(--border-secondary)] rounded-lg"
                >
                  <div className="flex items-start justify-between mb-2">
                    <h5 className="text-sm font-medium text-[var(--text-primary)] truncate">
                      {source.title}
                    </h5>
                    <span className={`text-xs font-mono ${getRelevanceColor(source.relevance)}`}>
                      {Math.round(source.relevance * 100)}%
                    </span>
                  </div>
                  <p className="text-xs text-[var(--text-secondary)] leading-relaxed mb-2">
                    {source.snippet}
                  </p>
                  <div className="flex items-center gap-2">
                    <Database className="w-3 h-3 text-[var(--text-tertiary)]" />
                    <span className="text-xs text-[var(--text-tertiary)]">{source.source}</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Instructions */}
      <div className="mt-6 p-3 bg-gray-50 dark:bg-gray-900/20 border border-gray-200 dark:border-gray-800 rounded-lg">
        <p className="text-xs text-gray-700 dark:text-gray-400">
          💡 <strong>Supported formats:</strong> PDF, DOCX, DOC, TXT, MD, JSON, CSV, XML, HTML.
          AgenticRAG provides enhanced reasoning and more accurate results for complex legal queries.
          All processing happens locally for maximum privacy.
        </p>
      </div>
    </div>
  );
};

export default RAGInterface;