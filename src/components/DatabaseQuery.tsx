import React, { useState, useRef } from 'react';
import { Database, Play, Download, Upload, AlertCircle, CheckCircle, History, Loader2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface QueryResult {
  columns: string[];
  rows: any[][];
  rowCount: number;
  executionTime: number;
}

interface QueryHistory {
  id: string;
  query: string;
  timestamp: Date;
  success: boolean;
  rowCount?: number;
  error?: string;
}

const DatabaseQuery: React.FC = () => {
  const [query, setQuery] = useState('');
  const [result, setResult] = useState<QueryResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const [queryHistory, setQueryHistory] = useState<QueryHistory[]>([]);
  const [showHistory, setShowHistory] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const executeQuery = async () => {
    if (!query.trim()) {
      setError('Please enter a SQL query');
      return;
    }

    setIsExecuting(true);
    setError(null);
    setResult(null);

    const startTime = Date.now();

    try {
      const result = await invoke<QueryResult>('execute_sql_query', {
        query: query.trim()
      });

      const executionTime = Date.now() - startTime;
      const queryResult = { ...result, executionTime };

      setResult(queryResult);

      // Add to history
      const historyEntry: QueryHistory = {
        id: Date.now().toString(),
        query: query.trim(),
        timestamp: new Date(),
        success: true,
        rowCount: result.rowCount
      };
      setQueryHistory(prev => [historyEntry, ...prev.slice(0, 19)]); // Keep last 20

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Query execution failed';
      setError(errorMessage);

      // Add error to history
      const historyEntry: QueryHistory = {
        id: Date.now().toString(),
        query: query.trim(),
        timestamp: new Date(),
        success: false,
        error: errorMessage
      };
      setQueryHistory(prev => [historyEntry, ...prev.slice(0, 19)]);
    } finally {
      setIsExecuting(false);
    }
  };

  const exportResults = () => {
    if (!result) return;

    const csv = [
      result.columns.join(','),
      ...result.rows.map(row => row.map(cell =>
        typeof cell === 'string' && cell.includes(',') ? `"${cell}"` : cell
      ).join(','))
    ].join('\n');

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `query_results_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      const jsonData = JSON.parse(text);

      // Generate INSERT statements from JSON
      if (Array.isArray(jsonData) && jsonData.length > 0) {
        const tableName = file.name.replace('.json', '').replace(/[^a-zA-Z0-9]/g, '_');
        const columns = Object.keys(jsonData[0]);
        const insertQueries = jsonData.map(row => {
          const values = columns.map(col => {
            const val = row[col];
            return typeof val === 'string' ? `'${val.replace(/'/g, "''")}'` : val;
          });
          return `INSERT INTO ${tableName} (${columns.join(', ')}) VALUES (${values.join(', ')});`;
        });

        setQuery(`-- Generated from ${file.name}\nCREATE TABLE IF NOT EXISTS ${tableName} (\n  ${columns.map(col => `${col} TEXT`).join(',\n  ')}\n);\n\n${insertQueries.join('\n')}`);
      }
    } catch (err) {
      setError('Failed to parse JSON file');
    }
  };

  const loadFromHistory = (historyQuery: string) => {
    setQuery(historyQuery);
    setShowHistory(false);
  };

  const commonQueries = [
    "SELECT name FROM sqlite_master WHERE type='table';",
    "PRAGMA table_info(your_table_name);",
    "SELECT COUNT(*) as total_records FROM your_table_name;",
    "SELECT * FROM your_table_name LIMIT 10;",
  ];

  return (
    <div className="bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-lg p-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Database className="w-5 h-5 text-[var(--accent)]" />
          <h3 className="text-lg font-semibold text-[var(--text-primary)]">
            SQLite Database Query Interface
          </h3>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowHistory(!showHistory)}
            className="p-2 rounded hover:bg-[var(--hover-bg)] transition-colors"
            title="Query History"
          >
            <History className="w-4 h-4 text-[var(--text-secondary)]" />
          </button>
          <input
            type="file"
            ref={fileInputRef}
            onChange={handleFileUpload}
            accept=".json"
            className="hidden"
          />
          <button
            onClick={() => fileInputRef.current?.click()}
            className="p-2 rounded hover:bg-[var(--hover-bg)] transition-colors"
            title="Import JSON Data"
          >
            <Upload className="w-4 h-4 text-[var(--text-secondary)]" />
          </button>
        </div>
      </div>

      {/* Query History Panel */}
      {showHistory && (
        <div className="mb-4 p-4 bg-[var(--bg-tertiary)] rounded-lg border border-[var(--border-secondary)]">
          <h4 className="text-sm font-medium text-[var(--text-primary)] mb-2">Recent Queries</h4>
          <div className="space-y-2 max-h-40 overflow-y-auto">
            {queryHistory.length === 0 ? (
              <p className="text-xs text-[var(--text-tertiary)]">No query history yet</p>
            ) : (
              queryHistory.map((item) => (
                <div
                  key={item.id}
                  className="flex items-start gap-2 p-2 rounded hover:bg-[var(--hover-bg)] cursor-pointer group"
                  onClick={() => loadFromHistory(item.query)}
                >
                  {item.success ? (
                    <CheckCircle className="w-3 h-3 text-green-500 mt-0.5 flex-shrink-0" />
                  ) : (
                    <AlertCircle className="w-3 h-3 text-red-500 mt-0.5 flex-shrink-0" />
                  )}
                  <div className="flex-1 min-w-0">
                    <p className="text-xs text-[var(--text-secondary)] truncate">
                      {item.query}
                    </p>
                    <p className="text-xs text-[var(--text-tertiary)]">
                      {item.timestamp.toLocaleTimeString()} •
                      {item.success ? ` ${item.rowCount} rows` : ` Error: ${item.error}`}
                    </p>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {/* Common Queries */}
      <div className="mb-4">
        <h4 className="text-sm font-medium text-[var(--text-primary)] mb-2">Common Queries</h4>
        <div className="flex flex-wrap gap-2">
          {commonQueries.map((commonQuery, index) => (
            <button
              key={index}
              onClick={() => setQuery(commonQuery)}
              className="px-3 py-1 text-xs bg-[var(--bg-tertiary)] hover:bg-[var(--hover-bg)] border border-[var(--border-secondary)] rounded transition-colors"
            >
              {commonQuery.includes('sqlite_master') ? 'List Tables' :
               commonQuery.includes('table_info') ? 'Table Schema' :
               commonQuery.includes('COUNT') ? 'Count Records' :
               'Preview Data'}
            </button>
          ))}
        </div>
      </div>

      {/* Query Input */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-[var(--text-primary)] mb-2">
          SQL Query
        </label>
        <textarea
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Enter your SQL query here... (SELECT statements only for security)"
          className="w-full h-32 px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-primary)] rounded-lg resize-none font-mono text-sm text-[var(--text-primary)] placeholder-[var(--text-tertiary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
        />
      </div>

      {/* Execute Button */}
      <div className="flex items-center gap-2 mb-4">
        <button
          onClick={executeQuery}
          disabled={isExecuting || !query.trim()}
          className="flex items-center gap-2 px-4 py-2 bg-[var(--accent)] hover:bg-[var(--accent-hover)] disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
        >
          {isExecuting ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Play className="w-4 h-4" />
          )}
          {isExecuting ? 'Executing...' : 'Execute Query'}
        </button>

        {result && (
          <button
            onClick={exportResults}
            className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors"
          >
            <Download className="w-4 h-4" />
            Export CSV
          </button>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-4 h-4 text-red-500" />
            <p className="text-sm text-red-700 dark:text-red-400">{error}</p>
          </div>
        </div>
      )}

      {/* Results Display */}
      {result && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h4 className="text-sm font-medium text-[var(--text-primary)]">Query Results</h4>
            <div className="text-xs text-[var(--text-tertiary)]">
              {result.rowCount} rows • {result.executionTime}ms
            </div>
          </div>

          <div className="border border-[var(--border-primary)] rounded-lg overflow-hidden">
            <div className="overflow-x-auto max-h-96">
              <table className="w-full text-sm">
                <thead className="bg-[var(--bg-tertiary)] border-b border-[var(--border-primary)]">
                  <tr>
                    {result.columns.map((column, index) => (
                      <th
                        key={index}
                        className="px-4 py-2 text-left font-medium text-[var(--text-primary)]"
                      >
                        {column}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {result.rows.map((row, rowIndex) => (
                    <tr
                      key={rowIndex}
                      className="border-b border-[var(--border-secondary)] hover:bg-[var(--hover-bg)]"
                    >
                      {row.map((cell, cellIndex) => (
                        <td
                          key={cellIndex}
                          className="px-4 py-2 text-[var(--text-secondary)] break-words max-w-xs"
                        >
                          {cell === null ? (
                            <span className="text-[var(--text-tertiary)] italic">NULL</span>
                          ) : (
                            String(cell)
                          )}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* Instructions */}
      <div className="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
        <p className="text-xs text-blue-700 dark:text-blue-400">
          💡 <strong>Tips:</strong> Use this interface to query your local SQLite database.
          Only SELECT statements are allowed for security. Upload JSON files to generate INSERT statements.
          Use "List Tables" to see available tables and "Table Schema" to examine table structure.
        </p>
      </div>
    </div>
  );
};

export default DatabaseQuery;