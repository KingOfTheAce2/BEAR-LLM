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
  private debugToken: string | null = null;
  private readonly DEBUG_TOKEN_KEY = 'bear_debug_token';
  private readonly DEBUG_ENABLED_KEY = 'bear_debug_enabled';

  constructor() {
    // Detect environment - use build-time environment variable
    this.isDevelopment = import.meta.env.DEV || import.meta.env.MODE === 'development';

    // Production debug mode with security token
    // Only enable console logging in production if:
    // 1. It's development mode, OR
    // 2. Valid debug token exists and debug is explicitly enabled
    this.enableConsole = this.isDevelopment || this.validateDebugMode();

    // Generate debug token on first run in production
    if (!this.isDevelopment && !this.getDebugToken()) {
      this.generateDebugToken();
    }
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
   * Send logs to monitoring service
   */
  private sendToMonitoring(entry: LogEntry): void {
    // Only log errors to monitoring
    if (entry.level !== 'error') return;

    try {
      // Store in localStorage for debugging and analysis
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
   * Generate a secure debug token for production debugging
   */
  private generateDebugToken(): string {
    // Generate a cryptographically secure random token
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    const token = Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');

    try {
      localStorage.setItem(this.DEBUG_TOKEN_KEY, token);
      this.debugToken = token;
      return token;
    } catch (err) {
      console.error('Failed to generate debug token:', err);
      return '';
    }
  }

  /**
   * Get the current debug token
   */
  private getDebugToken(): string | null {
    if (this.debugToken) return this.debugToken;

    try {
      this.debugToken = localStorage.getItem(this.DEBUG_TOKEN_KEY);
      return this.debugToken;
    } catch {
      return null;
    }
  }

  /**
   * Validate debug mode is properly enabled
   */
  private validateDebugMode(): boolean {
    try {
      const isEnabled = localStorage.getItem(this.DEBUG_ENABLED_KEY) === 'true';
      const hasToken = !!this.getDebugToken();
      return isEnabled && hasToken;
    } catch {
      return false;
    }
  }

  /**
   * Enable debug mode (useful for production debugging)
   * Requires the debug token for security
   */
  enableDebug(token?: string): boolean {
    if (this.isDevelopment) {
      this.enableConsole = true;
      this.info('Debug mode enabled (development)');
      return true;
    }

    // In production, verify token
    const storedToken = this.getDebugToken();
    if (!storedToken) {
      console.error('No debug token found. Cannot enable debug mode.');
      return false;
    }

    // If token provided, validate it
    if (token && token !== storedToken) {
      console.error('Invalid debug token. Debug mode not enabled.');
      return false;
    }

    // If no token provided but debug was previously enabled with valid token, allow it
    if (!token && !this.validateDebugMode()) {
      console.error('Debug mode requires token. Call enableDebug(token) with your debug token.');
      console.info('Debug token can be retrieved with: window.bearLogger.getDebugToken()');
      return false;
    }

    this.enableConsole = true;
    localStorage.setItem(this.DEBUG_ENABLED_KEY, 'true');
    this.info('Debug mode enabled (production)');
    return true;
  }

  /**
   * Disable debug mode
   */
  disableDebug(): void {
    this.enableConsole = this.isDevelopment;
    localStorage.removeItem(this.DEBUG_ENABLED_KEY);
    this.info('Debug mode disabled');
  }

  /**
   * Get debug token (only works if debug is already enabled or in development)
   */
  getPublicDebugToken(): string | null {
    if (this.isDevelopment || this.validateDebugMode()) {
      const token = this.getDebugToken();
      if (token) {
        console.info('🔑 Debug Token:', token);
        console.info('To enable debug mode: window.bearLogger.enableDebug("' + token + '")');
      }
      return token;
    }
    console.error('Debug mode must be enabled first to retrieve token.');
    return null;
  }

  /**
   * Rotate debug token (generates new token and disables debug mode)
   */
  rotateDebugToken(): string {
    this.disableDebug();
    localStorage.removeItem(this.DEBUG_TOKEN_KEY);
    this.debugToken = null;
    const newToken = this.generateDebugToken();
    this.info('Debug token rotated. Use new token to enable debug mode.');
    return newToken;
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

// Expose logger to window for debugging (limited access in production)
declare global {
  interface Window {
    bearLogger: {
      enableDebug: (token?: string) => boolean;
      disableDebug: () => void;
      getDebugToken: () => string | null;
      rotateDebugToken: () => string;
      exportLogs: () => string;
      getHistory: (level?: LogLevel) => LogEntry[];
    };
  }
}

// Development: expose full logger
if (import.meta.env.DEV) {
  (window as any).bearLogger = logger;
} else {
  // Production: expose only safe debugging methods
  (window as any).bearLogger = {
    enableDebug: (token?: string) => logger.enableDebug(token),
    disableDebug: () => logger.disableDebug(),
    getDebugToken: () => logger.getPublicDebugToken(),
    rotateDebugToken: () => logger.rotateDebugToken(),
    exportLogs: () => logger.exportLogs(),
    getHistory: (level?: LogLevel) => logger.getHistory(level),
  };
}
