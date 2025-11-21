use crate::models::*;
use crate::scanners::{
    http_scanner::HttpScanner,
    ssl_scanner::SslScanner,
    tech_detector::TechDetector,
    vulnerability_scanner::VulnerabilityScanner,
};
use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub task: ScanTask,
    pub headers: Vec<SecurityHeader>,
    pub ssl_analysis: Option<SslAnalysis>,
    pub technologies: Vec<DetectedTechnology>,
    pub vulnerabilities: Vec<ScanResult>,
}

pub struct ScanState {
    pub current_tasks: Arc<Mutex<Vec<ScanTask>>>,
    pub scan_results: Arc<Mutex<HashMap<String, ScanReport>>>,
}

#[tauri::command]
pub async fn start_scan(
    url: String,
    scan_type: String,
    state: State<'_, ScanState>,
) -> Result<String, String> {
    // 驗證 URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("無效的 URL 格式".to_string());
    }

    let task_id = Uuid::new_v4().to_string();

    let task = ScanTask {
        id: task_id.clone(),
        target_url: url.clone(),
        scan_type: match scan_type.as_str() {
            "full" => ScanType::Full,
            "quick" => ScanType::Quick,
            "vulnerability" => ScanType::Vulnerability,
            "port" => ScanType::Port,
            "ssl" => ScanType::Ssl,
            "headers" => ScanType::Headers,
            _ => return Err("未知的掃描類型".to_string()),
        },
        status: ScanStatus::Pending,
        started_at: None,
        completed_at: None,
        created_at: Utc::now(),
    };

    // 添加到任務列表
    let mut tasks = state.current_tasks.lock().await;
    tasks.push(task.clone());
    drop(tasks);

    // 在背景執行掃描
    let current_tasks = state.current_tasks.clone();
    let scan_results = state.scan_results.clone();
    let state_clone = ScanState { current_tasks, scan_results };
    let state_arc = Arc::new(state_clone);
    let task_id_clone = task_id.clone();
    tokio::spawn(async move {
        execute_scan(task_id_clone, url, scan_type, state_arc).await;
    });

    Ok(task_id)
}

async fn execute_scan(task_id: String, url: String, scan_type: String, state: Arc<ScanState>) {
    // 更新狀態為 Running
    update_task_status(&state, &task_id, ScanStatus::Running).await;

    // 初始化報告
    let mut report = ScanReport {
        task: ScanTask {
            id: task_id.clone(),
            target_url: url.clone(),
            scan_type: match scan_type.as_str() {
                "full" => ScanType::Full,
                "quick" => ScanType::Quick,
                "vulnerability" => ScanType::Vulnerability,
                "headers" => ScanType::Headers,
                "ssl" => ScanType::Ssl,
                _ => ScanType::Full,
            },
            status: ScanStatus::Running,
            started_at: Some(Utc::now()),
            completed_at: None,
            created_at: Utc::now(),
        },
        headers: Vec::new(),
        ssl_analysis: None,
        technologies: Vec::new(),
        vulnerabilities: Vec::new(),
    };

    let result = match scan_type.as_str() {
        "headers" => scan_headers_with_results(&task_id, &url, &mut report).await,
        "ssl" => scan_ssl_with_results(&task_id, &url, &mut report).await,
        "vulnerability" => scan_vulnerabilities_with_results(&task_id, &url, &mut report).await,
        "full" => scan_full_with_results(&task_id, &url, &mut report).await,
        _ => Err("未實現的掃描類型".to_string()),
    };

    // 更新狀態
    let status = if result.is_ok() {
        ScanStatus::Completed
    } else {
        ScanStatus::Failed
    };

    report.task.status = status.clone();
    report.task.completed_at = Some(Utc::now());

    // 存儲報告
    let mut results = state.scan_results.lock().await;
    results.insert(task_id.clone(), report);
    drop(results);

    update_task_status(&state, &task_id, status).await;
}

async fn update_task_status(state: &Arc<ScanState>, task_id: &str, status: ScanStatus) {
    let mut tasks = state.current_tasks.lock().await;
    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
        task.status = status;
        if task.started_at.is_none() {
            task.started_at = Some(Utc::now());
        }
        if matches!(task.status, ScanStatus::Completed | ScanStatus::Failed) {
            task.completed_at = Some(Utc::now());
        }
    }
}

async fn scan_headers_with_results(task_id: &str, url: &str, report: &mut ScanReport) -> Result<(), String> {
    let scanner = HttpScanner::new();
    report.headers = scanner.scan_headers(task_id, url)
        .await
        .map_err(|e| e.to_string())?;

    println!("掃描到 {} 個 HTTP 標頭", report.headers.len());
    Ok(())
}

async fn scan_ssl_with_results(task_id: &str, url: &str, report: &mut ScanReport) -> Result<(), String> {
    let hostname = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .ok_or("無效的 URL")?;

    let scanner = SslScanner::new()
        .map_err(|e| e.to_string())?;

    let analysis = scanner.scan_ssl(task_id, hostname)
        .await
        .map_err(|e| e.to_string())?;

    println!("SSL 分析完成，等級: {:?}", analysis.grade);
    report.ssl_analysis = Some(analysis);
    Ok(())
}

async fn scan_vulnerabilities_with_results(task_id: &str, url: &str, report: &mut ScanReport) -> Result<(), String> {
    let scanner = VulnerabilityScanner::new();
    report.vulnerabilities = scanner.scan(task_id, url)
        .await
        .map_err(|e| e.to_string())?;

    println!("發現 {} 個潛在漏洞", report.vulnerabilities.len());
    Ok(())
}

async fn scan_full_with_results(task_id: &str, url: &str, report: &mut ScanReport) -> Result<(), String> {
    // HTTP 標頭掃描
    scan_headers_with_results(task_id, url, report).await?;

    // SSL/TLS 分析
    if url.starts_with("https://") {
        scan_ssl_with_results(task_id, url, report).await?;
    }

    // 漏洞掃描
    scan_vulnerabilities_with_results(task_id, url, report).await?;

    // 技術檢測
    let detector = TechDetector::new();
    report.technologies = detector.detect(task_id, url)
        .await
        .map_err(|e| e.to_string())?;

    println!("檢測到 {} 個技術", report.technologies.len());

    Ok(())
}

#[tauri::command]
pub async fn get_scan_status(
    task_id: String,
    state: State<'_, ScanState>,
) -> Result<ScanTask, String> {
    let tasks = state.current_tasks.lock().await;
    tasks
        .iter()
        .find(|t| t.id == task_id)
        .cloned()
        .ok_or_else(|| "找不到該任務".to_string())
}

#[tauri::command]
pub async fn list_scans(
    state: State<'_, ScanState>,
) -> Result<Vec<ScanTask>, String> {
    let tasks = state.current_tasks.lock().await;
    Ok(tasks.clone())
}

#[tauri::command]
pub async fn get_scan_report(
    task_id: String,
    state: State<'_, ScanState>,
) -> Result<ScanReport, String> {
    let results = state.scan_results.lock().await;
    results
        .get(&task_id)
        .cloned()
        .ok_or_else(|| "找不到掃描報告".to_string())
}
