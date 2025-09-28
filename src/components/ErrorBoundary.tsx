import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertCircle, RefreshCw, Bug, Copy, ChevronDown, ChevronUp } from 'lucide-react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  showDetails: boolean;
  errorId: string;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      showDetails: false,
      errorId: '',
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    // Update state so the next render will show the fallback UI
    return {
      hasError: true,
      error,
      errorId: `ERR_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error details
    this.setState({
      error,
      errorInfo,
    });

    // Production-safe error logging
    this.logError(error, errorInfo);

    // Call optional error callback
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
  }

  private logError = (error: Error, errorInfo: ErrorInfo) => {
    // Production-safe error logging
    const errorDetails = {
      message: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
      timestamp: new Date().toISOString(),
      errorId: this.state.errorId,
      userAgent: navigator.userAgent,
      url: window.location.href,
    };

    try {
      // Log to console (safe for development and debugging)
      console.group('🚨 React Error Boundary Caught Error');
      console.error('Error:', error);
      console.error('Error Info:', errorInfo);
      console.error('Error Details:', errorDetails);
      console.groupEnd();

      // Store in localStorage for potential debugging (with size limits)
      const errorLog = {
        ...errorDetails,
        // Truncate stack traces to prevent localStorage overflow
        stack: error.stack?.substring(0, 2000),
        componentStack: errorInfo.componentStack?.substring(0, 2000),
      };

      try {
        const existingLogs = JSON.parse(localStorage.getItem('bear_error_logs') || '[]');
        const logs = [errorLog, ...existingLogs].slice(0, 10); // Keep only last 10 errors
        localStorage.setItem('bear_error_logs', JSON.stringify(logs));
      } catch (storageError) {
        console.warn('Failed to store error log in localStorage:', storageError);
      }

      // In a real production app, you might want to send to an error reporting service
      // This would be done through your backend to keep API keys secure
      // Example: await fetch('/api/log-error', { method: 'POST', body: JSON.stringify(errorDetails) });

    } catch (loggingError) {
      console.error('Failed to log error:', loggingError);
    }
  };

  private handleReload = () => {
    window.location.reload();
  };

  private handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      showDetails: false,
      errorId: '',
    });
  };

  private toggleDetails = () => {
    this.setState(prev => ({ showDetails: !prev.showDetails }));
  };

  private copyErrorDetails = async () => {
    if (!this.state.error || !this.state.errorInfo) return;

    const errorText = `
BEAR AI Error Report
==================
Error ID: ${this.state.errorId}
Timestamp: ${new Date().toISOString()}
Message: ${this.state.error.message}

Stack Trace:
${this.state.error.stack || 'No stack trace available'}

Component Stack:
${this.state.errorInfo.componentStack || 'No component stack available'}

User Agent: ${navigator.userAgent}
URL: ${window.location.href}
    `.trim();

    try {
      await navigator.clipboard.writeText(errorText);
      // You could show a toast notification here
      console.log('Error details copied to clipboard');
    } catch (err) {
      console.error('Failed to copy error details:', err);
      // Fallback: select text in a temporary textarea
      const textarea = document.createElement('textarea');
      textarea.value = errorText;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
    }
  };

  render() {
    if (this.state.hasError) {
      // Custom fallback UI
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default error UI
      return (
        <div className="min-h-screen bg-[var(--bg-primary)] flex items-center justify-center p-4">
          <div className="max-w-2xl w-full bg-[var(--bg-secondary)] rounded-xl border border-[var(--border-primary)] overflow-hidden shadow-xl">
            {/* Header */}
            <div className="bg-red-50 dark:bg-red-900/20 border-b border-red-200 dark:border-red-800 p-6">
              <div className="flex items-center gap-3">
                <div className="w-12 h-12 bg-red-500 rounded-full flex items-center justify-center">
                  <AlertCircle className="w-6 h-6 text-white" />
                </div>
                <div>
                  <h1 className="text-xl font-bold text-red-800 dark:text-red-200">
                    Something went wrong
                  </h1>
                  <p className="text-red-600 dark:text-red-300 text-sm">
                    BEAR AI encountered an unexpected error
                  </p>
                </div>
              </div>
            </div>

            {/* Content */}
            <div className="p-6 space-y-4">
              <div className="text-[var(--text-secondary)] text-sm">
                <p className="mb-2">
                  We apologize for the inconvenience. The application has encountered an error and cannot continue.
                </p>
                <p className="mb-4">
                  Your data is safe and remains on your device. You can try the actions below to resolve this issue.
                </p>
              </div>

              {/* Error ID */}
              <div className="bg-[var(--bg-primary)] rounded-lg p-3 border border-[var(--border-primary)]">
                <div className="text-xs text-[var(--text-tertiary)] mb-1">Error ID</div>
                <div className="font-mono text-sm text-[var(--text-primary)]">
                  {this.state.errorId}
                </div>
              </div>

              {/* Actions */}
              <div className="flex flex-wrap gap-3">
                <button
                  onClick={this.handleReset}
                  className="flex items-center gap-2 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
                >
                  <RefreshCw className="w-4 h-4" />
                  Try Again
                </button>

                <button
                  onClick={this.handleReload}
                  className="flex items-center gap-2 px-4 py-2 bg-[var(--bg-primary)] hover:bg-[var(--hover-bg)] border border-[var(--border-primary)] rounded-lg transition-colors"
                >
                  <RefreshCw className="w-4 h-4" />
                  Reload Application
                </button>

                <button
                  onClick={this.copyErrorDetails}
                  className="flex items-center gap-2 px-4 py-2 bg-[var(--bg-primary)] hover:bg-[var(--hover-bg)] border border-[var(--border-primary)] rounded-lg transition-colors"
                >
                  <Copy className="w-4 h-4" />
                  Copy Error Details
                </button>
              </div>

              {/* Toggle Error Details */}
              <div className="border-t border-[var(--border-primary)] pt-4">
                <button
                  onClick={this.toggleDetails}
                  className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
                >
                  <Bug className="w-4 h-4" />
                  Technical Details
                  {this.state.showDetails ? (
                    <ChevronUp className="w-4 h-4" />
                  ) : (
                    <ChevronDown className="w-4 h-4" />
                  )}
                </button>

                {this.state.showDetails && this.state.error && (
                  <div className="mt-3 space-y-3">
                    <div className="bg-[var(--bg-primary)] rounded-lg p-3 border border-[var(--border-primary)]">
                      <div className="text-xs text-[var(--text-tertiary)] mb-2">Error Message</div>
                      <div className="text-sm font-mono text-red-600 dark:text-red-400">
                        {this.state.error.message}
                      </div>
                    </div>

                    {this.state.error.stack && (
                      <div className="bg-[var(--bg-primary)] rounded-lg p-3 border border-[var(--border-primary)]">
                        <div className="text-xs text-[var(--text-tertiary)] mb-2">Stack Trace</div>
                        <pre className="text-xs font-mono text-[var(--text-secondary)] overflow-x-auto whitespace-pre-wrap max-h-32 overflow-y-auto">
                          {this.state.error.stack.substring(0, 1000)}
                          {this.state.error.stack.length > 1000 && '\n... (truncated)'}
                        </pre>
                      </div>
                    )}

                    {this.state.errorInfo?.componentStack && (
                      <div className="bg-[var(--bg-primary)] rounded-lg p-3 border border-[var(--border-primary)]">
                        <div className="text-xs text-[var(--text-tertiary)] mb-2">Component Stack</div>
                        <pre className="text-xs font-mono text-[var(--text-secondary)] overflow-x-auto whitespace-pre-wrap max-h-32 overflow-y-auto">
                          {this.state.errorInfo.componentStack.substring(0, 1000)}
                          {this.state.errorInfo.componentStack.length > 1000 && '\n... (truncated)'}
                        </pre>
                      </div>
                    )}
                  </div>
                )}
              </div>

              {/* Help Text */}
              <div className="text-xs text-[var(--text-tertiary)] border-t border-[var(--border-primary)] pt-4">
                <p>
                  If this error persists, please restart the application.
                  All your data is stored locally and will be preserved.
                </p>
              </div>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;