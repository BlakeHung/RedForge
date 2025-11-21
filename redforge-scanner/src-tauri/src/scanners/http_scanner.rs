use crate::models::*;
use crate::scanners::ScannerResult;
use reqwest::header::HeaderMap;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

pub struct HttpScanner {
    client: reqwest::Client,
}

impl HttpScanner {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true) // 為了測試目的
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    pub async fn scan_headers(&self, task_id: &str, url: &str) -> ScannerResult<Vec<SecurityHeader>> {
        let response = self.client.get(url).send().await?;
        let headers = response.headers();

        let mut results = Vec::new();

        // 定義應該檢查的安全標頭
        let security_headers = self.get_security_headers_checklist();

        for (header_name, (expected, recommendation)) in security_headers {
            let header_value = headers.get(&header_name).map(|v| v.to_str().unwrap_or("").to_string());
            let is_present = header_value.is_some();

            results.push(SecurityHeader {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                header_name: header_name.clone(),
                header_value: header_value.clone(),
                is_present,
                is_secure: is_present && self.validate_header(&header_name, &header_value),
                recommendation: Some(recommendation),
                created_at: Utc::now(),
            });
        }

        // 檢查額外的不安全標頭
        results.extend(self.check_unsafe_headers(task_id, headers));

        Ok(results)
    }

    fn get_security_headers_checklist(&self) -> HashMap<String, (bool, String)> {
        let mut headers = HashMap::new();

        headers.insert(
            "strict-transport-security".to_string(),
            (true, "啟用 HSTS 以強制使用 HTTPS 連線，建議值: max-age=31536000; includeSubDomains".to_string())
        );

        headers.insert(
            "content-security-policy".to_string(),
            (true, "設置 CSP 以防止 XSS 和資料注入攻擊".to_string())
        );

        headers.insert(
            "x-frame-options".to_string(),
            (true, "防止點擊劫持攻擊，建議值: DENY 或 SAMEORIGIN".to_string())
        );

        headers.insert(
            "x-content-type-options".to_string(),
            (true, "防止 MIME 類型嗅探，建議值: nosniff".to_string())
        );

        headers.insert(
            "referrer-policy".to_string(),
            (true, "控制 Referer 標頭資訊洩露，建議值: no-referrer 或 strict-origin-when-cross-origin".to_string())
        );

        headers.insert(
            "permissions-policy".to_string(),
            (true, "控制瀏覽器功能權限（原 Feature-Policy）".to_string())
        );

        headers.insert(
            "x-xss-protection".to_string(),
            (true, "啟用瀏覽器 XSS 過濾器，建議值: 1; mode=block".to_string())
        );

        headers
    }

    fn validate_header(&self, header_name: &str, header_value: &Option<String>) -> bool {
        if let Some(value) = header_value {
            match header_name {
                "strict-transport-security" => value.contains("max-age=") && value.len() > 20,
                "content-security-policy" => value.len() > 10,
                "x-frame-options" => value.to_uppercase().contains("DENY") || value.to_uppercase().contains("SAMEORIGIN"),
                "x-content-type-options" => value.to_lowercase().contains("nosniff"),
                "referrer-policy" => !value.is_empty(),
                "permissions-policy" => !value.is_empty(),
                "x-xss-protection" => value.contains("1"),
                _ => true,
            }
        } else {
            false
        }
    }

    fn check_unsafe_headers(&self, task_id: &str, headers: &HeaderMap) -> Vec<SecurityHeader> {
        let mut results = Vec::new();

        // 檢查是否洩露服務器版本資訊
        if let Some(server) = headers.get("server") {
            let server_value = server.to_str().unwrap_or("");
            results.push(SecurityHeader {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                header_name: "server".to_string(),
                header_value: Some(server_value.to_string()),
                is_present: true,
                is_secure: false,
                recommendation: Some("建議隱藏或移除服務器版本資訊以減少攻擊面".to_string()),
                created_at: Utc::now(),
            });
        }

        // 檢查是否洩露 X-Powered-By
        if let Some(powered_by) = headers.get("x-powered-by") {
            let value = powered_by.to_str().unwrap_or("");
            results.push(SecurityHeader {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                header_name: "x-powered-by".to_string(),
                header_value: Some(value.to_string()),
                is_present: true,
                is_secure: false,
                recommendation: Some("建議移除 X-Powered-By 標頭以避免洩露技術堆疊資訊".to_string()),
                created_at: Utc::now(),
            });
        }

        results
    }

    pub async fn detect_technologies(&self, task_id: &str, url: &str) -> ScannerResult<Vec<DetectedTechnology>> {
        let response = self.client.get(url).send().await?;
        let headers = response.headers().clone();
        let body = response.text().await?;

        let mut technologies = Vec::new();

        // 從標頭檢測
        if let Some(server) = headers.get("server") {
            let server_str = server.to_str().unwrap_or("");
            if server_str.to_lowercase().contains("nginx") {
                technologies.push(DetectedTechnology {
                    id: Uuid::new_v4().to_string(),
                    task_id: task_id.to_string(),
                    technology_name: "Nginx".to_string(),
                    technology_version: self.extract_version(server_str, "nginx"),
                    category: TechnologyCategory::Server,
                    confidence: 95,
                    created_at: Utc::now(),
                });
            }
            if server_str.to_lowercase().contains("apache") {
                technologies.push(DetectedTechnology {
                    id: Uuid::new_v4().to_string(),
                    task_id: task_id.to_string(),
                    technology_name: "Apache".to_string(),
                    technology_version: self.extract_version(server_str, "apache"),
                    category: TechnologyCategory::Server,
                    confidence: 95,
                    created_at: Utc::now(),
                });
            }
        }

        if let Some(powered_by) = headers.get("x-powered-by") {
            let powered_str = powered_by.to_str().unwrap_or("");
            if powered_str.to_lowercase().contains("php") {
                technologies.push(DetectedTechnology {
                    id: Uuid::new_v4().to_string(),
                    task_id: task_id.to_string(),
                    technology_name: "PHP".to_string(),
                    technology_version: self.extract_version(powered_str, "php"),
                    category: TechnologyCategory::Language,
                    confidence: 95,
                    created_at: Utc::now(),
                });
            }
        }

        // 從 HTML 內容檢測
        technologies.extend(self.detect_from_html(task_id, &body));

        Ok(technologies)
    }

    fn detect_from_html(&self, task_id: &str, html: &str) -> Vec<DetectedTechnology> {
        let mut technologies = Vec::new();
        let html_lower = html.to_lowercase();

        // 檢測 Next.js
        if html_lower.contains("__next") || html_lower.contains("next/script") {
            technologies.push(DetectedTechnology {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                technology_name: "Next.js".to_string(),
                technology_version: None,
                category: TechnologyCategory::Framework,
                confidence: 85,
                created_at: Utc::now(),
            });
        }

        // 檢測 React
        if html_lower.contains("react") || html_lower.contains("_reactroot") {
            technologies.push(DetectedTechnology {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                technology_name: "React".to_string(),
                technology_version: None,
                category: TechnologyCategory::Framework,
                confidence: 80,
                created_at: Utc::now(),
            });
        }

        // 檢測 Google Analytics
        if html_lower.contains("google-analytics.com") || html_lower.contains("gtag") {
            technologies.push(DetectedTechnology {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                technology_name: "Google Analytics".to_string(),
                technology_version: None,
                category: TechnologyCategory::Analytics,
                confidence: 95,
                created_at: Utc::now(),
            });
        }

        // 檢測 Tailwind CSS
        if html_lower.contains("tailwind") || self.has_tailwind_classes(html) {
            technologies.push(DetectedTechnology {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                technology_name: "Tailwind CSS".to_string(),
                technology_version: None,
                category: TechnologyCategory::Framework,
                confidence: 75,
                created_at: Utc::now(),
            });
        }

        technologies
    }

    fn has_tailwind_classes(&self, html: &str) -> bool {
        let tailwind_patterns = ["flex-", "grid-", "bg-", "text-", "p-", "m-", "w-", "h-"];
        tailwind_patterns.iter().any(|pattern| html.contains(pattern))
    }

    fn extract_version(&self, text: &str, tech: &str) -> Option<String> {
        let re = regex::Regex::new(&format!(r"{}/(\d+\.[\d.]+)", tech)).ok()?;
        re.captures(text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }
}
