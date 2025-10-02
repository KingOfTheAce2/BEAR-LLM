/**
 * TypeScript interfaces for Privacy Dashboard components
 * GDPR-compliant data structures
 */

export interface ConsentType {
  id: string;
  name: string;
  description: string;
  required: boolean;
  version: string;
  category: 'essential' | 'analytics' | 'marketing' | 'personalization';
}

export interface ConsentRecord {
  consentType: string;
  granted: boolean;
  timestamp: string;
  version: string;
  ipAddress?: string;
  userAgent?: string;
  expiresAt?: string;
}

export interface ConsentStatus {
  userId: string;
  consents: ConsentRecord[];
  lastUpdated: string;
}

export interface UserData {
  userId: string;
  personalInfo: {
    email?: string;
    name?: string;
    phoneNumber?: string;
    address?: string;
    dateOfBirth?: string;
  };
  activityData: {
    loginHistory: LoginRecord[];
    searchHistory: SearchRecord[];
    preferences: Record<string, unknown>;
  };
  metadata: {
    createdAt: string;
    lastModified: string;
    dataVersion: string;
  };
}

export interface LoginRecord {
  timestamp: string;
  ipAddress: string;
  userAgent: string;
  location?: string;
  success: boolean;
}

export interface SearchRecord {
  timestamp: string;
  query: string;
  resultsCount: number;
}

export interface ExportRequest {
  userId: string;
  formats: ExportFormat[];
  includeComplianceData: boolean;
  includeAuditTrail?: boolean;
}

export type ExportFormat = 'json' | 'csv' | 'xml' | 'pdf';

export interface ExportResult {
  requestId: string;
  userId: string;
  format: ExportFormat;
  downloadUrl?: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  createdAt: string;
  expiresAt: string;
  fileSize?: number;
  error?: string;
}

export interface AuditLogEntry {
  id: string;
  userId: string;
  action: string;
  category: 'consent' | 'data_access' | 'data_export' | 'data_deletion' | 'settings_change';
  timestamp: string;
  ipAddress: string;
  userAgent: string;
  details: Record<string, unknown>;
  complianceRelated: boolean;
}

export interface DeletionRequest {
  userId: string;
  reason?: string;
  requestedAt: string;
  scheduledFor?: string;
  status: 'pending' | 'scheduled' | 'processing' | 'completed' | 'cancelled';
}

export interface RetentionPolicy {
  dataType: string;
  retentionPeriodDays: number;
  description: string;
  userConfigurable: boolean;
  legalBasis: string;
}

export interface RetentionSettings {
  userId: string;
  policies: RetentionPolicy[];
  customSettings?: Record<string, number>;
  lastUpdated: string;
}

export interface PrivacyDashboardProps {
  userId: string;
  theme?: 'light' | 'dark';
  onError?: (error: Error) => void;
}

export interface NotificationState {
  show: boolean;
  message: string;
  type: 'success' | 'error' | 'info' | 'warning';
}
