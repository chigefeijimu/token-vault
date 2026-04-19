// Frontend logging module for tracking application state and debugging

export enum LogLevel {
  TRACE = 'TRACE',
  DEBUG = 'DEBUG',
  INFO = 'INFO',
  WARN = 'WARN',
  ERROR = 'ERROR',
}

export interface LogEntry {
  id: string;
  timestamp: string;
  level: LogLevel;
  module: string;
  message: string;
  context?: string;
  data?: unknown;
}

export interface LoggerConfig {
  level: LogLevel;
  enableConsole: boolean;
  enableBackend: boolean;
  maxEntries: number;
}

const defaultConfig: LoggerConfig = {
  level: LogLevel.INFO,
  enableConsole: true,
  enableBackend: true,
  maxEntries: 1000,
};

class Logger {
  private config: LoggerConfig;
  private entries: LogEntry[] = [];
  private listeners: ((entry: LogEntry) => void)[] = [];

  constructor(config: Partial<LoggerConfig> = {}) {
    this.config = { ...defaultConfig, ...config };
  }

  private generateId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private getTimestamp(): string {
    return new Date().toISOString();
  }

  private shouldLog(level: LogLevel): boolean {
    const levels = [LogLevel.TRACE, LogLevel.DEBUG, LogLevel.INFO, LogLevel.WARN, LogLevel.ERROR];
    const configLevel = levels.indexOf(this.config.level);
    const messageLevel = levels.indexOf(level);
    return messageLevel >= configLevel;
  }

  private createEntry(level: LogLevel, module: string, message: string, context?: string, data?: unknown): LogEntry {
    return {
      id: this.generateId(),
      timestamp: this.getTimestamp(),
      level,
      module,
      message,
      context,
      data,
    };
  }

  private addEntry(entry: LogEntry): void {
    this.entries.push(entry);
    if (this.entries.length > this.config.maxEntries) {
      this.entries = this.entries.slice(-this.config.maxEntries);
    }
    this.notifyListeners(entry);
  }

  private notifyListeners(entry: LogEntry): void {
    this.listeners.forEach((listener) => listener(entry));
  }

  private log(level: LogLevel, module: string, message: string, context?: string, data?: unknown): void {
    if (!this.shouldLog(level)) return;

    const entry = this.createEntry(level, module, message, context, data);

    if (this.config.enableConsole) {
      const prefix = `[${entry.timestamp}] [${entry.level}] [${module}]`;
      const contextStr = context ? ` [${context}]` : '';
      const logMessage = `${prefix}${contextStr} ${message}`;

      switch (level) {
        case LogLevel.ERROR:
          console.error(logMessage, data ?? '');
          break;
        case LogLevel.WARN:
          console.warn(logMessage, data ?? '');
          break;
        case LogLevel.DEBUG:
          console.debug(logMessage, data ?? '');
          break;
        default:
          console.log(logMessage, data ?? '');
      }
    }

    this.addEntry(entry);
  }

  // Public logging methods
  trace(module: string, message: string, context?: string, data?: unknown): void {
    this.log(LogLevel.TRACE, module, message, context, data);
  }

  debug(module: string, message: string, context?: string, data?: unknown): void {
    this.log(LogLevel.DEBUG, module, message, context, data);
  }

  info(module: string, message: string, context?: string, data?: unknown): void {
    this.log(LogLevel.INFO, module, message, context, data);
  }

  warn(module: string, message: string, context?: string, data?: unknown): void {
    this.log(LogLevel.WARN, module, message, context, data);
  }

  error(module: string, message: string, context?: string, data?: unknown): void {
    this.log(LogLevel.ERROR, module, message, context, data);
  }

  // Configuration methods
  setLevel(level: LogLevel): void {
    this.config.level = level;
  }

  getLevel(): LogLevel {
    return this.config.level;
  }

  setConfig(config: Partial<LoggerConfig>): void {
    this.config = { ...this.config, ...config };
  }

  // Entry management
  getEntries(limit?: number): LogEntry[] {
    if (limit) {
      return this.entries.slice(-limit);
    }
    return [...this.entries];
  }

  getEntriesByLevel(level: LogLevel): LogEntry[] {
    return this.entries.filter((e) => e.level === level);
  }

  getEntriesByModule(module: string): LogEntry[] {
    return this.entries.filter((e) => e.module === module);
  }

  clearEntries(): void {
    this.entries = [];
  }

  // Subscription
  subscribe(listener: (entry: LogEntry) => void): () => void {
    this.listeners.push(listener);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }
}

// Singleton instance
export const logger = new Logger();

// Helper functions for quick logging
export const log = {
  trace: (module: string, message: string, context?: string, data?: unknown) =>
    logger.trace(module, message, context, data),
  debug: (module: string, message: string, context?: string, data?: unknown) =>
    logger.debug(module, message, context, data),
  info: (module: string, message: string, context?: string, data?: unknown) =>
    logger.info(module, message, context, data),
  warn: (module: string, message: string, context?: string, data?: unknown) =>
    logger.warn(module, message, context, data),
  error: (module: string, message: string, context?: string, data?: unknown) =>
    logger.error(module, message, context, data),
};

// Module-specific loggers factory
export function createModuleLogger(moduleName: string) {
  return {
    trace: (message: string, context?: string, data?: unknown) =>
      logger.trace(moduleName, message, context, data),
    debug: (message: string, context?: string, data?: unknown) =>
      logger.debug(moduleName, message, context, data),
    info: (message: string, context?: string, data?: unknown) =>
      logger.info(moduleName, message, context, data),
    warn: (message: string, context?: string, data?: unknown) =>
      logger.warn(moduleName, message, context, data),
    error: (message: string, context?: string, data?: unknown) =>
      logger.error(moduleName, message, context, data),
  };
}