/**
 * Privacy Dashboard Module Exports
 */

export { default as PrivacyDashboard } from './PrivacyDashboard';
export { default as ConsentManager } from './ConsentManager';
export { default as DataViewer } from './DataViewer';
export { default as ExportPanel } from './ExportPanel';
export { default as DeletionRequest } from './DeletionRequest';
export { default as AuditTrail } from './AuditTrail';
export { default as RetentionSettings } from './RetentionSettings';

export type {
  ConsentType,
  ConsentRecord,
  ConsentStatus,
  UserData,
  LoginRecord,
  SearchRecord,
  ExportRequest,
  ExportFormat,
  ExportResult,
  AuditLogEntry,
  DeletionRequest,
  RetentionPolicy,
  RetentionSettings,
  PrivacyDashboardProps,
  NotificationState
} from './types';
