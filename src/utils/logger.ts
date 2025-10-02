/**
 * Simple logging service with export capability
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  context?: any;
  error?: {
    message: string;
    stack?: string;
    name?: string;
  };
}

class Logger {
  private logHistory: LogEntry[] = [];
  private maxHistorySize = 200; // Keep last 200 logs

  debug(message: string, context?: any): void {
    this.log('debug', message, context);
  }

  info(message: string, context?: any): void {
    this.log('info', message, context);
  }

  warn(message: string, context?: any): void {
    this.log('warn', message, context);
  }

  error(message: string, error?: Error | unknown, context?: any): void {
    const errorInfo = this.serializeError(error);
    this.log('error', message, context, errorInfo);
  }

  private log(level: LogLevel, message: string, context?: any, error?: any): void {
    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      context,
      error,
    };

    // Always store in history
    this.logHistory.push(entry);
    if (this.logHistory.length > this.maxHistorySize) {
      this.logHistory.shift();
    }

    // Log to console
    const prefix = `[BEAR AI]`;
    switch (level) {
      case 'debug':
        console.debug(prefix, message, context || '');
        break;
      case 'info':
        console.info(prefix, message, context || '');
        break;
      case 'warn':
        console.warn(prefix, message, context || '');
        break;
      case 'error':
        console.error(prefix, message, error || '', context || '');
        break;
    }
  }

  private serializeError(error: Error | unknown): any {
    if (!error) return undefined;

    if (error instanceof Error) {
      return {
        name: error.name,
        message: error.message,
        stack: error.stack,
      };
    }

    return {
      name: 'UnknownError',
      message: String(error),
    };
  }

  /**
   * Export logs as downloadable file
   */
  exportLogs(): void {
    const logData = {
      exportDate: new Date().toISOString(),
      totalLogs: this.logHistory.length,
      logs: this.logHistory,
    };

    const blob = new Blob([JSON.stringify(logData, null, 2)], {
      type: 'application/json',
    });

    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `bear-logs-${new Date().toISOString().split('T')[0]}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }
}

export const logger = new Logger();

// Expose to window for easy access
declare global {
  interface Window {
    bearLogger: {
      exportLogs: () => void;
    };
  }
}

(window as any).bearLogger = {
  exportLogs: () => logger.exportLogs(),
};

export type { LogLevel, LogEntry };
