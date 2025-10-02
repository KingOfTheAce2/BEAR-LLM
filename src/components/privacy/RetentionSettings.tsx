/**
 * Retention Settings Component
 * GDPR Article 5(1)(e) - Storage limitation
 * Allow users to configure data retention preferences
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { RetentionSettings as RetentionSettingsType, RetentionPolicy, NotificationState } from './types';

interface RetentionSettingsProps {
  userId: string;
  theme?: 'light' | 'dark';
  onNotification: (message: string, type: NotificationState['type']) => void;
}

const RetentionSettings: React.FC<RetentionSettingsProps> = ({ userId, theme, onNotification }) => {
  const [settings, setSettings] = useState<RetentionSettingsType | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [saving, setSaving] = useState<boolean>(false);
  const [customSettings, setCustomSettings] = useState<Record<string, number>>({});
  const [hasChanges, setHasChanges] = useState<boolean>(false);

  useEffect(() => {
    loadRetentionSettings();
  }, [userId]);

  const loadRetentionSettings = async () => {
    try {
      setLoading(true);
      // In a real implementation, this would call a backend API
      // For now, we'll use mock data
      const mockSettings: RetentionSettingsType = {
        userId,
        policies: [
          {
            dataType: 'Activity Logs',
            retentionPeriodDays: 365,
            description: 'Login history, search queries, and user interactions',
            userConfigurable: true,
            legalBasis: 'Legitimate interest for security and service improvement'
          },
          {
            dataType: 'Personal Information',
            retentionPeriodDays: -1, // Indefinite
            description: 'Name, email, profile data - retained while account is active',
            userConfigurable: false,
            legalBasis: 'Contract necessity - required for service delivery'
          },
          {
            dataType: 'Consent Records',
            retentionPeriodDays: 2555, // ~7 years
            description: 'History of consent grants and revocations',
            userConfigurable: false,
            legalBasis: 'Legal obligation - compliance evidence'
          },
          {
            dataType: 'Analytics Data',
            retentionPeriodDays: 730,
            description: 'Anonymized usage statistics and performance metrics',
            userConfigurable: true,
            legalBasis: 'Legitimate interest for service improvement'
          },
          {
            dataType: 'Communication History',
            retentionPeriodDays: 180,
            description: 'Email correspondence and support tickets',
            userConfigurable: true,
            legalBasis: 'Legitimate interest for customer support'
          },
          {
            dataType: 'Billing Records',
            retentionPeriodDays: 2555, // ~7 years
            description: 'Payment history and invoices',
            userConfigurable: false,
            legalBasis: 'Legal obligation - tax and accounting requirements'
          }
        ],
        customSettings: {},
        lastUpdated: new Date().toISOString()
      };

      setSettings(mockSettings);
      setCustomSettings(mockSettings.customSettings || {});
      setLoading(false);
    } catch (error) {
      console.error('Failed to load retention settings:', error);
      onNotification('Failed to load retention settings', 'error');
      setLoading(false);
    }
  };

  const handleRetentionChange = (dataType: string, days: number) => {
    setCustomSettings(prev => ({
      ...prev,
      [dataType]: days
    }));
    setHasChanges(true);
  };

  const handleSaveSettings = async () => {
    setSaving(true);
    try {
      // In a real implementation, this would save to backend
      await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call

      setSettings(prev => prev ? {
        ...prev,
        customSettings,
        lastUpdated: new Date().toISOString()
      } : null);

      setHasChanges(false);
      onNotification('Retention settings saved successfully', 'success');
    } catch (error) {
      console.error('Failed to save retention settings:', error);
      onNotification('Failed to save retention settings', 'error');
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    setCustomSettings(settings?.customSettings || {});
    setHasChanges(false);
    onNotification('Changes discarded', 'info');
  };

  const formatRetentionPeriod = (days: number): string => {
    if (days === -1) return 'While account is active';
    if (days < 30) return `${days} days`;
    if (days < 365) {
      const months = Math.floor(days / 30);
      return `${months} month${months !== 1 ? 's' : ''}`;
    }
    const years = Math.floor(days / 365);
    return `${years} year${years !== 1 ? 's' : ''}`;
  };

  const getRetentionOptions = (currentDays: number): number[] => {
    const baseOptions = [30, 90, 180, 365, 730, 1095]; // 1m, 3m, 6m, 1y, 2y, 3y
    if (!baseOptions.includes(currentDays)) {
      return [...baseOptions, currentDays].sort((a, b) => a - b);
    }
    return baseOptions;
  };

  if (loading) {
    return (
      <div className="retention-settings__loading">
        <div className="spinner" role="status">
          <div className="spinner__circle"></div>
        </div>
        <p>Loading retention settings...</p>
      </div>
    );
  }

  if (!settings) {
    return (
      <div className="retention-settings__error">
        <p>Failed to load retention settings</p>
        <button onClick={loadRetentionSettings} className="btn btn--primary">
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="retention-settings">
      <header className="retention-settings__header">
        <h2>Data Retention Settings</h2>
        <p className="retention-settings__description">
          Configure how long different types of data are retained.
          This complies with GDPR Article 5(1)(e) - Storage Limitation principle.
        </p>
      </header>

      <div className="retention-policies">
        {settings.policies.map((policy) => {
          const currentRetention = customSettings[policy.dataType] ?? policy.retentionPeriodDays;
          const hasCustomSetting = policy.dataType in customSettings;

          return (
            <div key={policy.dataType} className="retention-policy">
              <div className="retention-policy__header">
                <h3 className="retention-policy__name">{policy.dataType}</h3>
                {!policy.userConfigurable && (
                  <span className="badge badge--info">Non-configurable</span>
                )}
              </div>

              <p className="retention-policy__description">{policy.description}</p>

              <div className="retention-policy__details">
                <div className="retention-detail">
                  <strong>Legal Basis:</strong>
                  <p>{policy.legalBasis}</p>
                </div>

                <div className="retention-detail">
                  <strong>Current Retention Period:</strong>
                  {policy.userConfigurable ? (
                    <div className="retention-control">
                      <select
                        value={currentRetention}
                        onChange={(e) => handleRetentionChange(policy.dataType, Number(e.target.value))}
                        className="retention-select"
                        aria-label={`Retention period for ${policy.dataType}`}
                      >
                        {getRetentionOptions(policy.retentionPeriodDays).map(days => (
                          <option key={days} value={days}>
                            {formatRetentionPeriod(days)}
                          </option>
                        ))}
                      </select>
                      {hasCustomSetting && currentRetention !== policy.retentionPeriodDays && (
                        <span className="retention-modified" aria-label="Modified from default">
                          ✎ Modified
                        </span>
                      )}
                    </div>
                  ) : (
                    <p className="retention-fixed">{formatRetentionPeriod(policy.retentionPeriodDays)}</p>
                  )}
                </div>
              </div>

              {policy.userConfigurable && (
                <div className="retention-policy__info">
                  <small>
                    Default: {formatRetentionPeriod(policy.retentionPeriodDays)}
                    {currentRetention < policy.retentionPeriodDays &&
                      ' • Shorter retention enhances privacy'}
                    {currentRetention > policy.retentionPeriodDays &&
                      ' • Longer retention may be useful for historical analysis'}
                  </small>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Action Buttons */}
      {hasChanges && (
        <div className="retention-settings__actions">
          <button
            onClick={handleReset}
            disabled={saving}
            className="btn btn--secondary"
          >
            Discard Changes
          </button>
          <button
            onClick={handleSaveSettings}
            disabled={saving}
            className="btn btn--primary"
            aria-busy={saving}
          >
            {saving ? (
              <>
                <span className="spinner spinner--small"></span>
                Saving...
              </>
            ) : (
              'Save Changes'
            )}
          </button>
        </div>
      )}

      {/* Information */}
      <footer className="retention-settings__footer">
        <div className="info-box info-box--info">
          <strong>Data Retention Information:</strong>
          <ul>
            <li>
              <strong>Storage Limitation:</strong> Data should not be kept longer than necessary
              for the purposes for which it is processed
            </li>
            <li>
              <strong>Configurable Periods:</strong> Where allowed, you can adjust retention periods
              to balance functionality with privacy
            </li>
            <li>
              <strong>Legal Requirements:</strong> Some data must be retained for specific periods
              to comply with legal obligations (e.g., tax records)
            </li>
            <li>
              <strong>Automatic Deletion:</strong> Data is automatically deleted after the retention
              period expires unless there's a legal reason to keep it
            </li>
          </ul>
        </div>

        <p className="retention-settings__last-updated">
          Last updated: {new Date(settings.lastUpdated).toLocaleString()}
        </p>
      </footer>
    </div>
  );
};

export default RetentionSettings;
