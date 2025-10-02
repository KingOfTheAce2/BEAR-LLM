/**
 * Main Privacy Dashboard Component
 * GDPR-compliant user privacy control center
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import ConsentManager from './ConsentManager';
import DataViewer from './DataViewer';
import ExportPanel from './ExportPanel';
import DeletionRequest from './DeletionRequest';
import AuditTrail from './AuditTrail';
import RetentionSettings from './RetentionSettings';
import { PrivacyDashboardProps, NotificationState } from './types';
import './styles.css';

const PrivacyDashboard: React.FC<PrivacyDashboardProps> = ({ userId, theme = 'light', onError }) => {
  const [activeTab, setActiveTab] = useState<string>('consents');
  const [loading, setLoading] = useState<boolean>(true);
  const [notification, setNotification] = useState<NotificationState>({
    show: false,
    message: '',
    type: 'info'
  });

  useEffect(() => {
    // Initialize dashboard
    const initDashboard = async () => {
      try {
        setLoading(true);
        // Pre-load critical data
        await Promise.all([
          invoke('check_consent_status', { userId }),
          invoke('get_audit_trail', { userId, limit: 10 })
        ]);
        setLoading(false);
      } catch (error) {
        console.error('Failed to initialize privacy dashboard:', error);
        showNotification('Failed to load privacy dashboard', 'error');
        onError?.(error as Error);
        setLoading(false);
      }
    };

    initDashboard();
  }, [userId, onError]);

  const showNotification = (message: string, type: NotificationState['type']) => {
    setNotification({ show: true, message, type });
    setTimeout(() => {
      setNotification(prev => ({ ...prev, show: false }));
    }, 5000);
  };

  const tabs = [
    { id: 'consents', label: 'Consent Management', icon: '✓' },
    { id: 'data', label: 'My Data', icon: '📊' },
    { id: 'export', label: 'Data Export', icon: '⬇' },
    { id: 'deletion', label: 'Delete My Data', icon: '🗑' },
    { id: 'audit', label: 'Activity Log', icon: '📜' },
    { id: 'retention', label: 'Data Retention', icon: '⏱' }
  ];

  if (loading) {
    return (
      <div className={`privacy-dashboard privacy-dashboard--${theme}`}>
        <div className="privacy-dashboard__loading">
          <div className="spinner" role="status" aria-label="Loading privacy dashboard">
            <div className="spinner__circle"></div>
          </div>
          <p>Loading your privacy dashboard...</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`privacy-dashboard privacy-dashboard--${theme}`}>
      {/* Header */}
      <header className="privacy-dashboard__header">
        <h1>Privacy & Data Control Center</h1>
        <p className="privacy-dashboard__subtitle">
          Manage your data, consents, and privacy preferences in compliance with GDPR
        </p>
      </header>

      {/* Notification Toast */}
      {notification.show && (
        <div
          className={`notification notification--${notification.type}`}
          role="alert"
          aria-live="polite"
        >
          <span className="notification__icon">
            {notification.type === 'success' && '✓'}
            {notification.type === 'error' && '✕'}
            {notification.type === 'warning' && '⚠'}
            {notification.type === 'info' && 'ℹ'}
          </span>
          <span className="notification__message">{notification.message}</span>
          <button
            className="notification__close"
            onClick={() => setNotification(prev => ({ ...prev, show: false }))}
            aria-label="Close notification"
          >
            ✕
          </button>
        </div>
      )}

      {/* Tab Navigation */}
      <nav className="privacy-dashboard__tabs" role="tablist" aria-label="Privacy dashboard sections">
        {tabs.map(tab => (
          <button
            key={tab.id}
            role="tab"
            aria-selected={activeTab === tab.id}
            aria-controls={`panel-${tab.id}`}
            id={`tab-${tab.id}`}
            className={`tab ${activeTab === tab.id ? 'tab--active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            <span className="tab__icon" aria-hidden="true">{tab.icon}</span>
            <span className="tab__label">{tab.label}</span>
          </button>
        ))}
      </nav>

      {/* Tab Panels */}
      <main className="privacy-dashboard__content">
        {activeTab === 'consents' && (
          <div role="tabpanel" id="panel-consents" aria-labelledby="tab-consents">
            <ConsentManager
              userId={userId}
              onNotification={showNotification}
              theme={theme}
            />
          </div>
        )}

        {activeTab === 'data' && (
          <div role="tabpanel" id="panel-data" aria-labelledby="tab-data">
            <DataViewer
              userId={userId}
              onNotification={showNotification}
              theme={theme}
            />
          </div>
        )}

        {activeTab === 'export' && (
          <div role="tabpanel" id="panel-export" aria-labelledby="tab-export">
            <ExportPanel
              userId={userId}
              onNotification={showNotification}
              theme={theme}
            />
          </div>
        )}

        {activeTab === 'deletion' && (
          <div role="tabpanel" id="panel-deletion" aria-labelledby="tab-deletion">
            <DeletionRequest
              userId={userId}
              onNotification={showNotification}
              theme={theme}
            />
          </div>
        )}

        {activeTab === 'audit' && (
          <div role="tabpanel" id="panel-audit" aria-labelledby="tab-audit">
            <AuditTrail
              userId={userId}
              onNotification={showNotification}
              theme={theme}
            />
          </div>
        )}

        {activeTab === 'retention' && (
          <div role="tabpanel" id="panel-retention" aria-labelledby="tab-retention">
            <RetentionSettings
              userId={userId}
              onNotification={showNotification}
              theme={theme}
            />
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="privacy-dashboard__footer">
        <p>
          Your privacy rights are protected under GDPR (EU) and equivalent data protection laws.
          <a href="/privacy-policy" target="_blank" rel="noopener noreferrer"> Learn more</a>
        </p>
      </footer>
    </div>
  );
};

export default PrivacyDashboard;
