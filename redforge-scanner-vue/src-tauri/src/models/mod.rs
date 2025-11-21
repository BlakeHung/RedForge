use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTask {
    pub id: String,
    pub target_url: String,
    pub scan_type: ScanType,
    pub status: ScanStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanType {
    Full,
    Quick,
    Vulnerability,
    Port,
    Ssl,
    Headers,
}

impl std::fmt::Display for ScanType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScanType::Full => write!(f, "full"),
            ScanType::Quick => write!(f, "quick"),
            ScanType::Vulnerability => write!(f, "vulnerability"),
            ScanType::Port => write!(f, "port"),
            ScanType::Ssl => write!(f, "ssl"),
            ScanType::Headers => write!(f, "headers"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for ScanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScanStatus::Pending => write!(f, "pending"),
            ScanStatus::Running => write!(f, "running"),
            ScanStatus::Completed => write!(f, "completed"),
            ScanStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::High => write!(f, "high"),
            Severity::Medium => write!(f, "medium"),
            Severity::Low => write!(f, "low"),
            Severity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: String,
    pub task_id: String,
    pub result_type: ResultType,
    pub severity: Option<Severity>,
    pub title: String,
    pub description: Option<String>,
    pub raw_data: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultType {
    Port,
    Vulnerability,
    Ssl,
    Header,
    Technology,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub result_id: String,
    pub cve_id: Option<String>,
    pub cvss_score: Option<f64>,
    pub affected_component: Option<String>,
    pub proof_of_concept: Option<String>,
    pub remediation: Option<String>,
    pub references: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPort {
    pub id: String,
    pub task_id: String,
    pub port: u16,
    pub protocol: Protocol,
    pub service_name: Option<String>,
    pub service_version: Option<String>,
    pub banner: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslAnalysis {
    pub id: String,
    pub task_id: String,
    pub certificate_issuer: Option<String>,
    pub certificate_subject: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub signature_algorithm: Option<String>,
    pub tls_versions: Option<Vec<String>>,
    pub cipher_suites: Option<Vec<String>>,
    pub vulnerabilities: Option<Vec<String>>,
    pub grade: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeader {
    pub id: String,
    pub task_id: String,
    pub header_name: String,
    pub header_value: Option<String>,
    pub is_present: bool,
    pub is_secure: bool,
    pub recommendation: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTechnology {
    pub id: String,
    pub task_id: String,
    pub technology_name: String,
    pub technology_version: Option<String>,
    pub category: TechnologyCategory,
    pub confidence: u8,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TechnologyCategory {
    Framework,
    Cms,
    Server,
    Analytics,
    Cdn,
    Language,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub task_id: String,
    pub report_type: ReportType,
    pub file_path: Option<String>,
    pub executive_summary: Option<String>,
    pub total_vulnerabilities: i32,
    pub critical_count: i32,
    pub high_count: i32,
    pub medium_count: i32,
    pub low_count: i32,
    pub info_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportType {
    Pdf,
    Html,
    Json,
    Markdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub task_id: String,
    pub stage: String,
    pub progress: u8, // 0-100
    pub message: String,
}
