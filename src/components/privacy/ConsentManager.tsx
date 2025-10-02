/**
 * Consent Manager Component
 * GDPR Article 7 - Granular consent management with version tracking
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { ConsentStatus, ConsentRecord, ConsentType, NotificationState } from './types';

interface ConsentManagerProps {
  userId: string;
  theme?: 'light' | 'dark';
  onNotification: (message: string, type: NotificationState['type']) => void;
}

const CONSENT_TYPES: ConsentType[] = [
  {
    id: 'essential',
    name: 'Essential Cookies & Data',
    description: 'Required for basic functionality, security, and legal compliance. Cannot be disabled.',
    required: true,
    version: '1.0.0',
    category: 'essential'
  },
  {
    id: 'analytics',
    name: 'Analytics & Performance',
    description: 'Help us understand how you use the application to improve user experience.',
    required: false,
    version: '1.0.0',
    category: 'analytics'
  },
  {
    id: 'marketing',
    name: 'Marketing & Communications',
    description: 'Receive personalized recommendations and promotional content.',
    required: false,
    version: '1.0.0',
    category: 'marketing'
  },
  {
    id: 'personalization',
    name: 'Personalization',
    description: 'Customize your experience based on your preferences and behavior.',
    required: false,
    version: '1.0.0',
    category: 'personalization'
  }
];

const ConsentManager: React.FC<ConsentManagerProps> = ({ userId, theme, onNotification }) => {
  const [consentStatus, setConsentStatus] = useState<ConsentStatus | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [updating, setUpdating] = useState<Set<string>>(new Set());

  useEffect(() => {
    loadConsentStatus();
  }, [userId]);

  const loadConsentStatus = async () => {
    try {
      setLoading(true);
      const status = await invoke<ConsentStatus>('check_consent_status', { userId });
      setConsentStatus(status);
      setLoading(false);
    } catch (error) {
      console.error('Failed to load consent status:', error);
      onNotification('Failed to load consent preferences', 'error');
      setLoading(false);
    }
  };

  const handleConsentToggle = async (consentType: string, currentlyGranted: boolean) => {
    // Don't allow toggling essential consents
    const type = CONSENT_TYPES.find(t => t.id === consentType);
    if (type?.required) {
      onNotification('Essential consents cannot be disabled', 'warning');
      return;
    }

    setUpdating(prev => new Set(prev).add(consentType));

    try {
      // Get user context for audit trail
      const ipAddress = await invoke<string>('get_client_ip').catch(() => 'unknown');
      const userAgent = navigator.userAgent;

      if (currentlyGranted) {
        // Revoke consent
        await invoke('revoke_consent', {
          userId,
          consentType,
          reason: 'User choice'
        });
        onNotification(`${type?.name} consent revoked`, 'success');
      } else {
        // Grant consent
        await invoke('grant_consent', {
          userId,
          consentType,
          ipAddress,
          userAgent
        });
        onNotification(`${type?.name} consent granted`, 'success');
      }

      // Reload consent status
      await loadConsentStatus();
    } catch (error) {
      console.error('Failed to update consent:', error);
      onNotification('Failed to update consent preference', 'error');
    } finally {
      setUpdating(prev => {
        const next = new Set(prev);
        next.delete(consentType);
        return next;
      });
    }
  };

  const isConsentGranted = (consentType: string): boolean => {
    const consent = consentStatus?.consents.find(c => c.consentType === consentType);
    return consent?.granted ?? false;
  };

  const getConsentDate = (consentType: string): string | null => {
    const consent = consentStatus?.consents.find(c => c.consentType === consentType);
    return consent?.timestamp ?? null;
  };

  if (loading) {
    return (
      <div className="consent-manager__loading">
        <div className="spinner" role="status">
          <div className="spinner__circle"></div>
        </div>
        <p>Loading consent preferences...</p>
      </div>
    );
  }

  return (
    <div className="consent-manager">
      <header className="consent-manager__header">
        <h2>Consent Management</h2>
        <p className="consent-manager__description">
          Control how your data is used. You can change these settings at any time.
          All changes are tracked for compliance purposes.
        </p>
      </header>

      <div className="consent-list">
        {CONSENT_TYPES.map(consentType => {
          const granted = isConsentGranted(consentType.id);
          const date = getConsentDate(consentType.id);
          const isUpdating = updating.has(consentType.id);

          return (
            <div
              key={consentType.id}
              className={`consent-item ${granted ? 'consent-item--granted' : ''} ${consentType.required ? 'consent-item--required' : ''}`}
            >
              <div className="consent-item__header">
                <div className="consent-item__info">
                  <h3 className="consent-item__name">
                    {consentType.name}
                    {consentType.required && (
                      <span className="badge badge--required" aria-label="Required">Required</span>
                    )}
                  </h3>
                  <p className="consent-item__description">{consentType.description}</p>
                  {date && (
                    <p className="consent-item__meta">
                      {granted ? 'Granted' : 'Revoked'} on {new Date(date).toLocaleDateString()} at {new Date(date).toLocaleTimeString()}
                      <span className="consent-item__version"> • Version {consentType.version}</span>
                    </p>
                  )}
                </div>

                <div className="consent-item__toggle">
                  <label className="toggle-switch">
                    <input
                      type="checkbox"
                      checked={granted}
                      disabled={consentType.required || isUpdating}
                      onChange={() => handleConsentToggle(consentType.id, granted)}
                      aria-label={`Toggle ${consentType.name}`}
                      aria-describedby={`consent-${consentType.id}-desc`}
                    />
                    <span className="toggle-switch__slider"></span>
                  </label>
                  {isUpdating && (
                    <span className="consent-item__updating" aria-live="polite">
                      Updating...
                    </span>
                  )}
                </div>
              </div>

              <div id={`consent-${consentType.id}-desc`} className="sr-only">
                {consentType.description}
              </div>
            </div>
          );
        })}
      </div>

      <div className="consent-manager__footer">
        <div className="info-box info-box--info">
          <strong>Your Rights:</strong> You can withdraw consent at any time.
          Changes take effect immediately and are logged for compliance purposes.
          Essential consents cannot be disabled as they are required for the service to function.
        </div>

        {consentStatus && (
          <p className="consent-manager__last-updated">
            Last updated: {new Date(consentStatus.lastUpdated).toLocaleString()}
          </p>
        )}
      </div>
    </div>
  );
};

export default ConsentManager;
