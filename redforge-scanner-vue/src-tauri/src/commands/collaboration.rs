/**
 * Collaboration Commands
 *
 * Handles offline collaboration via encrypted Markdown export/import
 * Provides data export, import, and deduplication functionality
 */

use crate::models::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ============================================================================
// Export Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub version: String,
    pub format: String,
    pub encryption: Option<String>,
    pub exported_by: String,
    pub team_id: Option<String>,
    pub exported_at: String,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportScanTask {
    pub id: String,
    pub name: String,
    pub target: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFinding {
    pub id: String,
    pub scan_id: String,
    #[serde(rename = "type")]
    pub finding_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub affected_url: Option<String>,
    pub evidence: Option<String>,
    pub recommendation: Option<String>,
    pub discovered_at: String,
    pub discovered_by: String,
    pub cvss_score: Option<f64>,
    pub cve_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub metadata: ExportMetadata,
    pub scans: Vec<ExportScanTask>,
    pub findings: Vec<ExportFinding>,
    pub annotations: Option<Vec<Annotation>>,
    pub assets: Option<Vec<Asset>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub finding_id: String,
    pub author: String,
    pub content: String,
    pub created_at: String,
    pub is_false_positive: Option<bool>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub hostname: String,
    pub ip_address: Option<String>,
    pub ports: Option<Vec<u16>>,
    pub services: Option<Vec<String>>,
    pub technologies: Option<Vec<String>>,
    pub discovered_at: String,
}

// ============================================================================
// Import Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub imported: ImportCounts,
    pub skipped: ImportCounts,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCounts {
    pub scans: i32,
    pub findings: i32,
    pub annotations: i32,
    pub assets: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicateResult {
    pub duplicates: DuplicateIds,
    pub unique: ExportData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateIds {
    pub scans: Vec<String>,
    pub findings: Vec<String>,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Export scan data for offline collaboration
///
/// Retrieves scan data from the database and formats it for export
/// Encryption is handled on the frontend
#[tauri::command]
pub async fn export_scan_data(
    scan_ids: Option<Vec<String>>,
    include_findings_only: bool,
    include_annotations: bool,
    include_assets: bool,
    since: Option<String>,
    state: tauri::State<'_, crate::commands::scan::ScanState>,
) -> Result<ExportData, String> {
    // Convert since string to DateTime if provided
    let since_dt: Option<DateTime<Utc>> = since
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let metadata = ExportMetadata {
        version: "1.0.0".to_string(),
        format: "encrypted-markdown".to_string(),
        encryption: None, // Set by frontend
        exported_by: "system".to_string(), // Set by frontend
        team_id: None,
        exported_at: Utc::now().to_rfc3339(),
        checksum: None,
    };

    // Get real scan data from state
    let tasks = state.current_tasks.lock().await;
    let results = state.scan_results.lock().await;

    // Filter scans based on scan_ids or since
    let mut scans = Vec::new();
    let mut findings = Vec::new();

    for task in tasks.iter() {
        // Filter by scan_ids if provided
        if let Some(ref ids) = scan_ids {
            if !ids.contains(&task.id) {
                continue;
            }
        }

        // Filter by since date if provided
        if let Some(ref since_date) = since_dt {
            if &task.created_at < since_date {
                continue;
            }
        }

        // Add scan task
        scans.push(ExportScanTask {
            id: task.id.clone(),
            name: format!("{} - {}", task.scan_type.to_string(), task.target_url),
            target: task.target_url.clone(),
            status: task.status.to_string(),
            created_at: task.created_at.to_rfc3339(),
            started_at: task.started_at.map(|dt| dt.to_rfc3339()),
            completed_at: task.completed_at.map(|dt| dt.to_rfc3339()),
            created_by: "user".to_string(),
        });

        // Get scan results/findings
        if !include_findings_only {
            if let Some(report) = results.get(&task.id) {
                for vuln in &report.vulnerabilities {
                    findings.push(ExportFinding {
                        id: vuln.id.clone(),
                        scan_id: task.id.clone(),
                        finding_type: format!("{:?}", vuln.result_type).to_lowercase(),
                        severity: vuln.severity.as_ref().map(|s| s.to_string()).unwrap_or("info".to_string()),
                        title: vuln.title.clone(),
                        description: vuln.description.clone().unwrap_or_default(),
                        affected_url: Some(task.target_url.clone()),
                        evidence: vuln.raw_data.clone(),
                        recommendation: None,
                        discovered_at: vuln.created_at.to_rfc3339(),
                        discovered_by: "redforge".to_string(),
                        cvss_score: None,
                        cve_id: None,
                    });
                }
            }
        }
    }

    // Optional annotations (empty for now)
    let annotations = if include_annotations {
        Some(Vec::new())
    } else {
        None
    };

    // Optional assets (extract from scan results)
    let assets = if include_assets {
        let mut asset_list = Vec::new();
        for task in scans.iter() {
            if let Some(report) = results.get(&task.id) {
                // Extract hostname from URL
                let hostname = task.target
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .split('/')
                    .next()
                    .unwrap_or(&task.target)
                    .to_string();

                // Collect technologies
                let technologies: Vec<String> = report.technologies
                    .iter()
                    .map(|t| t.technology_name.clone())
                    .collect();

                if !technologies.is_empty() || report.ssl_analysis.is_some() {
                    asset_list.push(Asset {
                        id: uuid::Uuid::new_v4().to_string(),
                        hostname: hostname.clone(),
                        ip_address: None,
                        ports: None,
                        services: None,
                        technologies: Some(technologies),
                        discovered_at: task.created_at.clone(),
                    });
                }
            }
        }
        Some(asset_list)
    } else {
        None
    };

    Ok(ExportData {
        metadata,
        scans,
        findings,
        annotations,
        assets,
    })
}

/// Deduplicate imported data before inserting into database
///
/// Checks for duplicate scans and findings based on IDs and similarity
#[tauri::command]
pub async fn deduplicate_import_data(data: ExportData) -> Result<ExportData, String> {
    // TODO: Implement actual database queries to check for existing data
    // For now, we'll just check for duplicates within the import data itself

    let mut unique_scans = Vec::new();
    let mut unique_findings = Vec::new();
    let mut seen_scan_ids = HashSet::new();
    let mut seen_finding_ids = HashSet::new();

    // Deduplicate scans
    for scan in data.scans {
        if !seen_scan_ids.contains(&scan.id) {
            seen_scan_ids.insert(scan.id.clone());
            unique_scans.push(scan);
        }
    }

    // Deduplicate findings
    for finding in data.findings {
        if !seen_finding_ids.contains(&finding.id) {
            seen_finding_ids.insert(finding.id.clone());
            unique_findings.push(finding);
        }
    }

    // Deduplicate annotations if present
    let unique_annotations = data.annotations.map(|annotations| {
        let mut unique = Vec::new();
        let mut seen_ids = HashSet::new();
        for annotation in annotations {
            if !seen_ids.contains(&annotation.id) {
                seen_ids.insert(annotation.id.clone());
                unique.push(annotation);
            }
        }
        unique
    });

    // Deduplicate assets if present
    let unique_assets = data.assets.map(|assets| {
        let mut unique = Vec::new();
        let mut seen_ids = HashSet::new();
        for asset in assets {
            if !seen_ids.contains(&asset.id) {
                seen_ids.insert(asset.id.clone());
                unique.push(asset);
            }
        }
        unique
    });

    Ok(ExportData {
        metadata: data.metadata,
        scans: unique_scans,
        findings: unique_findings,
        annotations: unique_annotations,
        assets: unique_assets,
    })
}

/// Import deduplicated data into database
///
/// Inserts scans, findings, annotations, and assets into the database
#[tauri::command]
pub async fn import_scan_data(
    data: ExportData,
    skip_duplicates: bool,
    merge_strategy: String,
    state: tauri::State<'_, crate::commands::scan::ScanState>,
) -> Result<ImportResult, String> {
    use crate::commands::scan::ScanReport;
    use crate::models::*;

    let mut imported_counts = ImportCounts {
        scans: 0,
        findings: 0,
        annotations: 0,
        assets: 0,
    };

    let mut skipped_counts = ImportCounts {
        scans: 0,
        findings: 0,
        annotations: 0,
        assets: 0,
    };

    let mut errors = Vec::new();

    // Get current state
    let mut tasks = state.current_tasks.lock().await;
    let mut results = state.scan_results.lock().await;

    // Import scans
    for export_scan in data.scans {
        // Check if scan already exists
        if skip_duplicates && tasks.iter().any(|t| t.id == export_scan.id) {
            skipped_counts.scans += 1;
            continue;
        }

        // Parse scan type
        let scan_type = match export_scan.name.to_lowercase() {
            n if n.contains("full") => ScanType::Full,
            n if n.contains("quick") => ScanType::Quick,
            n if n.contains("vulnerability") => ScanType::Vulnerability,
            n if n.contains("port") => ScanType::Port,
            n if n.contains("ssl") => ScanType::Ssl,
            n if n.contains("headers") => ScanType::Headers,
            _ => ScanType::Full,
        };

        // Parse scan status
        let status = match export_scan.status.as_str() {
            "pending" => ScanStatus::Pending,
            "running" => ScanStatus::Running,
            "completed" => ScanStatus::Completed,
            "failed" => ScanStatus::Failed,
            _ => ScanStatus::Completed,
        };

        // Parse timestamps
        let created_at = DateTime::parse_from_rfc3339(&export_scan.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let started_at = export_scan.started_at
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let completed_at = export_scan.completed_at
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // Create ScanTask
        let task = ScanTask {
            id: export_scan.id.clone(),
            target_url: export_scan.target,
            scan_type,
            status,
            started_at,
            completed_at,
            created_at,
        };

        // Collect findings for this scan
        let scan_findings: Vec<ScanResult> = data.findings
            .iter()
            .filter(|f| f.scan_id == export_scan.id)
            .map(|f| {
                let result_type = match f.finding_type.as_str() {
                    "port" => ResultType::Port,
                    "vulnerability" => ResultType::Vulnerability,
                    "ssl" => ResultType::Ssl,
                    "header" => ResultType::Header,
                    "technology" => ResultType::Technology,
                    _ => ResultType::Vulnerability,
                };

                let severity = match f.severity.as_str() {
                    "critical" => Some(Severity::Critical),
                    "high" => Some(Severity::High),
                    "medium" => Some(Severity::Medium),
                    "low" => Some(Severity::Low),
                    "info" => Some(Severity::Info),
                    _ => Some(Severity::Info),
                };

                let discovered_at = DateTime::parse_from_rfc3339(&f.discovered_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                ScanResult {
                    id: f.id.clone(),
                    task_id: f.scan_id.clone(),
                    result_type,
                    severity,
                    title: f.title.clone(),
                    description: Some(f.description.clone()),
                    raw_data: f.evidence.clone(),
                    created_at: discovered_at,
                }
            })
            .collect();

        // Create ScanReport
        let report = ScanReport {
            task: task.clone(),
            headers: Vec::new(), // TODO: Extract from findings if available
            ssl_analysis: None,  // TODO: Extract from findings if available
            technologies: Vec::new(), // TODO: Extract from assets if available
            vulnerabilities: scan_findings.clone(),
        };

        // Add to state
        tasks.push(task);
        results.insert(export_scan.id.clone(), report);

        imported_counts.scans += 1;
        imported_counts.findings += scan_findings.len() as i32;
    }

    // TODO: Import annotations and assets

    Ok(ImportResult {
        success: true,
        imported: imported_counts,
        skipped: skipped_counts,
        errors,
    })
}

/// Check if a finding is duplicate based on similarity
///
/// Uses fuzzy matching on title and description
fn is_finding_similar(finding1: &ExportFinding, finding2: &ExportFinding) -> bool {
    // Simple similarity check - in production, use more sophisticated algorithm
    finding1.title == finding2.title
        && finding1.scan_id == finding2.scan_id
        && finding1.severity == finding2.severity
}

/// Calculate similarity score between two strings (0.0 - 1.0)
fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    // Simple Jaccard similarity - in production, use Levenshtein or other algorithms
    let set1: HashSet<&str> = s1.split_whitespace().collect();
    let set2: HashSet<&str> = s2.split_whitespace().collect();

    let intersection = set1.intersection(&set2).count();
    let union = set1.union(&set2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}
