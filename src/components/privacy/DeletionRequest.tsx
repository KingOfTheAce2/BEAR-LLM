/**
 * Deletion Request Component
 * GDPR Article 17 - Right to erasure ("right to be forgotten")
 */

import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { DeletionRequest as DeletionRequestType, NotificationState } from './types';

interface DeletionRequestProps {
  userId: string;
  theme?: 'light' | 'dark';
  onNotification: (message: string, type: NotificationState['type']) => void;
}

const DeletionRequest: React.FC<DeletionRequestProps> = ({ userId, theme, onNotification }) => {
  const [step, setStep] = useState<'initial' | 'confirm' | 'processing' | 'scheduled'>('initial');
  const [reason, setReason] = useState<string>('');
  const [confirmText, setConfirmText] = useState<string>('');
  const [understood, setUnderstood] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);
  const [deletionRequest, setDeletionRequest] = useState<DeletionRequestType | null>(null);

  const CONFIRMATION_TEXT = 'DELETE MY DATA';

  const handleInitiateDeletion = () => {
    if (!reason.trim()) {
      onNotification('Please provide a reason for deletion', 'warning');
      return;
    }
    setStep('confirm');
  };

  const handleConfirmDeletion = async () => {
    if (confirmText !== CONFIRMATION_TEXT) {
      onNotification(`Please type "${CONFIRMATION_TEXT}" to confirm`, 'warning');
      return;
    }

    if (!understood) {
      onNotification('Please acknowledge that you understand the consequences', 'warning');
      return;
    }

    setLoading(true);
    setStep('processing');

    try {
      // Submit deletion request
      await invoke('delete_user_data', { userId });

      const request: DeletionRequestType = {
        userId,
        reason: reason.trim(),
        requestedAt: new Date().toISOString(),
        scheduledFor: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString(), // 30 days
        status: 'scheduled'
      };

      setDeletionRequest(request);
      setStep('scheduled');
      onNotification('Deletion request submitted successfully', 'success');
    } catch (error) {
      console.error('Failed to submit deletion request:', error);
      onNotification('Failed to submit deletion request. Please try again.', 'error');
      setStep('confirm');
    } finally {
      setLoading(false);
    }
  };

  const handleCancel = () => {
    setStep('initial');
    setConfirmText('');
    setUnderstood(false);
  };

  if (step === 'scheduled' && deletionRequest) {
    return (
      <div className="deletion-request">
        <div className="deletion-request__success">
          <div className="success-icon" aria-hidden="true">✓</div>
          <h2>Deletion Request Scheduled</h2>
          <p className="deletion-request__message">
            Your data deletion request has been submitted and will be processed on{' '}
            <strong>{new Date(deletionRequest.scheduledFor!).toLocaleDateString()}</strong>.
          </p>

          <div className="info-box info-box--warning">
            <strong>Grace Period:</strong>
            <p>
              Your account will be deactivated immediately, but data will be retained for 30 days
              in case you change your mind. You can cancel this request before the deletion date.
            </p>
          </div>

          <div className="deletion-request__details">
            <h3>What happens next?</h3>
            <ol>
              <li>Your account is immediately deactivated</li>
              <li>You'll receive a confirmation email</li>
              <li>After 30 days, all your data will be permanently deleted</li>
              <li>You can cancel within the 30-day grace period</li>
            </ol>
          </div>

          <div className="deletion-request__actions">
            <button className="btn btn--secondary">
              Cancel Deletion Request
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (step === 'processing') {
    return (
      <div className="deletion-request">
        <div className="deletion-request__loading">
          <div className="spinner" role="status">
            <div className="spinner__circle"></div>
          </div>
          <h2>Processing Deletion Request...</h2>
          <p>Please wait while we process your request.</p>
        </div>
      </div>
    );
  }

  if (step === 'confirm') {
    return (
      <div className="deletion-request">
        <header className="deletion-request__header">
          <h2>⚠️ Confirm Data Deletion</h2>
        </header>

        <div className="deletion-request__content">
          <div className="info-box info-box--danger">
            <strong>Warning: This action cannot be undone after the grace period!</strong>
            <p>
              Deleting your data will permanently remove all information associated with your account.
              This includes personal information, activity history, preferences, and all generated content.
            </p>
          </div>

          <div className="deletion-request__consequences">
            <h3>What will be deleted:</h3>
            <ul>
              <li>✓ Personal information (name, email, address, etc.)</li>
              <li>✓ Account credentials and authentication data</li>
              <li>✓ Activity history and usage logs</li>
              <li>✓ Preferences and settings</li>
              <li>✓ All user-generated content</li>
              <li>✓ Consent records and privacy settings</li>
            </ul>

            <h3>What will be retained:</h3>
            <ul>
              <li>✓ Anonymized analytics (no personal identifiers)</li>
              <li>✓ Legal compliance records (as required by law)</li>
              <li>✓ Billing records (for tax purposes, up to 7 years)</li>
            </ul>
          </div>

          <div className="form-group">
            <label className="form-label">
              Reason for deletion:
            </label>
            <textarea
              value={reason}
              disabled
              className="form-textarea"
              rows={3}
              readOnly
            />
          </div>

          <div className="form-group">
            <label className="form-label">
              Type <strong>"{CONFIRMATION_TEXT}"</strong> to confirm:
            </label>
            <input
              type="text"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value)}
              className="form-input"
              placeholder={CONFIRMATION_TEXT}
              autoComplete="off"
              aria-describedby="confirm-help"
            />
            <small id="confirm-help" className="form-help">
              This confirmation is required to prevent accidental deletions
            </small>
          </div>

          <div className="form-group">
            <label className="checkbox-item checkbox-item--compact">
              <input
                type="checkbox"
                checked={understood}
                onChange={(e) => setUnderstood(e.target.checked)}
              />
              <span>
                I understand that this action is permanent after the 30-day grace period
                and cannot be reversed
              </span>
            </label>
          </div>

          <div className="deletion-request__actions">
            <button
              onClick={handleCancel}
              className="btn btn--secondary"
              disabled={loading}
            >
              Cancel
            </button>
            <button
              onClick={handleConfirmDeletion}
              disabled={loading || confirmText !== CONFIRMATION_TEXT || !understood}
              className="btn btn--danger"
              aria-busy={loading}
            >
              {loading ? 'Processing...' : 'Confirm Deletion'}
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Initial step
  return (
    <div className="deletion-request">
      <header className="deletion-request__header">
        <h2>Delete My Data</h2>
        <p className="deletion-request__description">
          Request permanent deletion of all your personal data in compliance with GDPR Article 17
          (Right to Erasure).
        </p>
      </header>

      <div className="deletion-request__content">
        <div className="info-box info-box--warning">
          <strong>Before you proceed:</strong>
          <p>
            Consider exporting your data first if you want to keep a copy.
            Once deleted, your data cannot be recovered after the 30-day grace period.
          </p>
        </div>

        <div className="form-group">
          <label htmlFor="deletion-reason" className="form-label">
            Please tell us why you want to delete your data (required):
          </label>
          <textarea
            id="deletion-reason"
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            className="form-textarea"
            rows={4}
            placeholder="E.g., I no longer use this service, privacy concerns, etc."
            aria-required="true"
          />
          <small className="form-help">
            This helps us improve our service. Your feedback is valuable.
          </small>
        </div>

        <div className="deletion-request__process">
          <h3>Deletion Process:</h3>
          <ol className="process-steps">
            <li className="process-step">
              <strong>Submit Request</strong>
              <p>Provide a reason and confirm your decision</p>
            </li>
            <li className="process-step">
              <strong>Immediate Deactivation</strong>
              <p>Your account is deactivated immediately</p>
            </li>
            <li className="process-step">
              <strong>30-Day Grace Period</strong>
              <p>You can cancel the deletion during this time</p>
            </li>
            <li className="process-step">
              <strong>Permanent Deletion</strong>
              <p>After 30 days, all data is permanently erased</p>
            </li>
          </ol>
        </div>

        <div className="deletion-request__actions">
          <button
            onClick={handleInitiateDeletion}
            disabled={!reason.trim()}
            className="btn btn--danger"
          >
            Proceed to Confirmation
          </button>
        </div>
      </div>

      <footer className="deletion-request__footer">
        <div className="info-box info-box--info">
          <strong>Your Rights:</strong> Under GDPR, you have the right to erasure of your personal data.
          However, we may retain certain data as required by law (e.g., billing records for tax purposes).
        </div>
      </footer>
    </div>
  );
};

export default DeletionRequest;
