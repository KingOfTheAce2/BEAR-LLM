/**
 * Audit Trail Component
 * GDPR Article 30 - Records of processing activities
 * Displays user's activity audit log for transparency
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { AuditLogEntry, NotificationState } from './types';

interface AuditTrailProps {
  userId: string;
  theme?: 'light' | 'dark';
  onNotification: (message: string, type: NotificationState['type']) => void;
}

const AuditTrail: React.FC<AuditTrailProps> = ({ userId, theme, onNotification }) => {
  const [auditLogs, setAuditLogs] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [filterCategory, setFilterCategory] = useState<string>('all');
  const [limit, setLimit] = useState<number>(50);
  const [expandedEntry, setExpandedEntry] = useState<string | null>(null);

  useEffect(() => {
    loadAuditTrail();
  }, [userId, limit]);

  const loadAuditTrail = async () => {
    try {
      setLoading(true);
      const logs = await invoke<AuditLogEntry[]>('get_audit_trail', { userId, limit });
      setAuditLogs(logs);
      setLoading(false);
    } catch (error) {
      console.error('Failed to load audit trail:', error);
      onNotification('Failed to load activity log', 'error');
      setLoading(false);
    }
  };

  const getCategoryIcon = (category: AuditLogEntry['category']): string => {
    const icons = {
      consent: '✓',
      data_access: '👁',
      data_export: '⬇',
      data_deletion: '🗑',
      settings_change: '⚙'
    };
    return icons[category] || '📋';
  };

  const getCategoryColor = (category: AuditLogEntry['category']): string => {
    const colors = {
      consent: 'category--consent',
      data_access: 'category--access',
      data_export: 'category--export',
      data_deletion: 'category--deletion',
      settings_change: 'category--settings'
    };
    return colors[category] || '';
  };

  const filteredLogs = filterCategory === 'all'
    ? auditLogs
    : auditLogs.filter(log => log.category === filterCategory);

  const toggleEntryDetails = (entryId: string) => {
    setExpandedEntry(expandedEntry === entryId ? null : entryId);
  };

  if (loading) {
    return (
      <div className="audit-trail__loading">
        <div className="spinner" role="status">
          <div className="spinner__circle"></div>
        </div>
        <p>Loading activity log...</p>
      </div>
    );
  }

  return (
    <div className="audit-trail">
      <header className="audit-trail__header">
        <h2>Activity Audit Trail</h2>
        <p className="audit-trail__description">
          Complete log of all privacy-related activities on your account.
          This ensures transparency and compliance with GDPR Article 30.
        </p>
      </header>

      {/* Filters */}
      <div className="audit-trail__filters">
        <div className="filter-group">
          <label htmlFor="category-filter" className="filter-label">
            Filter by category:
          </label>
          <select
            id="category-filter"
            value={filterCategory}
            onChange={(e) => setFilterCategory(e.target.value)}
            className="filter-select"
          >
            <option value="all">All Activities ({auditLogs.length})</option>
            <option value="consent">Consent Changes ({auditLogs.filter(l => l.category === 'consent').length})</option>
            <option value="data_access">Data Access ({auditLogs.filter(l => l.category === 'data_access').length})</option>
            <option value="data_export">Data Exports ({auditLogs.filter(l => l.category === 'data_export').length})</option>
            <option value="data_deletion">Deletion Requests ({auditLogs.filter(l => l.category === 'data_deletion').length})</option>
            <option value="settings_change">Settings Changes ({auditLogs.filter(l => l.category === 'settings_change').length})</option>
          </select>
        </div>

        <div className="filter-group">
          <label htmlFor="limit-filter" className="filter-label">
            Show entries:
          </label>
          <select
            id="limit-filter"
            value={limit}
            onChange={(e) => setLimit(Number(e.target.value))}
            className="filter-select"
          >
            <option value={25}>Last 25</option>
            <option value={50}>Last 50</option>
            <option value={100}>Last 100</option>
            <option value={500}>Last 500</option>
          </select>
        </div>

        <button onClick={loadAuditTrail} className="btn btn--secondary btn--small" aria-label="Refresh audit log">
          🔄 Refresh
        </button>
      </div>

      {/* Audit Log Entries */}
      {filteredLogs.length > 0 ? (
        <div className="audit-log">
          <div className="audit-log__timeline">
            {filteredLogs.map((entry) => (
              <article key={entry.id} className="audit-entry">
                <div className="audit-entry__marker">
                  <span className={`audit-entry__icon ${getCategoryColor(entry.category)}`}>
                    {getCategoryIcon(entry.category)}
                  </span>
                  {entry.complianceRelated && (
                    <span className="audit-entry__badge" title="GDPR compliance related">
                      GDPR
                    </span>
                  )}
                </div>

                <div className="audit-entry__content">
                  <header className="audit-entry__header">
                    <h3 className="audit-entry__action">{entry.action}</h3>
                    <time className="audit-entry__timestamp" dateTime={entry.timestamp}>
                      {new Date(entry.timestamp).toLocaleString()}
                    </time>
                  </header>

                  <div className="audit-entry__metadata">
                    <span className="audit-entry__category">
                      {entry.category.replace(/_/g, ' ')}
                    </span>
                    <span className="audit-entry__separator">•</span>
                    <span className="audit-entry__ip" title="IP Address">
                      <code>{entry.ipAddress}</code>
                    </span>
                  </div>

                  {Object.keys(entry.details).length > 0 && (
                    <>
                      <button
                        onClick={() => toggleEntryDetails(entry.id)}
                        className="audit-entry__toggle"
                        aria-expanded={expandedEntry === entry.id}
                        aria-controls={`details-${entry.id}`}
                      >
                        {expandedEntry === entry.id ? '▼' : '▶'} View Details
                      </button>

                      {expandedEntry === entry.id && (
                        <div id={`details-${entry.id}`} className="audit-entry__details">
                          <h4>Details:</h4>
                          <dl className="details-list">
                            {Object.entries(entry.details).map(([key, value]) => (
                              <div key={key} className="details-list__item">
                                <dt className="details-list__label">
                                  {key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase())}:
                                </dt>
                                <dd className="details-list__value">
                                  {typeof value === 'object'
                                    ? <pre>{JSON.stringify(value, null, 2)}</pre>
                                    : String(value)
                                  }
                                </dd>
                              </div>
                            ))}
                          </dl>

                          <div className="audit-entry__technical">
                            <h4>Technical Information:</h4>
                            <p><strong>User Agent:</strong> <code>{entry.userAgent}</code></p>
                            <p><strong>Entry ID:</strong> <code>{entry.id}</code></p>
                          </div>
                        </div>
                      )}
                    </>
                  )}
                </div>
              </article>
            ))}
          </div>
        </div>
      ) : (
        <div className="audit-trail__empty">
          <p>No activity log entries found for the selected filter.</p>
        </div>
      )}

      {/* Footer */}
      <footer className="audit-trail__footer">
        <div className="info-box info-box--info">
          <strong>Audit Trail Information:</strong>
          <ul>
            <li>All privacy-related activities are automatically logged</li>
            <li>Logs are retained for compliance purposes</li>
            <li>You can export your complete audit trail as part of data export</li>
            <li>Entries marked with "GDPR" are compliance-critical actions</li>
          </ul>
        </div>

        {filteredLogs.length > 0 && (
          <p className="audit-trail__stats">
            Showing {filteredLogs.length} of {auditLogs.length} total entries
          </p>
        )}
      </footer>
    </div>
  );
};

export default AuditTrail;
