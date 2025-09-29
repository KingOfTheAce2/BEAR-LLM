import React, { useState, useRef } from 'react';
import { Shield, Upload, FileText, AlertTriangle, CheckCircle, Eye, EyeOff, Download } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface PIIDetection {
  type: string;
  text: string;
  startIndex: number;
  endIndex: number;
  confidence: number;
  replacement: string;
}

interface DocumentAnalysis {
  filename: string;
  fileType: string;
  originalText: string;
  cleanedText: string;
  piiDetections: PIIDetection[];
  processingTime: number;
  supported: boolean;
  error?: string;
}

const DocumentAnalysis: React.FC = () => {
  const [analyses, setAnalyses] = useState<DocumentAnalysis[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [showOriginal, setShowOriginal] = useState<{ [key: string]: boolean }>({});
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files || []);
    if (files.length === 0) return;

    setIsProcessing(true);

    for (const file of files) {
      try {
        const fileContent = await file.arrayBuffer();
        const uint8Array = new Uint8Array(fileContent);

        const result = await invoke<DocumentAnalysis>('analyze_document_pii', {
          filename: file.name,
          content: Array.from(uint8Array)
        });

        setAnalyses(prev => [...prev, result]);
      } catch (error) {
        const errorAnalysis: DocumentAnalysis = {
          filename: file.name,
          fileType: file.name.split('.').pop() || 'unknown',
          originalText: '',
          cleanedText: '',
          piiDetections: [],
          processingTime: 0,
          supported: false,
          error: error instanceof Error ? error.message : 'Processing failed'
        };
        setAnalyses(prev => [...prev, errorAnalysis]);
      }
    }

    setIsProcessing(false);
  };

  const toggleShowOriginal = (filename: string) => {
    setShowOriginal(prev => ({
      ...prev,
      [filename]: !prev[filename]
    }));
  };

  const downloadCleanedDocument = (analysis: DocumentAnalysis) => {
    const blob = new Blob([analysis.cleanedText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${analysis.filename.split('.')[0]}_cleaned.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const getPIITypeColor = (type: string) => {
    const colors: { [key: string]: string } = {
      'ssn': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300',
      'email': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
      'phone': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
      'credit_card': 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300',
      'address': 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300',
      'name': 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-300',
      'case_number': 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-300',
      'organization': 'bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-300'
    };
    return colors[type.toLowerCase()] || 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300';
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.9) return 'text-green-600';
    if (confidence >= 0.7) return 'text-yellow-600';
    return 'text-red-600';
  };

  const supportedFormats = [
    'PDF', 'DOCX', 'DOC', 'TXT', 'MD', 'JSON', 'CSV', 'XML', 'HTML', 'XLSX', 'XLS', 'PPTX', 'PPT', 'RTF'
  ];

  return (
    <div className="bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-2">
          <Shield className="w-5 h-5 text-[var(--accent)]" />
          <h3 className="text-lg font-semibold text-[var(--text-primary)]">
            PII Guard - Document Analysis
          </h3>
        </div>

        <div>
          <input
            type="file"
            ref={fileInputRef}
            onChange={handleFileUpload}
            multiple
            accept=".pdf,.docx,.doc,.txt,.md,.json,.csv,.xml,.html,.xlsx,.xls,.pptx,.ppt,.rtf"
            className="hidden"
          />
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={isProcessing}
            className="flex items-center gap-2 px-4 py-2 bg-[var(--accent)] hover:bg-[var(--accent-hover)] disabled:opacity-50 text-white rounded-lg transition-colors"
          >
            <Upload className="w-4 h-4" />
            {isProcessing ? 'Processing...' : 'Upload Documents'}
          </button>
        </div>
      </div>

      {/* Supported Formats */}
      <div className="mb-6 p-4 bg-[var(--bg-tertiary)] rounded-lg border border-[var(--border-secondary)]">
        <h4 className="text-sm font-medium text-[var(--text-primary)] mb-2">Supported Document Formats</h4>
        <div className="flex flex-wrap gap-2">
          {supportedFormats.map((format) => (
            <span
              key={format}
              className="px-2 py-1 text-xs bg-[var(--bg-primary)] border border-[var(--border-primary)] rounded"
            >
              {format}
            </span>
          ))}
        </div>
        <p className="text-xs text-[var(--text-tertiary)] mt-2">
          All document processing happens locally for maximum privacy and security.
        </p>
      </div>

      {/* Analysis Results */}
      {analyses.length > 0 && (
        <div className="space-y-6">
          <h4 className="text-sm font-medium text-[var(--text-primary)]">Analysis Results</h4>

          {analyses.map((analysis, index) => (
            <div
              key={index}
              className="p-4 bg-[var(--bg-primary)] border border-[var(--border-primary)] rounded-lg"
            >
              {/* Header */}
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <FileText className="w-4 h-4 text-[var(--text-secondary)]" />
                  <h5 className="text-sm font-medium text-[var(--text-primary)]">
                    {analysis.filename}
                  </h5>
                  <span className="px-2 py-1 text-xs bg-[var(--bg-tertiary)] rounded">
                    {analysis.fileType.toUpperCase()}
                  </span>
                </div>

                <div className="flex items-center gap-2">
                  {analysis.supported ? (
                    <>
                      <CheckCircle className="w-4 h-4 text-green-500" />
                      <span className="text-xs text-[var(--text-tertiary)]">
                        {analysis.processingTime}ms
                      </span>
                    </>
                  ) : (
                    <>
                      <AlertTriangle className="w-4 h-4 text-red-500" />
                      <span className="text-xs text-red-600">{analysis.error}</span>
                    </>
                  )}
                </div>
              </div>

              {analysis.supported && (
                <>
                  {/* PII Detection Summary */}
                  <div className="mb-4 p-3 bg-[var(--bg-tertiary)] rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <h6 className="text-sm font-medium text-[var(--text-primary)]">
                        PII Detection Summary
                      </h6>
                      <span className="text-xs text-[var(--text-tertiary)]">
                        {analysis.piiDetections.length} items found
                      </span>
                    </div>

                    {analysis.piiDetections.length > 0 ? (
                      <div className="space-y-2">
                        {analysis.piiDetections.map((pii, piiIndex) => (
                          <div
                            key={piiIndex}
                            className="flex items-center justify-between p-2 bg-[var(--bg-primary)] rounded text-xs"
                          >
                            <div className="flex items-center gap-2">
                              <span className={`px-2 py-1 rounded-full ${getPIITypeColor(pii.type)}`}>
                                {pii.type.replace('_', ' ').toUpperCase()}
                              </span>
                              <span className="font-mono text-[var(--text-secondary)]">
                                {showOriginal[analysis.filename] ? pii.text : pii.replacement}
                              </span>
                            </div>
                            <span className={`font-mono ${getConfidenceColor(pii.confidence)}`}>
                              {Math.round(pii.confidence * 100)}%
                            </span>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <p className="text-xs text-green-600">✅ No PII detected in this document</p>
                    )}
                  </div>

                  {/* Document Text */}
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <h6 className="text-sm font-medium text-[var(--text-primary)]">
                        Document Content
                      </h6>
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => toggleShowOriginal(analysis.filename)}
                          className="flex items-center gap-1 px-2 py-1 text-xs bg-[var(--bg-tertiary)] hover:bg-[var(--hover-bg)] rounded transition-colors"
                        >
                          {showOriginal[analysis.filename] ? (
                            <>
                              <EyeOff className="w-3 h-3" />
                              Hide Original
                            </>
                          ) : (
                            <>
                              <Eye className="w-3 h-3" />
                              Show Original
                            </>
                          )}
                        </button>
                        <button
                          onClick={() => downloadCleanedDocument(analysis)}
                          className="flex items-center gap-1 px-2 py-1 text-xs bg-green-600 hover:bg-green-700 text-white rounded transition-colors"
                        >
                          <Download className="w-3 h-3" />
                          Download Cleaned
                        </button>
                      </div>
                    </div>

                    <div className="p-3 bg-[var(--bg-tertiary)] rounded-lg border border-[var(--border-secondary)]">
                      <pre className="text-xs text-[var(--text-secondary)] whitespace-pre-wrap max-h-48 overflow-y-auto">
                        {showOriginal[analysis.filename]
                          ? analysis.originalText || 'Original text not available'
                          : analysis.cleanedText || 'No cleaned text available'
                        }
                      </pre>
                    </div>
                  </div>
                </>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Instructions */}
      <div className="mt-6 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
        <div className="flex items-start gap-2">
          <Shield className="w-4 h-4 text-yellow-600 mt-0.5 flex-shrink-0" />
          <div>
            <p className="text-xs text-yellow-700 dark:text-yellow-400">
              <strong>PII Guard automatically detects and redacts:</strong>
            </p>
            <ul className="text-xs text-yellow-600 dark:text-yellow-500 mt-1 space-y-1">
              <li>• Social Security Numbers (SSN)</li>
              <li>• Email addresses and phone numbers</li>
              <li>• Credit card numbers and addresses</li>
              <li>• Names and legal case numbers</li>
              <li>• Organization names and identifiers</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DocumentAnalysis;