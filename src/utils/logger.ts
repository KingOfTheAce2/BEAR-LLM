/**
 * Production-ready logging service for BEAR AI
 *
 * Features:
 * - Environment-aware logging (dev vs production)
 * - Structured logging with context
 * - Multiple log levels (debug, info, warn, error)
 * - Safe error serialization
 * - Optional monitoring service integration
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogContext {
  [key: string]: any;
}

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  context?: LogContext;
  error?: {
    message: string;
    stack?: string;
    name?: string;
  };
}

class Logger {
  private isDevelopment: boolean;
  private enableConsole: boolean;
  private logHistory: LogEntry[] = [];
  private maxHistorySize = 100;

  constructor() {
    // Detect environment
    this.isDevelopment = import.meta.env.DEV || import.meta.env.MODE === 'development';
    // In production, we can still enable console for debugging via localStorage
    this.enableConsole = this.isDevelopment || localStorage.getItem('bear_debug') === 'true';
  }

  /**
   * Log debug information (only in development)
   */
  debug(message: string, context?: LogContext): void {
    if (!this.isDevelopment) return;
    this.log('debug', message, context);
  }

  /**
   * Log informational messages
   */
  info(message: string, context?: LogContext): void {
    this.log('info', message, context);
  }

  /**
   * Log warning messages
   */
  warn(message: string, context?: LogContext): void {
    this.log('warn', message, context);
  }

  /**
   * Log error messages
   */
  error(message: string, error?: Error | unknown, context?: LogContext): void {
    const errorInfo = this.serializeError(error);
    this.log('error', message, context, errorInfo);
  }

  /**
   * Core logging method
   */
  private log(
    level: LogLevel,
    message: string,
    context?: LogContext,
    error?: { message: string; stack?: string; name?: string }
  ): void {
    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      context,
      error,
    };

    // Store in history (with size limit)
    this.addToHistory(entry);

    // Console output (development or when debug is enabled)
    if (this.enableConsole) {
      this.logToConsole(entry);
    }

    // In production, you could send to a monitoring service
    if (!this.isDevelopment) {
      this.sendToMonitoring(entry);
    }
  }

  /**
   * Log to browser console with formatting
   */
  private logToConsole(entry: LogEntry): void {
    const prefix = `[BEAR AI] [${entry.level.toUpperCase()}]`;
    const timestamp = new Date(entry.timestamp).toLocaleTimeString();
    const logMessage = `${prefix} ${timestamp} - ${entry.message}`;

    switch (entry.level) {
      case 'debug':
        console.debug(logMessage, entry.context || '');
        break;
      case 'info':
        console.info(logMessage, entry.context || '');
        break;
      case 'warn':
        console.warn(logMessage, entry.context || '');
        break;
      case 'error':
        console.error(logMessage);
        if (entry.error) {
          console.error('Error details:', entry.error);
        }
        if (entry.context) {
          console.error('Context:', entry.context);
        }
        break;
    }
  }

  /**
   * Store log entry in memory
   */
  private addToHistory(entry: LogEntry): void {
    this.logHistory.push(entry);
    // Keep only recent logs
    if (this.logHistory.length > this.maxHistorySize) {
      this.logHistory.shift();
    }
  }

  /**
   * Send logs to monitoring service (placeholder for future implementation)
   */
  private sendToMonitoring(entry: LogEntry): void {
    // Only log errors to monitoring in production
    if (entry.level !== 'error') return;

    try {
      // Store in localStorage for now (can be retrieved for debugging)
      const errorLogs = JSON.parse(localStorage.getItem('bear_error_logs') || '[]');
      const logs = [entry, ...errorLogs].slice(0, 10); // Keep only last 10 errors
      localStorage.setItem('bear_error_logs', JSON.stringify(logs));

      // Future: Send to external monitoring service
      // await fetch('/api/log', {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify(entry)
      // });
    } catch (err) {
      // Silently fail to prevent logging errors from breaking the app
      console.error('Failed to store error log:', err);
    }
  }

  /**
   * Safely serialize error objects
   */
  private serializeError(error: Error | unknown): { message: string; stack?: string; name?: string } | undefined {
    if (!error) return undefined;

    if (error instanceof Error) {
      return {
        name: error.name,
        message: error.message,
        stack: this.isDevelopment ? error.stack : undefined, // Only include stack in development
      };
    }

    // Handle non-Error objects
    if (typeof error === 'object' && error !== null) {
      return {
        name: 'UnknownError',
        message: JSON.stringify(error),
      };
    }

    return {
      name: 'UnknownError',
      message: String(error),
    };
  }

  /**
   * Get recent log history
   */
  getHistory(level?: LogLevel): LogEntry[] {
    if (level) {
      return this.logHistory.filter(entry => entry.level === level);
    }
    return [...this.logHistory];
  }

  /**
   * Clear log history
   */
  clearHistory(): void {
    this.logHistory = [];
  }

  /**
   * Enable debug mode (useful for production debugging)
   */
  enableDebug(): void {
    this.enableConsole = true;
    localStorage.setItem('bear_debug', 'true');
    this.info('Debug mode enabled');
  }

  /**
   * Disable debug mode
   */
  disableDebug(): void {
    this.enableConsole = this.isDevelopment;
    localStorage.removeItem('bear_debug');
  }

  /**
   * Export logs as JSON string (useful for bug reports)
   */
  exportLogs(): string {
    return JSON.stringify(this.logHistory, null, 2);
  }
}

// Export singleton instance
export const logger = new Logger();

// Export for type checking
export type { LogLevel, LogContext, LogEntry };

// Development helper: expose logger to window for debugging
if (import.meta.env.DEV) {
  (window as any).bearLogger = logger;
}
