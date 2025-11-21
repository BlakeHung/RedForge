-- RedForge Scanner Database Schema v1.0
-- Created: 2025-11-21

-- =============================================================================
-- Scan Tasks Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS scan_tasks (
    id TEXT PRIMARY KEY,
    target_url TEXT NOT NULL,
    scan_type TEXT NOT NULL CHECK(scan_type IN ('full', 'quick', 'vulnerability', 'port', 'ssl', 'headers')),
    status TEXT NOT NULL CHECK(status IN ('pending', 'running', 'completed', 'failed')),
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT DEFAULT 'user'
);

CREATE INDEX idx_scan_tasks_status ON scan_tasks(status);
CREATE INDEX idx_scan_tasks_created_at ON scan_tasks(created_at DESC);
CREATE INDEX idx_scan_tasks_target_url ON scan_tasks(target_url);

-- =============================================================================
-- Scan Results Table (Vulnerabilities, Findings)
-- =============================================================================
CREATE TABLE IF NOT EXISTS scan_results (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    result_type TEXT NOT NULL CHECK(result_type IN ('port', 'vulnerability', 'ssl', 'header', 'technology')),
    severity TEXT CHECK(severity IN ('critical', 'high', 'medium', 'low', 'info')),
    title TEXT NOT NULL,
    description TEXT,
    raw_data TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_scan_results_task_id ON scan_results(task_id);
CREATE INDEX idx_scan_results_severity ON scan_results(severity);
CREATE INDEX idx_scan_results_type ON scan_results(result_type);

-- =============================================================================
-- Security Headers Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS security_headers (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    header_name TEXT NOT NULL,
    header_value TEXT,
    is_present INTEGER NOT NULL DEFAULT 0,
    is_secure INTEGER NOT NULL DEFAULT 0,
    recommendation TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_security_headers_task_id ON security_headers(task_id);

-- =============================================================================
-- SSL/TLS Analysis Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS ssl_analysis (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    certificate_issuer TEXT,
    certificate_subject TEXT,
    valid_from TEXT,
    valid_to TEXT,
    signature_algorithm TEXT,
    tls_versions TEXT, -- JSON array
    cipher_suites TEXT, -- JSON array
    vulnerabilities TEXT, -- JSON array
    grade TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_ssl_analysis_task_id ON ssl_analysis(task_id);

-- =============================================================================
-- Detected Technologies Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS detected_technologies (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    technology_name TEXT NOT NULL,
    technology_version TEXT,
    category TEXT CHECK(category IN ('framework', 'cms', 'server', 'analytics', 'cdn', 'language', 'database')),
    confidence INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_detected_technologies_task_id ON detected_technologies(task_id);

-- =============================================================================
-- Annotations Table (for collaboration)
-- =============================================================================
CREATE TABLE IF NOT EXISTS annotations (
    id TEXT PRIMARY KEY,
    finding_id TEXT NOT NULL,
    author TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    is_false_positive INTEGER DEFAULT 0,
    priority TEXT CHECK(priority IN ('high', 'medium', 'low')),
    FOREIGN KEY (finding_id) REFERENCES scan_results(id) ON DELETE CASCADE
);

CREATE INDEX idx_annotations_finding_id ON annotations(finding_id);
CREATE INDEX idx_annotations_author ON annotations(author);

-- =============================================================================
-- Assets Table (discovered hosts, IPs, services)
-- =============================================================================
CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    hostname TEXT NOT NULL,
    ip_address TEXT,
    ports TEXT, -- JSON array of port numbers
    services TEXT, -- JSON array of service names
    technologies TEXT, -- JSON array of technologies
    discovered_at TEXT NOT NULL,
    first_scan_id TEXT,
    FOREIGN KEY (first_scan_id) REFERENCES scan_tasks(id)
);

CREATE INDEX idx_assets_hostname ON assets(hostname);
CREATE INDEX idx_assets_ip ON assets(ip_address);

-- =============================================================================
-- Export/Import Logs Table (for tracking collaboration)
-- =============================================================================
CREATE TABLE IF NOT EXISTS export_import_logs (
    id TEXT PRIMARY KEY,
    operation TEXT NOT NULL CHECK(operation IN ('export', 'import')),
    file_path TEXT,
    exported_by TEXT,
    team_id TEXT,
    scan_count INTEGER DEFAULT 0,
    finding_count INTEGER DEFAULT 0,
    timestamp TEXT NOT NULL,
    metadata TEXT -- JSON metadata
);

CREATE INDEX idx_export_import_logs_operation ON export_import_logs(operation);
CREATE INDEX idx_export_import_logs_timestamp ON export_import_logs(timestamp DESC);
