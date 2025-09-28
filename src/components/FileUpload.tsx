import React, { useState } from 'react';
import { Upload, FileText, CheckCircle, XCircle, Loader2 } from 'lucide-react';

interface FileUploadProps {
  onFileSelect: (file: File) => void;
  isProcessing: boolean;
}

const FileUpload: React.FC<FileUploadProps> = ({ onFileSelect, isProcessing }) => {
  const [dragActive, setDragActive] = useState(false);
  const [uploadStatus, setUploadStatus] = useState<'idle' | 'success' | 'error'>('idle');

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true);
    } else if (e.type === 'dragleave') {
      setDragActive(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);

    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      onFileSelect(e.dataTransfer.files[0]);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    e.preventDefault();
    if (e.target.files && e.target.files[0]) {
      onFileSelect(e.target.files[0]);
    }
  };

  return (
    <div
      className={`relative border-2 border-dashed rounded-lg p-8 transition-colors ${
        dragActive
          ? 'border-legal-accent bg-legal-accent/10'
          : 'border-legal-secondary hover:border-legal-accent'
      }`}
      onDragEnter={handleDrag}
      onDragLeave={handleDrag}
      onDragOver={handleDrag}
      onDrop={handleDrop}
    >
      <input
        type="file"
        id="file-upload"
        onChange={handleChange}
        accept=".txt,.pdf,.docx,.doc,.xlsx,.xls,.csv,.pptx,.ppt"
        className="hidden"
        disabled={isProcessing}
      />

      <label
        htmlFor="file-upload"
        className="flex flex-col items-center justify-center cursor-pointer"
      >
        {isProcessing ? (
          <Loader2 className="w-12 h-12 mb-4 animate-spin text-legal-accent" />
        ) : uploadStatus === 'success' ? (
          <CheckCircle className="w-12 h-12 mb-4 text-green-400" />
        ) : uploadStatus === 'error' ? (
          <XCircle className="w-12 h-12 mb-4 text-red-400" />
        ) : (
          <Upload className="w-12 h-12 mb-4 text-legal-accent" />
        )}

        <p className="text-lg font-semibold mb-2">
          {isProcessing
            ? 'Processing document...'
            : 'Drop files here or click to browse'}
        </p>

        <p className="text-sm text-gray-400 text-center">
          Supported formats: PDF, Word, Excel, PowerPoint, CSV, TXT
        </p>

        <div className="mt-4 flex items-center space-x-2 text-xs text-green-400">
          <FileText className="w-4 h-4" />
          <span>All PII will be automatically removed before processing</span>
        </div>
      </label>
    </div>
  );
};

export default FileUpload;