/**
 * Export Panel Component
 * GDPR Article 20 - Right to data portability
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { ExportRequest, ExportResult, ExportFormat, NotificationState } from './types';

interface ExportPanelProps {
  userId: string;
  theme?: 'light' | 'dark';
  onNotification: (message: string, type: NotificationState['type']) => void;
}

const ExportPanel: React.FC<ExportPanelProps> = ({ userId, theme, onNotification }) => {
  const [selectedFormats, setSelectedFormats] = useState<Set<ExportFormat>>(new Set(['json']));
  const [includeCompliance, setIncludeCompliance] = useState<boolean>(true);
  const [includeAuditTrail, setIncludeAuditTrail] = useState<boolean>(false);
  const [exports, setExports] = useState<ExportResult[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [polling, setPolling] = useState<boolean>(false);

  const formatOptions: { value: ExportFormat; label: string; description: string }[] = [
    { value: 'json', label: 'JSON', description: 'Machine-readable format, best for developers' },
    { value: 'csv', label: 'CSV', description: 'Spreadsheet format, opens in Excel/Google Sheets' },
    { value: 'xml', label: 'XML', description: 'Structured markup format' },
    { value: 'pdf', label: 'PDF', description: 'Human-readable document format' }
  ];

  useEffect(() => {
    loadExportHistory();
  }, [userId]);

  useEffect(() => {
    // Poll for export status updates
    let interval: NodeJS.Timeout;

    if (polling && exports.some(e => e.status === 'pending' || e.status === 'processing')) {
      interval = setInterval(() => {
        loadExportHistory();
      }, 3000);
    }

    return () => {
      if (interval) clearInterval(interval);
    };
  }, [polling, exports]);

  const loadExportHistory = async () => {
    try {
      // This would normally call a backend function to get export history
      // For now, we'll use local state
      setPolling(false);
    } catch (error) {
      console.error('Failed to load export history:', error);
    }
  };

  const toggleFormat = (format: ExportFormat) => {
    setSelectedFormats(prev => {
      const next = new Set(prev);
      if (next.has(format)) {
        if (next.size > 1) {
          next.delete(format);
        } else {
          onNotification('At least one format must be selected', 'warning');
        }
      } else {
        next.add(format);
      }
      return next;
    });
  };

  const handleExport = async () => {
    if (selectedFormats.size === 0) {
      onNotification('Please select at least one export format', 'warning');
      return;
    }

    setLoading(true);
    try {
      const request: ExportRequest = {
        userId,
        formats: Array.from(selectedFormats),
        includeComplianceData: includeCompliance,
        includeAuditTrail
      };

      const result = await invoke<ExportResult>('export_user_data', { request });

      setExports(prev => [result, ...prev]);
      setPolling(true);

      onNotification(
        `Export request submitted. You'll receive ${selectedFormats.size} file(s).`,
        'success'
      );
    } catch (error) {
      console.error('Export failed:', error);
      onNotification('Failed to create export. Please try again.', 'error');
    } finally {
      setLoading(false);
    }
  };

  const downloadExport = (exportResult: ExportResult) => {
    if (exportResult.status === 'completed' && exportResult.downloadUrl) {
      window.open(exportResult.downloadUrl, '_blank');
      onNotification('Download started', 'success');
    }
  };

  const formatFileSize = (bytes?: number): string => {
    if (!bytes) return 'Unknown';
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(2)} MB`;
  };

  const getStatusBadge = (status: ExportResult['status']) => {
    const badges = {
      pending: { class: 'badge--warning', label: 'Pending' },
      processing: { class: 'badge--info', label: 'Processing' },
      completed: { class: 'badge--success', label: 'Completed' },
      failed: { class: 'badge--error', label: 'Failed' }
    };
    return badges[status];
  };

  return (
    <div className="export-panel">
      <header className="export-panel__header">
        <h2>Data Export</h2>
        <p className="export-panel__description">
          Download a complete copy of your personal data in your preferred format.
          This complies with GDPR Article 20 (Right to Data Portability).
        </p>
      </header>

      {/* Export Configuration */}
      <section className="export-config">
        <h3>Export Configuration</h3>

        {/* Format Selection */}
        <div className="form-group">
          <label className="form-label">Select Export Formats:</label>
          <div className="checkbox-group">
            {formatOptions.map(option => (
              <label key={option.value} className="checkbox-item">
                <input
                  type="checkbox"
                  checked={selectedFormats.has(option.value)}
                  onChange={() => toggleFormat(option.value)}
                  aria-describedby={`format-${option.value}-desc`}
                />
                <div className="checkbox-item__content">
                  <strong>{option.label}</strong>
                  <p id={`format-${option.value}-desc`} className="checkbox-item__description">
                    {option.description}
                  </p>
                </div>
              </label>
            ))}
          </div>
        </div>

        {/* Additional Options */}
        <div className="form-group">
          <label className="form-label">Additional Options:</label>
          <div className="checkbox-group">
            <label className="checkbox-item checkbox-item--compact">
              <input
                type="checkbox"
                checked={includeCompliance}
                onChange={(e) => setIncludeCompliance(e.target.checked)}
              />
              <span>Include compliance metadata (consent records, processing history)</span>
            </label>
            <label className="checkbox-item checkbox-item--compact">
              <input
                type="checkbox"
                checked={includeAuditTrail}
                onChange={(e) => setIncludeAuditTrail(e.target.checked)}
              />
              <span>Include complete audit trail</span>
            </label>
          </div>
        </div>

        {/* Export Button */}
        <div className="export-config__actions">
          <button
            onClick={handleExport}
            disabled={loading || selectedFormats.size === 0}
            className="btn btn--primary btn--large"
            aria-busy={loading}
          >
            {loading ? (
              <>
                <span className="spinner spinner--small"></span>
                Creating Export...
              </>
            ) : (
              <>
                ⬇ Create Export ({selectedFormats.size} format{selectedFormats.size !== 1 ? 's' : ''})
              </>
            )}
          </button>
        </div>
      </section>

      {/* Export History */}
      {exports.length > 0 && (
        <section className="export-history">
          <h3>Export History</h3>
          <div className="export-list">
            {exports.map((exportResult, idx) => {
              const badge = getStatusBadge(exportResult.status);
              const isDownloadable = exportResult.status === 'completed' && exportResult.downloadUrl;

              return (
                <div key={exportResult.requestId || idx} className="export-item">
                  <div className="export-item__header">
                    <div className="export-item__info">
                      <h4 className="export-item__format">{exportResult.format.toUpperCase()}</h4>
                      <span className={`badge ${badge.class}`}>{badge.label}</span>
                    </div>
                    <time className="export-item__date">
                      {new Date(exportResult.createdAt).toLocaleString()}
                    </time>
                  </div>

                  <div className="export-item__details">
                    {exportResult.fileSize && (
                      <span className="export-item__size">Size: {formatFileSize(exportResult.fileSize)}</span>
                    )}
                    {exportResult.expiresAt && (
                      <span className="export-item__expiry">
                        Expires: {new Date(exportResult.expiresAt).toLocaleString()}
                      </span>
                    )}
                  </div>

                  {exportResult.error && (
                    <div className="export-item__error">
                      <strong>Error:</strong> {exportResult.error}
                    </div>
                  )}

                  {isDownloadable && (
                    <div className="export-item__actions">
                      <button
                        onClick={() => downloadExport(exportResult)}
                        className="btn btn--secondary btn--small"
                      >
                        ⬇ Download
                      </button>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </section>
      )}

      {/* Information */}
      <footer className="export-panel__footer">
        <div className="info-box info-box--info">
          <strong>Export Information:</strong>
          <ul>
            <li>Exports are generated in real-time and may take a few minutes</li>
            <li>Download links expire after 7 days for security</li>
            <li>All exports are encrypted and secure</li>
            <li>You can create multiple exports in different formats</li>
          </ul>
        </div>
      </footer>
    </div>
  );
};

export default ExportPanel;
