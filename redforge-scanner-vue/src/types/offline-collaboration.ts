/**
 * Offline Collaboration Type Definitions
 *
 * Defines types for encrypted Markdown export/import functionality
 * Based on OFFLINE_COLLABORATION.md specification
 */

// ============================================================================
// Export Data Structures
// ============================================================================

export interface ExportMetadata {
  version: string;
  format: 'encrypted-markdown';
  encryption?: 'AES-256-GCM';
  exported_by: string;
  team_id?: string;
  exported_at: string;
  checksum?: string;
}

export interface ScanTask {
  id: string;
  name: string;
  target: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  created_at: string;
  started_at?: string;
  completed_at?: string;
  created_by: string;
}

export interface SecurityHeader {
  id: string;
  scan_id: string;
  header_name: string;
  header_value?: string;
  is_present: boolean;
  is_secure: boolean;
  severity: 'info' | 'low' | 'medium' | 'high' | 'critical';
  recommendation?: string;
}

export interface Finding {
  id: string;
  scan_id: string;
  type: string;
  severity: 'info' | 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string;
  affected_url?: string;
  evidence?: string;
  recommendation?: string;
  discovered_at: string;
  discovered_by: string;
  cvss_score?: number;
  cve_id?: string;
}

export interface Annotation {
  id: string;
  finding_id: string;
  author: string;
  content: string;
  created_at: string;
  is_false_positive?: boolean;
  priority?: 'low' | 'medium' | 'high';
}

export interface Asset {
  id: string;
  hostname: string;
  ip_address?: string;
  ports?: number[];
  services?: string[];
  technologies?: string[];
  discovered_at: string;
}

export interface ExportData {
  metadata: ExportMetadata;
  scans: ScanTask[];
  findings: Finding[];
  annotations?: Annotation[];
  assets?: Asset[];
}

// ============================================================================
// Encryption Structures
// ============================================================================

export interface EncryptedData {
  ciphertext: string;
  iv: string;
  salt: string;
}

export interface EncryptionOptions {
  passphrase: string;
  iterations?: number; // Default: 100000
}

// ============================================================================
// Export/Import Options
// ============================================================================

export interface ExportOptions {
  scanIds?: string[];
  includeFindingsOnly?: boolean;
  includeAnnotations?: boolean;
  includeAssets?: boolean;
  since?: Date;
  encrypt?: boolean;
  passphrase?: string;
  teamId?: string;
  exportedBy: string;
}

export interface ImportOptions {
  passphrase?: string;
  skipDuplicates?: boolean;
  mergeStrategy?: 'skip' | 'overwrite' | 'merge';
}

export interface ImportResult {
  success: boolean;
  imported: {
    scans: number;
    findings: number;
    annotations: number;
    assets: number;
  };
  skipped: {
    scans: number;
    findings: number;
    annotations: number;
    assets: number;
  };
  errors: string[];
}

// ============================================================================
// Markdown Format
// ============================================================================

export interface MarkdownExport {
  frontMatter: ExportMetadata;
  encryptedBlock?: EncryptedData;
  summary: ExportSummary;
  rawContent: string;
}

export interface ExportSummary {
  totalScans: number;
  totalFindings: number;
  findingsBySeverity: {
    critical: number;
    high: number;
    medium: number;
    low: number;
    info: number;
  };
  totalAnnotations?: number;
  totalAssets?: number;
  dateRange?: {
    from: string;
    to: string;
  };
}

// ============================================================================
// Deduplication
// ============================================================================

export interface DeduplicationResult {
  duplicates: {
    scans: string[];
    findings: string[];
  };
  unique: ExportData;
}

export interface DuplicateCheckResult {
  isDuplicate: boolean;
  existingId?: string;
  similarity?: number;
}
