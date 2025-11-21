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
) -> Result<ExportData, String> {
    // Convert since string to DateTime if provided
    let since_dt: Option<DateTime<Utc>> = since
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    // Mock data for now - in production, fetch from database
    // TODO: Implement actual database queries

    let metadata = ExportMetadata {
        version: "1.0.0".to_string(),
        format: "encrypted-markdown".to_string(),
        encryption: None, // Set by frontend
        exported_by: "system".to_string(), // Set by frontend
        team_id: None,
        exported_at: Utc::now().to_rfc3339(),
        checksum: None,
    };

    // Mock scans
    let mut scans = Vec::new();
    if let Some(ids) = scan_ids {
        for id in ids {
            scans.push(ExportScanTask {
                id: id.clone(),
                name: format!("Scan {}", id),
                target: "https://example.com".to_string(),
                status: "completed".to_string(),
                created_at: Utc::now().to_rfc3339(),
                started_at: Some(Utc::now().to_rfc3339()),
                completed_at: Some(Utc::now().to_rfc3339()),
                created_by: "user".to_string(),
            });
        }
    }

    // Mock findings
    let mut findings = Vec::new();
    if !scans.is_empty() && !include_findings_only {
        findings.push(ExportFinding {
            id: uuid::Uuid::new_v4().to_string(),
            scan_id: scans[0].id.clone(),
            finding_type: "vulnerability".to_string(),
            severity: "high".to_string(),
            title: "SQL Injection".to_string(),
            description: "Potential SQL injection vulnerability detected".to_string(),
            affected_url: Some("https://example.com/login".to_string()),
            evidence: Some("payload: ' OR 1=1--".to_string()),
            recommendation: Some("Use parameterized queries".to_string()),
            discovered_at: Utc::now().to_rfc3339(),
            discovered_by: "redforge".to_string(),
            cvss_score: Some(8.5),
            cve_id: None,
        });
    }

    // Optional annotations
    let annotations = if include_annotations {
        Some(Vec::new())
    } else {
        None
    };

    // Optional assets
    let assets = if include_assets {
        Some(Vec::new())
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
) -> Result<ImportResult, String> {
    // TODO: Implement actual database insertions
    // For now, return mock result

    let imported = ImportCounts {
        scans: data.scans.len() as i32,
        findings: data.findings.len() as i32,
        annotations: data.annotations.as_ref().map(|a| a.len() as i32).unwrap_or(0),
        assets: data.assets.as_ref().map(|a| a.len() as i32).unwrap_or(0),
    };

    let skipped = ImportCounts {
        scans: 0,
        findings: 0,
        annotations: 0,
        assets: 0,
    };

    Ok(ImportResult {
        success: true,
        imported,
        skipped,
        errors: Vec::new(),
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
