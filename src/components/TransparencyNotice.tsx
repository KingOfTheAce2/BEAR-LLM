import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  AlertTriangle,
  Info,
  Shield,
  FileText,
  CheckCircle2,
  X,
  ExternalLink,
  ChevronDown,
  ChevronUp,
  Scale,
  Eye,
  AlertCircle
} from 'lucide-react';

interface TransparencyNoticeProps {
  onClose: () => void;
  theme?: 'light' | 'dark';
  triggerSource?: 'firstLaunch' | 'menu';
}

interface ModelInfo {
  name: string;
  parameters: string;
  size: string;
  accuracy: string;
  limitations: string[];
}

export const TransparencyNotice: React.FC<TransparencyNoticeProps> = ({
  onClose,
  theme = 'dark',
  triggerSource = 'menu'
}) => {
  const [accepted, setAccepted] = useState(false);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['risk']));
  const [currentModel, setCurrentModel] = useState<string>('Unknown');

  useEffect(() => {
    // Get current model from backend
    const fetchCurrentModel = async () => {
      try {
        const model = await invoke<string>('get_current_model');
        setCurrentModel(model);
      } catch (err) {
        console.error('Failed to get current model:', err);
      }
    };
    fetchCurrentModel();
  }, []);

  const toggleSection = (section: string) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(section)) {
      newExpanded.delete(section);
    } else {
      newExpanded.add(section);
    }
    setExpandedSections(newExpanded);
  };

  const handleAccept = async () => {
    setAccepted(true);
    if (triggerSource === 'firstLaunch') {
      try {
        await invoke('set_transparency_acknowledged');
      } catch (err) {
        console.error('Failed to save acknowledgment:', err);
      }
    }
    setTimeout(() => onClose(), 500);
  };

  const getModelInfo = (modelName: string): ModelInfo => {
    if (modelName.toLowerCase().includes('tinyllama')) {
      return {
        name: 'TinyLlama-1.1B-Chat-v1.0',
        parameters: '1.1 billion',
        size: '~850 MB',
        accuracy: 'MMLU: 25.3%, HumanEval: 10.2%',
        limitations: [
          'Limited reasoning capability',
          'Higher hallucination rate (~15-25%)',
          '2,048 token context limit',
          'Basic tasks only'
        ]
      };
    } else if (modelName.toLowerCase().includes('phi')) {
      return {
        name: 'Phi-2',
        parameters: '2.7 billion',
        size: '~1.8 GB',
        accuracy: 'MMLU: 56.3%, HumanEval: 47.0%',
        limitations: [
          'Good reasoning but not perfect',
          'Moderate hallucination rate (~8-12%)',
          '2,048 token context limit',
          'May struggle with specialized domains'
        ]
      };
    } else if (modelName.toLowerCase().includes('mistral')) {
      return {
        name: 'Mistral-7B-Instruct-v0.2',
        parameters: '7.24 billion',
        size: '~4.6 GB',
        accuracy: 'MMLU: 62.5%, HumanEval: 40.2%',
        limitations: [
          'Strong but not infallible reasoning',
          'Low hallucination rate (~5-8%)',
          '32,768 token context (may miss middle details)',
          'Requires 32GB RAM for long contexts'
        ]
      };
    }
    return {
      name: 'Unknown Model',
      parameters: 'Unknown',
      size: 'Unknown',
      accuracy: 'Unknown',
      limitations: ['Model information not available']
    };
  };

  const modelInfo = getModelInfo(currentModel);

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4 overflow-y-auto">
      <div
        className={`bg-[var(--bg-primary)] border-2 border-[var(--border-primary)] rounded-lg shadow-2xl max-w-4xl w-full max-h-[90vh] overflow-y-auto scrollbar-custom ${
          accepted ? 'animate-fadeOut' : 'animate-fadeIn'
        }`}
      >
        {/* Header */}
        <div className="sticky top-0 bg-[var(--bg-secondary)] border-b-2 border-[var(--border-primary)] p-6 flex items-center justify-between z-10">
          <div className="flex items-center gap-3">
            <Scale className="w-8 h-8 text-[var(--accent)]" />
            <div>
              <h2 className="text-2xl font-bold text-[var(--text-primary)]">
                AI Transparency Notice
              </h2>
              <p className="text-sm text-[var(--text-secondary)]">
                EU AI Act Compliance - Articles 13 & 52
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-[var(--hover-bg)] rounded-lg transition-all"
            aria-label="Close transparency notice"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Critical Risk Classification */}
          <div className="bg-orange-500/10 border-2 border-orange-500/30 rounded-lg p-4">
            <div className="flex items-start gap-3">
              <AlertTriangle className="w-6 h-6 text-orange-500 flex-shrink-0 mt-0.5" />
              <div className="flex-1">
                <h3 className="font-bold text-lg text-orange-500 mb-2">
                  High-Risk AI System
                </h3>
                <p className="text-sm text-[var(--text-primary)]">
                  BEAR AI is classified as a <strong>high-risk AI system</strong> under EU AI Act Annex III
                  because it processes legal and professional documents that may inform critical decisions.
                  This system <strong>MUST NOT</strong> be used as sole authority for legal opinions,
                  medical decisions, or financial advice.
                </p>
              </div>
            </div>
          </div>

          {/* Current Model Information */}
          <div className="bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg p-4">
            <div className="flex items-center gap-2 mb-3">
              <Eye className="w-5 h-5 text-[var(--accent)]" />
              <h3 className="font-bold text-lg">Current AI Model: {modelInfo.name}</h3>
            </div>
            <div className="grid grid-cols-2 gap-3 text-sm">
              <div>
                <span className="text-[var(--text-secondary)]">Parameters:</span>
                <span className="ml-2 font-medium">{modelInfo.parameters}</span>
              </div>
              <div>
                <span className="text-[var(--text-secondary)]">Size:</span>
                <span className="ml-2 font-medium">{modelInfo.size}</span>
              </div>
              <div className="col-span-2">
                <span className="text-[var(--text-secondary)]">Accuracy:</span>
                <span className="ml-2 font-medium">{modelInfo.accuracy}</span>
              </div>
            </div>
            <div className="mt-3 pt-3 border-t border-[var(--border-primary)]">
              <p className="text-xs text-[var(--text-secondary)] mb-2">Key Limitations:</p>
              <ul className="text-xs space-y-1">
                {modelInfo.limitations.map((limitation, idx) => (
                  <li key={idx} className="flex items-start gap-2">
                    <span className="text-orange-500 mt-0.5">•</span>
                    <span>{limitation}</span>
                  </li>
                ))}
              </ul>
            </div>
          </div>

          {/* Expandable Sections */}
          <div className="space-y-3">
            {/* Capabilities */}
            <ExpandableSection
              id="capabilities"
              title="Capabilities and Intended Use"
              icon={<CheckCircle2 className="w-5 h-5" />}
              expanded={expandedSections.has('capabilities')}
              onToggle={() => toggleSection('capabilities')}
            >
              <div className="space-y-3 text-sm">
                <div>
                  <h4 className="font-semibold mb-2">What BEAR AI CAN Do:</h4>
                  <ul className="space-y-1 text-[var(--text-secondary)]">
                    <li>• Assist with document analysis and summarization</li>
                    <li>• Answer questions about uploaded documents (RAG)</li>
                    <li>• Detect and scrub PII (accuracy: ~85-95% with Presidio)</li>
                    <li>• Generate drafts for review by professionals</li>
                    <li>• Process documents 100% locally (no cloud services)</li>
                  </ul>
                </div>
                <div>
                  <h4 className="font-semibold mb-2 text-red-500">What BEAR AI CANNOT Do:</h4>
                  <ul className="space-y-1 text-[var(--text-secondary)]">
                    <li>• <strong>Provide legal advice</strong> or professional opinions</li>
                    <li>• Replace qualified attorneys, doctors, or financial advisors</li>
                    <li>• Guarantee 100% accuracy or freedom from errors</li>
                    <li>• Make final decisions in high-stakes contexts</li>
                    <li>• Access current events or information after training cutoff</li>
                  </ul>
                </div>
              </div>
            </ExpandableSection>

            {/* Limitations and Risks */}
            <ExpandableSection
              id="risk"
              title="Limitations and Risks"
              icon={<AlertCircle className="w-5 h-5" />}
              expanded={expandedSections.has('risk')}
              onToggle={() => toggleSection('risk')}
              defaultOpen
            >
              <div className="space-y-3 text-sm">
                <div className="bg-red-500/10 border border-red-500/30 rounded p-3">
                  <h4 className="font-semibold text-red-500 mb-2">Hallucination Risk</h4>
                  <p className="text-[var(--text-secondary)]">
                    AI models can generate plausible but <strong>incorrect or fabricated information</strong>.
                    Hallucination rates: TinyLlama (~15-25%), Phi-2 (~8-12%), Mistral (~5-8%).
                    <strong className="text-red-500"> Always verify critical information.</strong>
                  </p>
                </div>
                <div className="bg-orange-500/10 border border-orange-500/30 rounded p-3">
                  <h4 className="font-semibold text-orange-500 mb-2">PII Detection Limitations</h4>
                  <p className="text-[var(--text-secondary)]">
                    Built-in detection: ~75-85% accuracy. Presidio enhancement: ~85-95% accuracy.
                    <strong className="text-orange-500"> Never rely solely on automated PII detection for regulatory compliance.</strong>
                  </p>
                </div>
                <div>
                  <h4 className="font-semibold mb-2">Other Important Limitations:</h4>
                  <ul className="space-y-1 text-[var(--text-secondary)]">
                    <li>• Knowledge cutoff dates (2023-2024, varies by model)</li>
                    <li>• English-optimized (limited multilingual support)</li>
                    <li>• Potential biases from training data</li>
                    <li>• Context length limits (2k-32k tokens depending on model)</li>
                  </ul>
                </div>
              </div>
            </ExpandableSection>

            {/* Privacy and Data Handling */}
            <ExpandableSection
              id="privacy"
              title="Privacy and Data Handling"
              icon={<Shield className="w-5 h-5" />}
              expanded={expandedSections.has('privacy')}
              onToggle={() => toggleSection('privacy')}
            >
              <div className="space-y-3 text-sm">
                <div className="flex items-start gap-2 text-green-600 dark:text-green-400">
                  <CheckCircle2 className="w-5 h-5 flex-shrink-0 mt-0.5" />
                  <div>
                    <strong>100% Local Processing:</strong> All AI computations occur on your device.
                    No data transmitted to external servers.
                  </div>
                </div>
                <div className="space-y-2 text-[var(--text-secondary)]">
                  <p>• <strong>No Telemetry:</strong> Zero analytics or usage tracking</p>
                  <p>• <strong>No Cloud Services:</strong> Complete offline operation after model download</p>
                  <p>• <strong>Encrypted Storage:</strong> Documents and embeddings encrypted (AES-256)</p>
                  <p>• <strong>User Control:</strong> Delete all data and models at any time</p>
                  <p>• <strong>GDPR Compliant:</strong> Data minimization and right to erasure</p>
                </div>
              </div>
            </ExpandableSection>

            {/* Your Rights and Responsibilities */}
            <ExpandableSection
              id="rights"
              title="Your Rights and Responsibilities"
              icon={<Scale className="w-5 h-5" />}
              expanded={expandedSections.has('rights')}
              onToggle={() => toggleSection('rights')}
            >
              <div className="space-y-3 text-sm">
                <div>
                  <h4 className="font-semibold mb-2">Your Rights (EU AI Act & GDPR):</h4>
                  <ul className="space-y-1 text-[var(--text-secondary)]">
                    <li>• Right to information about AI system capabilities</li>
                    <li>• Right to explanation of how outputs are generated</li>
                    <li>• Right to human review of AI-generated content</li>
                    <li>• Right to delete all data and models</li>
                    <li>• Right to lodge complaints with regulatory authorities</li>
                  </ul>
                </div>
                <div>
                  <h4 className="font-semibold mb-2">Your Responsibilities:</h4>
                  <ul className="space-y-1 text-[var(--text-secondary)]">
                    <li>• <strong>Verify all AI outputs</strong> before professional reliance</li>
                    <li>• Maintain expert judgment and human oversight</li>
                    <li>• Use only for intended purposes (not prohibited uses)</li>
                    <li>• Ensure compliance with applicable regulations</li>
                    <li>• Inform others when sharing AI-generated content</li>
                  </ul>
                </div>
              </div>
            </ExpandableSection>
          </div>

          {/* Model Cards Link */}
          <div className="bg-blue-500/10 border border-blue-500/30 rounded-lg p-4">
            <div className="flex items-start gap-3">
              <FileText className="w-5 h-5 text-blue-500 flex-shrink-0 mt-0.5" />
              <div className="flex-1">
                <h4 className="font-semibold mb-2">Detailed Model Information</h4>
                <p className="text-sm text-[var(--text-secondary)] mb-3">
                  Comprehensive model cards are available for all supported AI models, including
                  performance benchmarks, bias testing, and environmental impact.
                </p>
                <button
                  onClick={() => {
                    invoke('open_model_cards_folder');
                  }}
                  className="flex items-center gap-2 text-sm text-blue-500 hover:text-blue-400 transition-colors"
                >
                  <span>View Model Cards</span>
                  <ExternalLink className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>

          {/* Acknowledgment Section */}
          {triggerSource === 'firstLaunch' && (
            <div className="bg-[var(--bg-secondary)] border-2 border-[var(--accent)] rounded-lg p-4">
              <div className="flex items-start gap-3">
                <Info className="w-6 h-6 text-[var(--accent)] flex-shrink-0 mt-0.5" />
                <div className="flex-1">
                  <h4 className="font-semibold mb-2">Acknowledgment Required</h4>
                  <p className="text-sm text-[var(--text-secondary)] mb-4">
                    By clicking "I Understand and Accept", you acknowledge that you have read and
                    understood this transparency notice, including the system's capabilities,
                    limitations, risks, and your responsibilities as a user.
                  </p>
                  <label className="flex items-start gap-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={accepted}
                      onChange={(e) => setAccepted(e.target.checked)}
                      className="mt-1"
                    />
                    <span className="text-sm">
                      I understand that BEAR AI is a <strong>high-risk AI system</strong> that must not
                      replace professional judgment. I will verify all outputs and maintain human oversight.
                    </span>
                  </label>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="sticky bottom-0 bg-[var(--bg-secondary)] border-t-2 border-[var(--border-primary)] p-4 flex items-center justify-between">
          <div className="text-xs text-[var(--text-secondary)]">
            Version 1.0.0 • EU AI Act Compliant • Last Updated: October 2, 2025
          </div>
          <div className="flex gap-3">
            {triggerSource === 'menu' ? (
              <button
                onClick={onClose}
                className="px-4 py-2 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white rounded-lg transition-all"
              >
                Close
              </button>
            ) : (
              <>
                <button
                  onClick={onClose}
                  className="px-4 py-2 border border-[var(--border-primary)] hover:bg-[var(--hover-bg)] rounded-lg transition-all"
                >
                  Cancel
                </button>
                <button
                  onClick={handleAccept}
                  disabled={!accepted}
                  className={`px-6 py-2 rounded-lg transition-all ${
                    accepted
                      ? 'bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white'
                      : 'bg-gray-300 text-gray-500 cursor-not-allowed'
                  }`}
                >
                  I Understand and Accept
                </button>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

// Expandable Section Component
interface ExpandableSectionProps {
  id: string;
  title: string;
  icon: React.ReactNode;
  expanded: boolean;
  onToggle: () => void;
  children: React.ReactNode;
  defaultOpen?: boolean;
}

const ExpandableSection: React.FC<ExpandableSectionProps> = ({
  title,
  icon,
  expanded,
  onToggle,
  children,
  defaultOpen = false
}) => {
  return (
    <div className="bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg overflow-hidden">
      <button
        onClick={onToggle}
        className="w-full flex items-center justify-between p-4 hover:bg-[var(--hover-bg)] transition-all"
      >
        <div className="flex items-center gap-3">
          <span className="text-[var(--accent)]">{icon}</span>
          <h3 className="font-semibold text-[var(--text-primary)]">{title}</h3>
        </div>
        {expanded ? (
          <ChevronUp className="w-5 h-5 text-[var(--text-secondary)]" />
        ) : (
          <ChevronDown className="w-5 h-5 text-[var(--text-secondary)]" />
        )}
      </button>
      {expanded && (
        <div className="p-4 pt-0 border-t border-[var(--border-primary)] animate-fadeIn">
          {children}
        </div>
      )}
    </div>
  );
};

export default TransparencyNotice;
