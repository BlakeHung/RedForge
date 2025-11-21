-- 掃描任務表
CREATE TABLE IF NOT EXISTS scan_tasks (
    id TEXT PRIMARY KEY,
    target_url TEXT NOT NULL,
    scan_type TEXT NOT NULL, -- 'full', 'quick', 'vulnerability', 'port'
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'running', 'completed', 'failed'
    started_at DATETIME,
    completed_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 掃描結果表
CREATE TABLE IF NOT EXISTS scan_results (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    result_type TEXT NOT NULL, -- 'port', 'vulnerability', 'ssl', 'header', 'technology'
    severity TEXT, -- 'critical', 'high', 'medium', 'low', 'info'
    title TEXT NOT NULL,
    description TEXT,
    raw_data TEXT, -- JSON 格式的原始資料
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

-- 漏洞詳情表
CREATE TABLE IF NOT EXISTS vulnerabilities (
    id TEXT PRIMARY KEY,
    result_id TEXT NOT NULL,
    cve_id TEXT,
    cvss_score REAL,
    affected_component TEXT,
    proof_of_concept TEXT,
    remediation TEXT,
    references TEXT, -- JSON array
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (result_id) REFERENCES scan_results(id) ON DELETE CASCADE
);

-- 開放端口表
CREATE TABLE IF NOT EXISTS open_ports (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    port INTEGER NOT NULL,
    protocol TEXT NOT NULL, -- 'tcp', 'udp'
    service_name TEXT,
    service_version TEXT,
    banner TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

-- SSL/TLS 分析表
CREATE TABLE IF NOT EXISTS ssl_analysis (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    certificate_issuer TEXT,
    certificate_subject TEXT,
    valid_from DATETIME,
    valid_to DATETIME,
    signature_algorithm TEXT,
    tls_versions TEXT, -- JSON array
    cipher_suites TEXT, -- JSON array
    vulnerabilities TEXT, -- JSON array
    grade TEXT, -- 'A+', 'A', 'B', 'C', 'D', 'F'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

-- HTTP 標頭安全分析表
CREATE TABLE IF NOT EXISTS security_headers (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    header_name TEXT NOT NULL,
    header_value TEXT,
    is_present BOOLEAN NOT NULL,
    is_secure BOOLEAN NOT NULL,
    recommendation TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

-- 技術堆疊檢測表
CREATE TABLE IF NOT EXISTS detected_technologies (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    technology_name TEXT NOT NULL,
    technology_version TEXT,
    category TEXT, -- 'framework', 'cms', 'server', 'analytics', 'cdn'
    confidence INTEGER, -- 0-100
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

-- 報告表
CREATE TABLE IF NOT EXISTS reports (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    report_type TEXT NOT NULL, -- 'pdf', 'html', 'json', 'markdown'
    file_path TEXT,
    executive_summary TEXT,
    total_vulnerabilities INTEGER DEFAULT 0,
    critical_count INTEGER DEFAULT 0,
    high_count INTEGER DEFAULT 0,
    medium_count INTEGER DEFAULT 0,
    low_count INTEGER DEFAULT 0,
    info_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES scan_tasks(id) ON DELETE CASCADE
);

-- 審計日誌表
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    action TEXT NOT NULL,
    target TEXT,
    details TEXT,
    user_agent TEXT,
    ip_address TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 創建索引以提升查詢效能
CREATE INDEX IF NOT EXISTS idx_scan_tasks_status ON scan_tasks(status);
CREATE INDEX IF NOT EXISTS idx_scan_tasks_created_at ON scan_tasks(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_scan_results_task_id ON scan_results(task_id);
CREATE INDEX IF NOT EXISTS idx_scan_results_severity ON scan_results(severity);
CREATE INDEX IF NOT EXISTS idx_vulnerabilities_cve_id ON vulnerabilities(cve_id);
CREATE INDEX IF NOT EXISTS idx_open_ports_task_id ON open_ports(task_id);
