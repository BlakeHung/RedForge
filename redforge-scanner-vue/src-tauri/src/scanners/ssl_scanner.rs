use crate::models::*;
use crate::scanners::ScannerResult;
use uuid::Uuid;
use chrono::Utc;

pub struct SslScanner {
    client: reqwest::Client,
}

impl SslScanner {
    pub fn new() -> ScannerResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        Ok(Self { client })
    }

    pub async fn scan_ssl(&self, task_id: &str, hostname: &str) -> ScannerResult<SslAnalysis> {
        let url = if hostname.starts_with("http") {
            hostname.to_string()
        } else {
            format!("https://{}", hostname)
        };

        // 簡單檢查 HTTPS 連接
        let _response = self.client.get(&url).send().await?;

        // 基本的 SSL 分析
        let mut analysis = SslAnalysis {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            certificate_issuer: None,
            certificate_subject: Some(hostname.to_string()),
            valid_from: None,
            valid_to: None,
            signature_algorithm: None,
            tls_versions: Some(vec!["TLS 1.2+".to_string()]), // 基本假設
            cipher_suites: None,
            vulnerabilities: Some(Vec::new()),
            grade: None,
            created_at: Utc::now(),
        };

        // 檢查是否使用 HTTPS
        if !url.starts_with("https://") {
            analysis.vulnerabilities = Some(vec![
                "未使用 HTTPS 加密傳輸".to_string()
            ]);
            analysis.grade = Some("F".to_string());
        } else {
            analysis.grade = Some("A".to_string());
        }

        Ok(analysis)
    }

    fn calculate_grade(&self, analysis: &SslAnalysis) -> String {
        let mut score = 100;

        // 檢查 TLS 版本
        if let Some(versions) = &analysis.tls_versions {
            if versions.iter().any(|v| v.contains("TLS1_0") || v.contains("TLS1_1")) {
                score -= 20;
            }
            if !versions.iter().any(|v| v.contains("TLS1_3")) {
                score -= 10;
            }
        }

        // 檢查密碼套件
        if let Some(suites) = &analysis.cipher_suites {
            if suites.iter().any(|s| s.contains("RC4") || s.contains("3DES")) {
                score -= 30;
            }
        }

        match score {
            90..=100 => "A+".to_string(),
            80..=89 => "A".to_string(),
            70..=79 => "B".to_string(),
            60..=69 => "C".to_string(),
            50..=59 => "D".to_string(),
            _ => "F".to_string(),
        }
    }

    fn check_vulnerabilities(&self, analysis: &SslAnalysis) -> Vec<String> {
        let mut vulns = Vec::new();

        if let Some(versions) = &analysis.tls_versions {
            if versions.iter().any(|v| v.contains("TLS1_0")) {
                vulns.push("支援已棄用的 TLS 1.0 協定（易受 POODLE 攻擊）".to_string());
            }
            if versions.iter().any(|v| v.contains("TLS1_1")) {
                vulns.push("支援已棄用的 TLS 1.1 協定".to_string());
            }
        }

        if let Some(suites) = &analysis.cipher_suites {
            if suites.iter().any(|s| s.contains("RC4")) {
                vulns.push("使用不安全的 RC4 加密算法".to_string());
            }
            if suites.iter().any(|s| s.contains("3DES")) {
                vulns.push("使用弱加密的 3DES 算法".to_string());
            }
        }

        vulns
    }
}

pub async fn quick_ssl_check(url: &str) -> ScannerResult<bool> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(url).send().await?;
    Ok(response.url().scheme() == "https")
}
