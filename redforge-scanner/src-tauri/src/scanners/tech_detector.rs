use crate::models::*;
use crate::scanners::ScannerResult;
use reqwest::Client;
use uuid::Uuid;
use chrono::Utc;

pub struct TechDetector {
    client: Client,
}

impl TechDetector {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    pub async fn detect(&self, task_id: &str, url: &str) -> ScannerResult<Vec<DetectedTechnology>> {
        let response = self.client.get(url).send().await?;
        let headers = response.headers().clone();
        let body = response.text().await?;

        let mut technologies = Vec::new();

        // JavaScript 框架檢測
        technologies.extend(self.detect_js_frameworks(task_id, &body));

        // CSS 框架檢測
        technologies.extend(self.detect_css_frameworks(task_id, &body));

        // 分析工具檢測
        technologies.extend(self.detect_analytics(task_id, &body));

        // CDN 檢測
        technologies.extend(self.detect_cdn(task_id, &body, &headers));

        Ok(technologies)
    }

    fn detect_js_frameworks(&self, task_id: &str, html: &str) -> Vec<DetectedTechnology> {
        let mut techs = Vec::new();
        let html_lower = html.to_lowercase();

        let frameworks = vec![
            ("React", vec!["_reactroot", "react-", "__react"], TechnologyCategory::Framework, 85),
            ("Vue.js", vec!["data-v-", "__vue__", "vue.js"], TechnologyCategory::Framework, 85),
            ("Angular", vec!["ng-version", "angular", "_nghost"], TechnologyCategory::Framework, 85),
            ("Next.js", vec!["__next", "_next/static"], TechnologyCategory::Framework, 90),
            ("Nuxt.js", vec!["__nuxt", "_nuxt"], TechnologyCategory::Framework, 90),
            ("Svelte", vec!["svelte-", "__svelte"], TechnologyCategory::Framework, 85),
        ];

        for (name, patterns, category, confidence) in frameworks {
            if patterns.iter().any(|p| html_lower.contains(p)) {
                techs.push(DetectedTechnology {
                    id: Uuid::new_v4().to_string(),
                    task_id: task_id.to_string(),
                    technology_name: name.to_string(),
                    technology_version: None,
                    category,
                    confidence,
                    created_at: Utc::now(),
                });
            }
        }

        techs
    }

    fn detect_css_frameworks(&self, task_id: &str, html: &str) -> Vec<DetectedTechnology> {
        let mut techs = Vec::new();
        let html_lower = html.to_lowercase();

        if html_lower.contains("bootstrap") || html_lower.contains("btn btn-") {
            techs.push(DetectedTechnology {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                technology_name: "Bootstrap".to_string(),
                technology_version: None,
                category: TechnologyCategory::Framework,
                confidence: 80,
                created_at: Utc::now(),
            });
        }

        if self.has_tailwind_classes(html) {
            techs.push(DetectedTechnology {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                technology_name: "Tailwind CSS".to_string(),
                technology_version: None,
                category: TechnologyCategory::Framework,
                confidence: 75,
                created_at: Utc::now(),
            });
        }

        techs
    }

    fn detect_analytics(&self, task_id: &str, html: &str) -> Vec<DetectedTechnology> {
        let mut techs = Vec::new();
        let html_lower = html.to_lowercase();

        let analytics = vec![
            ("Google Analytics", vec!["google-analytics.com", "gtag", "ga.js"], 95),
            ("Google Tag Manager", vec!["googletagmanager.com", "gtm.js"], 95),
            ("Facebook Pixel", vec!["facebook.net/en_us/fbevents.js", "fbq("], 90),
            ("Hotjar", vec!["hotjar.com", "hjid"], 90),
            ("Mixpanel", vec!["mixpanel.com", "mixpanel"], 85),
        ];

        for (name, patterns, confidence) in analytics {
            if patterns.iter().any(|p| html_lower.contains(p)) {
                techs.push(DetectedTechnology {
                    id: Uuid::new_v4().to_string(),
                    task_id: task_id.to_string(),
                    technology_name: name.to_string(),
                    technology_version: None,
                    category: TechnologyCategory::Analytics,
                    confidence,
                    created_at: Utc::now(),
                });
            }
        }

        techs
    }

    fn detect_cdn(&self, task_id: &str, html: &str, headers: &reqwest::header::HeaderMap) -> Vec<DetectedTechnology> {
        let mut techs = Vec::new();
        let html_lower = html.to_lowercase();

        // 從 CDN URL 檢測
        let cdns = vec![
            ("Cloudflare", vec!["cloudflare.com", "cf-ray"], 90),
            ("Fastly", vec!["fastly.net"], 85),
            ("Akamai", vec!["akamai.net", "akamaihd.net"], 85),
            ("Amazon CloudFront", vec!["cloudfront.net"], 90),
        ];

        for (name, patterns, confidence) in cdns {
            let in_html = patterns.iter().any(|p| html_lower.contains(p));
            let in_headers = headers.iter().any(|(k, v)| {
                let key = k.as_str().to_lowercase();
                let value = v.to_str().unwrap_or("").to_lowercase();
                patterns.iter().any(|p| key.contains(p) || value.contains(p))
            });

            if in_html || in_headers {
                techs.push(DetectedTechnology {
                    id: Uuid::new_v4().to_string(),
                    task_id: task_id.to_string(),
                    technology_name: name.to_string(),
                    technology_version: None,
                    category: TechnologyCategory::Cdn,
                    confidence,
                    created_at: Utc::now(),
                });
            }
        }

        techs
    }

    fn has_tailwind_classes(&self, html: &str) -> bool {
        let tailwind_patterns = [
            "flex-", "grid-", "bg-", "text-", "p-", "m-", "w-", "h-",
            "rounded-", "shadow-", "hover:", "focus:", "md:", "lg:"
        ];
        let count = tailwind_patterns.iter().filter(|pattern| html.contains(*pattern)).count();
        count >= 3 // 如果有 3 個以上的 Tailwind 類別，就認為使用了 Tailwind
    }
}
