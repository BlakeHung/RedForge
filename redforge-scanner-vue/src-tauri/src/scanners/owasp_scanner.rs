/**
 * OWASP Top 10 Enhanced Scanner
 *
 * 完整實作 OWASP Top 10 (2021) 漏洞掃描
 *
 * Coverage:
 * ✅ A01:2021 – Broken Access Control
 * ✅ A02:2021 – Cryptographic Failures
 * ✅ A03:2021 – Injection
 * ✅ A04:2021 – Insecure Design
 * ✅ A05:2021 – Security Misconfiguration
 * ✅ A06:2021 – Vulnerable and Outdated Components
 * ✅ A07:2021 – Identification and Authentication Failures
 * ✅ A08:2021 – Software and Data Integrity Failures
 * ✅ A09:2021 – Security Logging and Monitoring Failures
 * ✅ A10:2021 – Server-Side Request Forgery (SSRF)
 */

use crate::models::*;
use crate::scanners::ScannerResult;
use reqwest::Client;
use uuid::Uuid;
use chrono::Utc;

pub struct OwaspScanner {
    client: Client,
}

impl OwaspScanner {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(15))
                .redirect(reqwest::redirect::Policy::none()) // 不自動跟隨重定向
                .build()
                .unwrap(),
        }
    }

    /// 執行完整的 OWASP Top 10 掃描
    pub async fn scan_all(&self, task_id: &str, url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        println!("🔍 開始 OWASP Top 10 掃描: {}", url);

        // A01: Broken Access Control
        results.extend(self.a01_broken_access_control(task_id, url).await?);

        // A02: Cryptographic Failures
        results.extend(self.a02_cryptographic_failures(task_id, url).await?);

        // A03: Injection
        results.extend(self.a03_injection(task_id, url).await?);

        // A04: Insecure Design (靜態分析)
        results.extend(self.a04_insecure_design(task_id, url).await?);

        // A05: Security Misconfiguration
        results.extend(self.a05_security_misconfiguration(task_id, url).await?);

        // A06: Vulnerable and Outdated Components
        results.extend(self.a06_vulnerable_components(task_id, url).await?);

        // A07: Identification and Authentication Failures
        results.extend(self.a07_authentication_failures(task_id, url).await?);

        // A08: Software and Data Integrity Failures
        results.extend(self.a08_integrity_failures(task_id, url).await?);

        // A09: Security Logging and Monitoring Failures
        results.extend(self.a09_logging_failures(task_id, url).await?);

        // A10: Server-Side Request Forgery
        results.extend(self.a10_ssrf(task_id, url).await?);

        println!("✅ OWASP Top 10 掃描完成，發現 {} 個潛在問題", results.len());

        Ok(results)
    }

    // ========================================================================
    // A01: Broken Access Control
    // ========================================================================
    async fn a01_broken_access_control(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        // 檢查常見的管理後台路徑
        let admin_paths = vec![
            "/admin", "/administrator", "/admin.php", "/admin/",
            "/wp-admin", "/adminpanel", "/cpanel", "/controlpanel",
            "/dashboard", "/manage", "/manager", "/backend",
        ];

        for path in admin_paths {
            let test_url = format!("{}{}", base_url.trim_end_matches('/'), path);

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    let status = response.status().as_u16();

                    // 200 OK 或 403 Forbidden 都代表路徑存在
                    if status == 200 || status == 403 {
                        let severity = if status == 200 {
                            Severity::High
                        } else {
                            Severity::Medium
                        };

                        results.push(self.create_result(
                            task_id,
                            severity,
                            format!("發現管理後台路徑: {}", path),
                            format!(
                                "管理後台可訪問 (HTTP {})，可能存在未授權訪問風險。建議: 1) 使用強認證 2) IP 白名單 3) 隱藏管理路徑",
                                status
                            ),
                            serde_json::json!({
                                "owasp": "A01:2021",
                                "path": path,
                                "status": status,
                                "url": test_url
                            })
                        ));
                    }
                },
                Err(_) => continue,
            }
        }

        // 檢查 IDOR (Insecure Direct Object Reference)
        let idor_patterns = vec![
            "?id=1", "?user_id=1", "?doc_id=1", "?file_id=1",
        ];

        for pattern in idor_patterns {
            let test_url = format!("{}{}", base_url, pattern);

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let body = response.text().await.unwrap_or_default();

                        // 檢查是否返回了用戶數據
                        if body.contains("email") || body.contains("username") || body.contains("user") {
                            results.push(self.create_result(
                                task_id,
                                Severity::High,
                                format!("潛在的 IDOR 漏洞: {}", pattern),
                                "URL 參數可能存在不安全的直接對象引用 (IDOR)，攻擊者可能通過修改 ID 訪問其他用戶資料".to_string(),
                                serde_json::json!({
                                    "owasp": "A01:2021",
                                    "type": "IDOR",
                                    "pattern": pattern,
                                    "url": test_url
                                })
                            ));
                            break;
                        }
                    }
                },
                Err(_) => continue,
            }
        }

        // 檢查 Path Traversal
        let path_traversal_payloads = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "....//....//....//etc/passwd",
        ];

        for payload in path_traversal_payloads {
            let test_url = format!("{}?file={}", base_url, urlencoding::encode(payload));

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    let body = response.text().await.unwrap_or_default();

                    if body.contains("root:") || body.contains("[boot loader]") {
                        results.push(self.create_result(
                            task_id,
                            Severity::Critical,
                            "路徑遍歷漏洞 (Path Traversal)".to_string(),
                            format!(
                                "使用 payload '{}' 成功讀取系統文件，攻擊者可能讀取任意文件",
                                payload
                            ),
                            serde_json::json!({
                                "owasp": "A01:2021",
                                "type": "Path Traversal",
                                "payload": payload,
                                "url": test_url
                            })
                        ));
                        break;
                    }
                },
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    // ========================================================================
    // A02: Cryptographic Failures
    // ========================================================================
    async fn a02_cryptographic_failures(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        // 檢查是否使用 HTTPS
        if !base_url.starts_with("https://") {
            results.push(self.create_result(
                task_id,
                Severity::High,
                "未使用 HTTPS 加密傳輸".to_string(),
                "網站未使用 HTTPS，所有傳輸資料（包括密碼、個人資訊）都可能被中間人攔截。建議: 啟用 HTTPS 並強制重定向".to_string(),
                serde_json::json!({
                    "owasp": "A02:2021",
                    "protocol": "http",
                    "url": base_url
                })
            ));
        }

        // 檢查 HTTP 是否會自動重定向到 HTTPS
        if base_url.starts_with("https://") {
            let http_url = base_url.replace("https://", "http://");

            match self.client.get(&http_url).send().await {
                Ok(response) => {
                    let location = response.headers().get("location")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("");

                    if !location.starts_with("https://") {
                        results.push(self.create_result(
                            task_id,
                            Severity::Medium,
                            "HTTP 未自動重定向到 HTTPS".to_string(),
                            "HTTP 請求未自動重定向到 HTTPS，用戶可能在不安全的連接下訪問網站".to_string(),
                            serde_json::json!({
                                "owasp": "A02:2021",
                                "http_url": http_url,
                                "redirect": location
                            })
                        ));
                    }
                },
                Err(_) => {},
            }
        }

        // 檢查原始碼中的敏感資訊洩露
        match self.client.get(base_url).send().await {
            Ok(response) => {
                let body = response.text().await.unwrap_or_default();

                // 檢查各種密鑰和令牌
                let sensitive_patterns = vec![
                    (r#"(?i)api[_-]?key['\"]?\s*[:=]\s*['\"]([a-zA-Z0-9_\-]{20,})"#, "API Key"),
                    (r#"(?i)secret[_-]?key['\"]?\s*[:=]\s*['\"]([a-zA-Z0-9_\-]{20,})"#, "Secret Key"),
                    (r#"(?i)access[_-]?token['\"]?\s*[:=]\s*['\"]([a-zA-Z0-9_\-]{20,})"#, "Access Token"),
                    (r#"(?i)password['\"]?\s*[:=]\s*['\"]([^'\"]{3,})"#, "Password"),
                    (r#"(?i)aws[_-]?access[_-]?key['\"]?\s*[:=]\s*['\"]([A-Z0-9]{20})"#, "AWS Access Key"),
                    (r#"(?i)private[_-]?key['\"]?\s*[:=]"#, "Private Key"),
                    (r#"-----BEGIN (RSA |DSA )?PRIVATE KEY-----"#, "PEM Private Key"),
                ];

                for (pattern, name) in sensitive_patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if re.is_match(&body) {
                            results.push(self.create_result(
                                task_id,
                                Severity::Critical,
                                format!("HTML 原始碼中發現 {}", name),
                                format!(
                                    "網頁原始碼中包含 {}，這可能導致嚴重的資訊洩露。建議: 1) 移除硬編碼密鑰 2) 使用環境變數 3) 使用密鑰管理服務",
                                    name
                                ),
                                serde_json::json!({
                                    "owasp": "A02:2021",
                                    "type": name,
                                    "pattern": pattern
                                })
                            ));
                        }
                    }
                }

                // 檢查是否有明文密碼輸入框
                if body.contains(r#"type="password""#) && !body.contains("autocomplete=\"off\"") {
                    results.push(self.create_result(
                        task_id,
                        Severity::Low,
                        "密碼輸入框未禁用自動完成".to_string(),
                        "密碼輸入框未設置 autocomplete=\"off\"，可能導致密碼被瀏覽器緩存".to_string(),
                        serde_json::json!({
                            "owasp": "A02:2021",
                            "type": "Autocomplete"
                        })
                    ));
                }
            },
            Err(_) => {},
        }

        Ok(results)
    }

    // ========================================================================
    // A03: Injection
    // ========================================================================
    async fn a03_injection(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        // SQL Injection
        results.extend(self.check_sql_injection(task_id, base_url).await?);

        // XSS (Cross-Site Scripting)
        results.extend(self.check_xss(task_id, base_url).await?);

        // Command Injection
        results.extend(self.check_command_injection(task_id, base_url).await?);

        // LDAP Injection
        results.extend(self.check_ldap_injection(task_id, base_url).await?);

        Ok(results)
    }

    async fn check_sql_injection(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        let sql_payloads = vec![
            ("' OR '1'='1", "Basic OR injection"),
            ("' OR '1'='1' --", "OR injection with comment"),
            ("1' OR '1' = '1", "Numeric OR injection"),
            ("admin'--", "Admin bypass"),
            ("' UNION SELECT NULL--", "UNION injection"),
            ("' AND 1=0 UNION ALL SELECT 'admin', '81dc9bdb52d04dc20036dbd8313ed055'", "UNION hash injection"),
            ("1' AND SLEEP(5)--", "Time-based blind injection"),
        ];

        for (payload, description) in sql_payloads {
            let test_url = format!("{}?id={}", base_url, urlencoding::encode(payload));

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    let body = response.text().await.unwrap_or_default().to_lowercase();

                    // SQL 錯誤訊息特徵
                    let sql_errors = vec![
                        "sql syntax", "mysql", "postgresql", "sqlite", "syntax error",
                        "odbc", "jdbc", "oracle", "warning: mysql", "unclosed quotation",
                        "quoted string not properly terminated", "sqlexception",
                    ];

                    if sql_errors.iter().any(|err| body.contains(err)) {
                        results.push(self.create_result(
                            task_id,
                            Severity::Critical,
                            format!("SQL Injection 漏洞: {}", description),
                            format!(
                                "使用 payload '{}' 觸發了資料庫錯誤訊息，確認存在 SQL 注入漏洞。建議: 1) 使用參數化查詢 2) 使用 ORM 3) 輸入驗證",
                                payload
                            ),
                            serde_json::json!({
                                "owasp": "A03:2021",
                                "type": "SQL Injection",
                                "payload": payload,
                                "description": description,
                                "url": test_url
                            })
                        ));
                        break;
                    }
                },
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    async fn check_xss(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        let xss_payloads = vec![
            ("<script>alert('XSS')</script>", "Basic XSS"),
            ("<img src=x onerror=alert('XSS')>", "Image XSS"),
            ("javascript:alert('XSS')", "JavaScript protocol"),
            ("<svg onload=alert('XSS')>", "SVG XSS"),
            ("<iframe src=javascript:alert('XSS')>", "Iframe XSS"),
            ("'><script>alert(String.fromCharCode(88,83,83))</script>", "Encoded XSS"),
        ];

        for (payload, description) in xss_payloads {
            let test_url = format!("{}?q={}", base_url, urlencoding::encode(payload));

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    let body = response.text().await.unwrap_or_default();

                    // 檢查 payload 是否未經編碼直接出現在響應中
                    if body.contains(payload) || body.contains(&payload.replace("'", "\"")) {
                        results.push(self.create_result(
                            task_id,
                            Severity::High,
                            format!("XSS (跨站腳本) 漏洞: {}", description),
                            format!(
                                "輸入內容未正確編碼就輸出到 HTML 中，可能存在 XSS 漏洞。建議: 1) 輸出編碼 2) Content Security Policy 3) HttpOnly Cookie"
                            ),
                            serde_json::json!({
                                "owasp": "A03:2021",
                                "type": "XSS",
                                "payload": payload,
                                "description": description,
                                "url": test_url
                            })
                        ));
                        break;
                    }
                },
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    async fn check_command_injection(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        let command_payloads = vec![
            (";ls", "Semicolon command separator"),
            ("| ls", "Pipe operator"),
            ("$(ls)", "Command substitution"),
            ("`ls`", "Backtick execution"),
            ("&& ls", "AND operator"),
            ("|| ls", "OR operator"),
        ];

        for (payload, description) in command_payloads {
            let test_url = format!("{}?cmd={}", base_url, urlencoding::encode(payload));

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    let body = response.text().await.unwrap_or_default();

                    // 檢查命令執行的特徵
                    if body.contains("bin") || body.contains("usr") || body.contains("etc") {
                        results.push(self.create_result(
                            task_id,
                            Severity::Critical,
                            format!("命令注入漏洞: {}", description),
                            format!(
                                "使用 payload '{}' 可能觸發了命令執行，存在 OS 命令注入漏洞。建議: 1) 避免調用系統命令 2) 使用白名單驗證 3) 使用安全的 API",
                                payload
                            ),
                            serde_json::json!({
                                "owasp": "A03:2021",
                                "type": "Command Injection",
                                "payload": payload,
                                "description": description,
                                "url": test_url
                            })
                        ));
                        break;
                    }
                },
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    async fn check_ldap_injection(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        let ldap_payloads = vec![
            ("*", "Wildcard"),
            ("admin*)(uid=*", "LDAP filter injection"),
            ("*)(uid=*))(|(uid=*", "Complex LDAP injection"),
        ];

        for (payload, description) in ldap_payloads {
            let test_url = format!("{}?user={}", base_url, urlencoding::encode(payload));

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();

                    // 檢查 LDAP 錯誤或異常行為
                    if body.to_lowercase().contains("ldap") || status.as_u16() == 500 {
                        results.push(self.create_result(
                            task_id,
                            Severity::High,
                            format!("潛在的 LDAP 注入: {}", description),
                            "應用程序可能存在 LDAP 注入漏洞，攻擊者可能繞過身份驗證或提取敏感資訊".to_string(),
                            serde_json::json!({
                                "owasp": "A03:2021",
                                "type": "LDAP Injection",
                                "payload": payload,
                                "description": description,
                                "url": test_url
                            })
                        ));
                        break;
                    }
                },
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    // ========================================================================
    // A04: Insecure Design
    // ========================================================================
    async fn a04_insecure_design(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        match self.client.get(base_url).send().await {
            Ok(response) => {
                let body = response.text().await.unwrap_or_default();

                // 檢查是否缺少速率限制 (通過多次請求測試)
                let mut success_count = 0;
                for _ in 0..10 {
                    if let Ok(r) = self.client.get(base_url).send().await {
                        if r.status().is_success() {
                            success_count += 1;
                        }
                    }
                }

                if success_count == 10 {
                    results.push(self.create_result(
                        task_id,
                        Severity::Medium,
                        "缺少速率限制 (Rate Limiting)".to_string(),
                        "應用程序未實施速率限制，可能遭受暴力破解、DDoS 攻擊。建議: 實施請求速率限制和 IP 黑名單".to_string(),
                        serde_json::json!({
                            "owasp": "A04:2021",
                            "type": "No Rate Limiting",
                            "test_requests": 10
                        })
                    ));
                }

                // 檢查是否有明顯的用戶枚舉問題
                if body.contains("User not found") || body.contains("Invalid username") {
                    results.push(self.create_result(
                        task_id,
                        Severity::Medium,
                        "用戶枚舉漏洞".to_string(),
                        "登錄失敗時區分用戶名不存在和密碼錯誤，攻擊者可枚舉有效用戶名。建議: 使用統一的錯誤訊息".to_string(),
                        serde_json::json!({
                            "owasp": "A04:2021",
                            "type": "User Enumeration"
                        })
                    ));
                }
            },
            Err(_) => {},
        }

        Ok(results)
    }

    // ========================================================================
    // A05: Security Misconfiguration
    // ========================================================================
    async fn a05_security_misconfiguration(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        // 檢查敏感文件
        let sensitive_files = vec![
            ("/.git/config", Severity::Critical, "Git 配置文件"),
            ("/.env", Severity::Critical, "環境變數文件"),
            ("/config.php", Severity::High, "PHP 配置文件"),
            ("/wp-config.php", Severity::High, "WordPress 配置"),
            ("/.htaccess", Severity::Medium, "Apache 配置"),
            ("/phpinfo.php", Severity::High, "PHP 資訊頁面"),
            ("/web.config", Severity::High, "IIS 配置"),
            ("/backup.sql", Severity::Critical, "資料庫備份"),
            ("/database.sql", Severity::Critical, "資料庫備份"),
            ("/.DS_Store", Severity::Low, "macOS 系統文件"),
            ("/robots.txt", Severity::Info, "Robots 文件"),
            ("/sitemap.xml", Severity::Info, "網站地圖"),
        ];

        for (path, severity, description) in sensitive_files {
            let test_url = format!("{}{}", base_url.trim_end_matches('/'), path);

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        results.push(self.create_result(
                            task_id,
                            severity,
                            format!("發現可訪問的敏感文件: {}", description),
                            format!(
                                "{} ({}) 可以被公開訪問，這可能洩露重要的配置資訊或原始碼",
                                description, path
                            ),
                            serde_json::json!({
                                "owasp": "A05:2021",
                                "path": path,
                                "url": test_url,
                                "status": response.status().as_u16()
                            })
                        ));
                    }
                },
                Err(_) => continue,
            }
        }

        // 檢查目錄列表
        let directories = vec!["/uploads", "/images", "/static", "/assets", "/backup", "/tmp"];
        for dir in directories {
            let test_url = format!("{}{}", base_url.trim_end_matches('/'), dir);

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    let body = response.text().await.unwrap_or_default();

                    if body.contains("Index of") || body.contains("Directory listing") || body.contains("Parent Directory") {
                        results.push(self.create_result(
                            task_id,
                            Severity::Medium,
                            format!("發現目錄列表: {}", dir),
                            format!(
                                "目錄 {} 啟用了目錄列表功能，可能洩露文件結構和敏感文件名稱",
                                dir
                            ),
                            serde_json::json!({
                                "owasp": "A05:2021",
                                "type": "Directory Listing",
                                "path": dir,
                                "url": test_url
                            })
                        ));
                    }
                },
                Err(_) => continue,
            }
        }

        // 檢查 HTTP 安全標頭
        match self.client.get(base_url).send().await {
            Ok(response) => {
                let headers = response.headers();

                // 檢查關鍵安全標頭
                if !headers.contains_key("strict-transport-security") {
                    results.push(self.create_result(
                        task_id,
                        Severity::Medium,
                        "缺少 Strict-Transport-Security 標頭".to_string(),
                        "未設置 HSTS，瀏覽器可能使用不安全的 HTTP 連接。建議: 添加 Strict-Transport-Security 標頭".to_string(),
                        serde_json::json!({
                            "owasp": "A05:2021",
                            "header": "Strict-Transport-Security"
                        })
                    ));
                }

                if !headers.contains_key("x-frame-options") && !headers.contains_key("content-security-policy") {
                    results.push(self.create_result(
                        task_id,
                        Severity::Medium,
                        "缺少 Clickjacking 防護".to_string(),
                        "未設置 X-Frame-Options 或 CSP frame-ancestors，可能遭受 Clickjacking 攻擊".to_string(),
                        serde_json::json!({
                            "owasp": "A05:2021",
                            "header": "X-Frame-Options / CSP"
                        })
                    ));
                }

                if !headers.contains_key("content-security-policy") {
                    results.push(self.create_result(
                        task_id,
                        Severity::Low,
                        "缺少 Content-Security-Policy 標頭".to_string(),
                        "未設置 CSP，無法防禦 XSS 和資料注入攻擊。建議: 實施嚴格的 CSP 策略".to_string(),
                        serde_json::json!({
                            "owasp": "A05:2021",
                            "header": "Content-Security-Policy"
                        })
                    ));
                }
            },
            Err(_) => {},
        }

        Ok(results)
    }

    // ========================================================================
    // A06: Vulnerable and Outdated Components
    // ========================================================================
    async fn a06_vulnerable_components(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        match self.client.get(base_url).send().await {
            Ok(response) => {
                // 先提取 headers (因為 text() 會移動所有權)
                let server_header = response.headers().get("server")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                let body = response.text().await.unwrap_or_default();

                // 檢查已知的過時庫和框架
                let vulnerable_libs = vec![
                    // jQuery
                    ("jquery-1.", "jQuery 1.x", Severity::High, "已知多個 XSS 漏洞，CVE-2015-9251"),
                    ("jquery-2.", "jQuery 2.x", Severity::Medium, "已知安全問題"),

                    // AngularJS
                    ("angular.js/1.0", "AngularJS 1.0", Severity::High, "已停止支援"),
                    ("angular.js/1.2", "AngularJS 1.2", Severity::High, "已停止支援"),

                    // Bootstrap
                    ("bootstrap/3.", "Bootstrap 3", Severity::Medium, "已停止安全更新"),

                    // WordPress
                    ("wp-content/plugins/", "WordPress Plugins", Severity::Medium, "需檢查插件版本"),

                    // Other frameworks
                    ("lodash@4.17.1", "Lodash 4.17.1", Severity::High, "已知原型污染漏洞"),
                    ("moment.js/2.19.", "Moment.js 2.19.x", Severity::Low, "已停止維護，建議遷移到 Day.js"),
                ];

                for (pattern, lib_name, severity, issue) in vulnerable_libs {
                    if body.to_lowercase().contains(pattern) {
                        results.push(self.create_result(
                            task_id,
                            severity,
                            format!("使用過時的組件: {}", lib_name),
                            format!(
                                "檢測到使用 {}，{}。建議: 升級到最新版本或使用替代方案",
                                lib_name, issue
                            ),
                            serde_json::json!({
                                "owasp": "A06:2021",
                                "library": lib_name,
                                "pattern": pattern,
                                "issue": issue
                            })
                        ));
                    }
                }

                // 檢查 Server 版本
                if let Some(server_str) = server_header {
                    // 檢查是否洩露版本資訊
                    if server_str.contains('/') {
                        results.push(self.create_result(
                            task_id,
                            Severity::Low,
                            "Server 標頭洩露版本資訊".to_string(),
                            format!(
                                "Server 標頭包含版本資訊 '{}'，可能幫助攻擊者識別已知漏洞。建議: 隱藏版本資訊",
                                server_str
                            ),
                            serde_json::json!({
                                "owasp": "A06:2021",
                                "header": "Server",
                                "value": server_str
                            })
                        ));
                    }
                }
            },
            Err(_) => {},
        }

        Ok(results)
    }

    // ========================================================================
    // A07: Identification and Authentication Failures
    // ========================================================================
    async fn a07_authentication_failures(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        // 檢查登錄頁面
        let login_paths = vec!["/login", "/signin", "/auth", "/user/login"];

        for path in login_paths {
            let test_url = format!("{}{}", base_url.trim_end_matches('/'), path);

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let body = response.text().await.unwrap_or_default();

                        // 檢查是否缺少 CSRF token
                        if body.to_lowercase().contains("password") {
                            if !body.contains("csrf") && !body.contains("token") && !body.contains("_token") {
                                results.push(self.create_result(
                                    task_id,
                                    Severity::High,
                                    "登錄表單缺少 CSRF 保護".to_string(),
                                    "登錄表單未包含 CSRF token，可能遭受跨站請求偽造攻擊。建議: 實施 CSRF token 驗證".to_string(),
                                    serde_json::json!({
                                        "owasp": "A07:2021",
                                        "type": "Missing CSRF Token",
                                        "path": path,
                                        "url": test_url
                                    })
                                ));
                            }

                            // 檢查是否支持弱密碼
                            // 這裡只能做靜態檢查，無法真正測試
                            if !body.to_lowercase().contains("password") || !body.contains("minimum") {
                                results.push(self.create_result(
                                    task_id,
                                    Severity::Medium,
                                    "未顯示密碼強度要求".to_string(),
                                    "登錄/註冊頁面未顯示密碼強度要求，用戶可能設置弱密碼。建議: 實施密碼策略並顯示要求".to_string(),
                                    serde_json::json!({
                                        "owasp": "A07:2021",
                                        "type": "Weak Password Policy",
                                        "path": path
                                    })
                                ));
                            }
                        }
                    }
                },
                Err(_) => continue,
            }
        }

        // 檢查 Session Cookie 安全性
        match self.client.get(base_url).send().await {
            Ok(response) => {
                for cookie in response.cookies() {
                    let name = cookie.name().to_lowercase();

                    // 檢查是否為 session cookie
                    if name.contains("session") || name.contains("sess") || name == "phpsessid" {
                        if !cookie.secure() {
                            results.push(self.create_result(
                                task_id,
                                Severity::High,
                                format!("Session Cookie 未設置 Secure 標誌: {}", cookie.name()),
                                "Session cookie 未設置 Secure 標誌，可能在 HTTP 連接中被竊取。建議: 設置 Secure 和 HttpOnly 標誌".to_string(),
                                serde_json::json!({
                                    "owasp": "A07:2021",
                                    "cookie_name": cookie.name(),
                                    "missing_flags": vec!["Secure"]
                                })
                            ));
                        }

                        if !cookie.http_only() {
                            results.push(self.create_result(
                                task_id,
                                Severity::High,
                                format!("Session Cookie 未設置 HttpOnly 標誌: {}", cookie.name()),
                                "Session cookie 未設置 HttpOnly 標誌，可能被 JavaScript 竊取 (XSS)。建議: 設置 HttpOnly 標誌".to_string(),
                                serde_json::json!({
                                    "owasp": "A07:2021",
                                    "cookie_name": cookie.name(),
                                    "missing_flags": vec!["HttpOnly"]
                                })
                            ));
                        }
                    }
                }
            },
            Err(_) => {},
        }

        // 檢查預設憑證 (常見的用戶名密碼組合)
        let default_creds = vec![
            ("admin", "admin"),
            ("admin", "password"),
            ("root", "root"),
            ("administrator", "administrator"),
        ];

        for (username, password) in default_creds {
            // 注意: 這裡只是檢測，不實際測試
            results.push(self.create_result(
                task_id,
                Severity::Info,
                "建議測試預設憑證".to_string(),
                format!(
                    "建議手動測試常見的預設憑證組合 (如 {}/{})，確保不使用預設憑證",
                    username, password
                ),
                serde_json::json!({
                    "owasp": "A07:2021",
                    "type": "Default Credentials Check",
                    "note": "Manual testing required"
                })
            ));
            break; // 只提示一次
        }

        Ok(results)
    }

    // ========================================================================
    // A08: Software and Data Integrity Failures
    // ========================================================================
    async fn a08_integrity_failures(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        match self.client.get(base_url).send().await {
            Ok(response) => {
                // 先提取 cookies 資料為 owned values (因為 text() 會移動所有權)
                let cookie_data: Vec<(String, String)> = response.cookies()
                    .map(|c| (c.name().to_string(), c.value().to_string()))
                    .collect();

                let body = response.text().await.unwrap_or_default();

                // 檢查是否從不安全的 CDN 加載資源
                if body.contains("http://") && (body.contains(".js") || body.contains(".css")) {
                    results.push(self.create_result(
                        task_id,
                        Severity::High,
                        "從不安全的 HTTP 加載外部資源".to_string(),
                        "網頁從 HTTP 協議加載 JavaScript 或 CSS 資源，可能被中間人攻擊篡改。建議: 使用 HTTPS 和 SRI (Subresource Integrity)".to_string(),
                        serde_json::json!({
                            "owasp": "A08:2021",
                            "type": "Insecure Resource Loading"
                        })
                    ));
                }

                // 檢查是否使用 SRI (Subresource Integrity)
                let has_external_scripts = body.contains("<script src=\"http") || body.contains("<link");
                let has_sri = body.contains("integrity=\"") && body.contains("sha");

                if has_external_scripts && !has_sri {
                    results.push(self.create_result(
                        task_id,
                        Severity::Medium,
                        "外部資源未使用 SRI 驗證".to_string(),
                        "從 CDN 加載的資源未使用 Subresource Integrity，無法驗證完整性。建議: 為所有外部資源添加 integrity 屬性".to_string(),
                        serde_json::json!({
                            "owasp": "A08:2021",
                            "type": "Missing SRI"
                        })
                    ));
                }

                // 檢查 Cookies 中是否包含序列化資料
                for (cookie_name, value) in cookie_data {

                    // 檢查是否為 Java、PHP、Python 序列化資料
                    if value.len() > 50 && (
                        value.starts_with("O:") ||        // PHP serialize
                        value.starts_with("rO0") ||       // Java serialize (base64)
                        value.contains("__pickle") ||      // Python pickle
                        value.contains("__reduce")
                    ) {
                        results.push(self.create_result(
                            task_id,
                            Severity::High,
                            format!("Cookie 中發現序列化資料: {}", cookie_name),
                            "Cookie 中包含序列化物件，如果未正確驗證可能導致遠程代碼執行 (反序列化漏洞)。建議: 使用 JSON 或 JWT，並驗證簽名".to_string(),
                            serde_json::json!({
                                "owasp": "A08:2021",
                                "cookie_name": cookie_name,
                                "value_prefix": &value[..20.min(value.len())]
                            })
                        ));
                    }
                }
            },
            Err(_) => {},
        }

        Ok(results)
    }

    // ========================================================================
    // A09: Security Logging and Monitoring Failures
    // ========================================================================
    async fn a09_logging_failures(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        // 嘗試觸發錯誤並檢查是否洩露詳細資訊
        let error_paths = vec![
            "/nonexistent-page-12345",
            "/?id=99999999",
            "/<script>alert(1)</script>",
        ];

        for path in error_paths {
            let test_url = format!("{}{}", base_url.trim_end_matches('/'), path);

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    let body = response.text().await.unwrap_or_default();

                    // 檢查是否洩露堆棧追踪或敏感資訊
                    let sensitive_info = vec![
                        "stack trace", "traceback", "exception",
                        "line ", "file:", "at ",
                        "sql", "query", "database",
                    ];

                    if sensitive_info.iter().any(|info| body.to_lowercase().contains(info)) {
                        results.push(self.create_result(
                            task_id,
                            Severity::Medium,
                            "錯誤頁面洩露詳細資訊".to_string(),
                            "錯誤頁面顯示堆棧追踪或技術細節，可能幫助攻擊者了解系統架構。建議: 1) 使用自定義錯誤頁面 2) 記錄到日誌而非顯示給用戶".to_string(),
                            serde_json::json!({
                                "owasp": "A09:2021",
                                "type": "Information Disclosure",
                                "url": test_url
                            })
                        ));
                        break;
                    }
                },
                Err(_) => continue,
            }
        }

        // 檢查是否有安全日誌端點 (這只是提示)
        results.push(self.create_result(
            task_id,
            Severity::Info,
            "建議實施安全日誌和監控".to_string(),
            "確保應用程序記錄以下事件: 1) 登錄失敗 2) 訪問敏感資源 3) 輸入驗證失敗 4) 權限檢查失敗。建議: 使用 SIEM 系統集中管理日誌".to_string(),
            serde_json::json!({
                "owasp": "A09:2021",
                "type": "Logging Best Practices",
                "note": "Manual verification required"
            })
        ));

        Ok(results)
    }

    // ========================================================================
    // A10: Server-Side Request Forgery (SSRF)
    // ========================================================================
    async fn a10_ssrf(&self, task_id: &str, base_url: &str) -> ScannerResult<Vec<ScanResult>> {
        let mut results = Vec::new();

        // SSRF 測試 payload
        let ssrf_payloads = vec![
            ("http://localhost", "Localhost"),
            ("http://127.0.0.1", "Loopback IP"),
            ("http://169.254.169.254", "AWS Metadata"),
            ("http://metadata.google.internal", "GCP Metadata"),
            ("http://[::1]", "IPv6 Loopback"),
            ("file:///etc/passwd", "File Protocol"),
        ];

        // 測試常見的 SSRF 參數
        let params = vec!["url", "uri", "path", "dest", "redirect", "fetch", "file", "document"];

        for param in params {
            for (payload, description) in &ssrf_payloads {
                let test_url = format!("{}?{}={}", base_url, param, urlencoding::encode(payload));

                match self.client.get(&test_url).send().await {
                    Ok(response) => {
                        let body = response.text().await.unwrap_or_default();

                        // 檢查是否成功訪問內部資源
                        let ssrf_indicators = vec![
                            "root:", "localhost", "127.0.0.1",
                            "ami-id", "instance-id", // AWS metadata
                            "kube-env", // GCP metadata
                        ];

                        if ssrf_indicators.iter().any(|indicator| body.to_lowercase().contains(indicator)) {
                            results.push(self.create_result(
                                task_id,
                                Severity::Critical,
                                format!("SSRF (服務器端請求偽造) 漏洞: {}", description),
                                format!(
                                    "應用程序可能存在 SSRF 漏洞，攻擊者可訪問內部資源。Payload: {}。建議: 1) 驗證和白名單 URL 2) 禁用不必要的協議 3) 使用網絡隔離",
                                    payload
                                ),
                                serde_json::json!({
                                    "owasp": "A10:2021",
                                    "payload": payload,
                                    "description": description,
                                    "parameter": param,
                                    "url": test_url
                                })
                            ));
                            break;
                        }

                        // 檢查響應時間 (time-based SSRF detection)
                        // 如果響應時間明顯變長，可能正在訪問內部網絡
                    },
                    Err(_) => continue,
                }
            }
        }

        // 檢查 Open Redirect (開放重定向)
        let redirect_payloads = vec![
            "https://evil.com",
            "//evil.com",
            "/\\evil.com",
        ];

        for payload in redirect_payloads {
            let test_url = format!("{}?redirect={}", base_url, urlencoding::encode(payload));

            match self.client.get(&test_url).send().await {
                Ok(response) => {
                    if let Some(location) = response.headers().get("location") {
                        if let Ok(location_str) = location.to_str() {
                            if location_str.contains("evil.com") {
                                results.push(self.create_result(
                                    task_id,
                                    Severity::Medium,
                                    "開放重定向 (Open Redirect) 漏洞".to_string(),
                                    format!(
                                        "應用程序存在開放重定向漏洞，可能被用於釣魚攻擊。Payload: {}。建議: 驗證重定向 URL 並使用白名單",
                                        payload
                                    ),
                                    serde_json::json!({
                                        "owasp": "A10:2021",
                                        "type": "Open Redirect",
                                        "payload": payload,
                                        "redirect_to": location_str,
                                        "url": test_url
                                    })
                                ));
                                break;
                            }
                        }
                    }
                },
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================
    fn create_result(
        &self,
        task_id: &str,
        severity: Severity,
        title: String,
        description: String,
        raw_data: serde_json::Value,
    ) -> ScanResult {
        ScanResult {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            result_type: ResultType::Vulnerability,
            severity: Some(severity),
            title,
            description: Some(description),
            raw_data: Some(serde_json::to_string(&raw_data).unwrap()),
            created_at: Utc::now(),
        }
    }
}
