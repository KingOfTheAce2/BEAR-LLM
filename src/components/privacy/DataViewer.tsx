/**
 * Data Viewer Component
 * GDPR Article 15 - Right of access to personal data
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { UserData, NotificationState } from './types';

interface DataViewerProps {
  userId: string;
  theme?: 'light' | 'dark';
  onNotification: (message: string, type: NotificationState['type']) => void;
}

const DataViewer: React.FC<DataViewerProps> = ({ userId, theme, onNotification }) => {
  const [userData, setUserData] = useState<UserData | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['personalInfo']));

  useEffect(() => {
    loadUserData();
  }, [userId]);

  const loadUserData = async () => {
    try {
      setLoading(true);
      // Get export preview to view user data without creating export
      const preview = await invoke<UserData>('get_export_preview', { userId });
      setUserData(preview);
      setLoading(false);
    } catch (error) {
      console.error('Failed to load user data:', error);
      onNotification('Failed to load your data', 'error');
      setLoading(false);
    }
  };

  const toggleSection = (section: string) => {
    setExpandedSections(prev => {
      const next = new Set(prev);
      if (next.has(section)) {
        next.delete(section);
      } else {
        next.add(section);
      }
      return next;
    });
  };

  const renderValue = (value: unknown): React.ReactNode => {
    if (value === null || value === undefined) {
      return <span className="data-value data-value--empty">Not set</span>;
    }

    if (typeof value === 'boolean') {
      return <span className="data-value data-value--boolean">{value ? 'Yes' : 'No'}</span>;
    }

    if (typeof value === 'object') {
      return <pre className="data-value data-value--json">{JSON.stringify(value, null, 2)}</pre>;
    }

    if (typeof value === 'string' && value.match(/^\d{4}-\d{2}-\d{2}/)) {
      return <span className="data-value data-value--date">{new Date(value).toLocaleString()}</span>;
    }

    return <span className="data-value">{String(value)}</span>;
  };

  if (loading) {
    return (
      <div className="data-viewer__loading">
        <div className="spinner" role="status">
          <div className="spinner__circle"></div>
        </div>
        <p>Loading your data...</p>
      </div>
    );
  }

  if (!userData) {
    return (
      <div className="data-viewer__error">
        <p>No data available to display</p>
        <button onClick={loadUserData} className="btn btn--primary">
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="data-viewer">
      <header className="data-viewer__header">
        <h2>Your Personal Data</h2>
        <p className="data-viewer__description">
          View all personal data we store about you. This is provided in compliance with GDPR Article 15 (Right of Access).
        </p>
        <button onClick={loadUserData} className="btn btn--secondary" aria-label="Refresh data">
          🔄 Refresh
        </button>
      </header>

      {/* Personal Information */}
      <section className="data-section">
        <button
          className="data-section__header"
          onClick={() => toggleSection('personalInfo')}
          aria-expanded={expandedSections.has('personalInfo')}
          aria-controls="section-personalInfo"
        >
          <span className="data-section__icon">{expandedSections.has('personalInfo') ? '▼' : '▶'}</span>
          <h3>Personal Information</h3>
          <span className="data-section__count">
            {Object.keys(userData.personalInfo).length} fields
          </span>
        </button>

        {expandedSections.has('personalInfo') && (
          <div id="section-personalInfo" className="data-section__content">
            <dl className="data-list">
              {Object.entries(userData.personalInfo).map(([key, value]) => (
                <div key={key} className="data-list__item">
                  <dt className="data-list__label">{key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase())}:</dt>
                  <dd className="data-list__value">{renderValue(value)}</dd>
                </div>
              ))}
            </dl>
          </div>
        )}
      </section>

      {/* Activity Data */}
      <section className="data-section">
        <button
          className="data-section__header"
          onClick={() => toggleSection('activityData')}
          aria-expanded={expandedSections.has('activityData')}
          aria-controls="section-activityData"
        >
          <span className="data-section__icon">{expandedSections.has('activityData') ? '▼' : '▶'}</span>
          <h3>Activity Data</h3>
          <span className="data-section__count">
            {userData.activityData.loginHistory.length} login records,
            {userData.activityData.searchHistory.length} searches
          </span>
        </button>

        {expandedSections.has('activityData') && (
          <div id="section-activityData" className="data-section__content">
            {/* Login History */}
            <div className="data-subsection">
              <h4>Login History (Last 10)</h4>
              {userData.activityData.loginHistory.length > 0 ? (
                <div className="data-table-wrapper">
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Timestamp</th>
                        <th>IP Address</th>
                        <th>Location</th>
                        <th>Status</th>
                      </tr>
                    </thead>
                    <tbody>
                      {userData.activityData.loginHistory.slice(0, 10).map((login, idx) => (
                        <tr key={idx}>
                          <td>{new Date(login.timestamp).toLocaleString()}</td>
                          <td><code>{login.ipAddress}</code></td>
                          <td>{login.location || 'Unknown'}</td>
                          <td>
                            <span className={`badge badge--${login.success ? 'success' : 'error'}`}>
                              {login.success ? 'Success' : 'Failed'}
                            </span>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <p className="data-empty">No login history available</p>
              )}
            </div>

            {/* Search History */}
            <div className="data-subsection">
              <h4>Search History (Last 10)</h4>
              {userData.activityData.searchHistory.length > 0 ? (
                <div className="data-table-wrapper">
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Timestamp</th>
                        <th>Query</th>
                        <th>Results</th>
                      </tr>
                    </thead>
                    <tbody>
                      {userData.activityData.searchHistory.slice(0, 10).map((search, idx) => (
                        <tr key={idx}>
                          <td>{new Date(search.timestamp).toLocaleString()}</td>
                          <td>{search.query}</td>
                          <td>{search.resultsCount}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <p className="data-empty">No search history available</p>
              )}
            </div>

            {/* Preferences */}
            <div className="data-subsection">
              <h4>Preferences</h4>
              <pre className="data-value data-value--json">
                {JSON.stringify(userData.activityData.preferences, null, 2)}
              </pre>
            </div>
          </div>
        )}
      </section>

      {/* Metadata */}
      <section className="data-section">
        <button
          className="data-section__header"
          onClick={() => toggleSection('metadata')}
          aria-expanded={expandedSections.has('metadata')}
          aria-controls="section-metadata"
        >
          <span className="data-section__icon">{expandedSections.has('metadata') ? '▼' : '▶'}</span>
          <h3>Account Metadata</h3>
        </button>

        {expandedSections.has('metadata') && (
          <div id="section-metadata" className="data-section__content">
            <dl className="data-list">
              <div className="data-list__item">
                <dt className="data-list__label">Account Created:</dt>
                <dd className="data-list__value">{new Date(userData.metadata.createdAt).toLocaleString()}</dd>
              </div>
              <div className="data-list__item">
                <dt className="data-list__label">Last Modified:</dt>
                <dd className="data-list__value">{new Date(userData.metadata.lastModified).toLocaleString()}</dd>
              </div>
              <div className="data-list__item">
                <dt className="data-list__label">Data Version:</dt>
                <dd className="data-list__value"><code>{userData.metadata.dataVersion}</code></dd>
              </div>
            </dl>
          </div>
        )}
      </section>

      <footer className="data-viewer__footer">
        <div className="info-box info-box--info">
          <strong>Data Accuracy:</strong> If you notice any inaccuracies in your data, please contact support to request corrections.
          You have the right to rectification under GDPR Article 16.
        </div>
      </footer>
    </div>
  );
};

export default DataViewer;
