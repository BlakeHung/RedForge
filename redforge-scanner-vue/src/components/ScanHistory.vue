<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface ScanTask {
  id: string;
  target_url: string;
  scan_type: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  started_at?: string;
  completed_at?: string;
  created_at: string;
}

const scans = ref<ScanTask[]>([]);
const loading = ref(true);

onMounted(() => {
  loadScans();
});

const loadScans = async () => {
  try {
    const scanList = await invoke<ScanTask[]>('list_scans');
    scans.value = scanList.reverse(); // 最新的在前面
  } catch (error) {
    console.error('Failed to load scans:', error);
  } finally {
    loading.value = false;
  }
};

const generateMarkdownReport = (report: any) => {
  const task = report.task;
  let md = `# 🔥 RedForge 安全掃描報告

## 📋 掃描資訊

- **任務 ID**: \`${task.id}\`
- **目標 URL**: ${task.target_url}
- **掃描類型**: ${task.scan_type.toUpperCase()}
- **狀態**: ${task.status.toUpperCase()}
- **建立時間**: ${new Date(task.created_at).toLocaleString('zh-TW')}
${task.started_at ? `- **開始時間**: ${new Date(task.started_at).toLocaleString('zh-TW')}\n` : ''}${task.completed_at ? `- **完成時間**: ${new Date(task.completed_at).toLocaleString('zh-TW')}\n` : ''}

---

`;

  // HTTP 安全標頭
  if (report.headers && report.headers.length > 0) {
    md += `## 🛡️ HTTP 安全標頭分析

`;
    const secureHeaders = report.headers.filter((h: any) => h.is_secure);
    const insecureHeaders = report.headers.filter((h: any) => !h.is_secure);

    md += `### ✅ 安全標頭 (${secureHeaders.length})\n\n`;
    secureHeaders.forEach((h: any) => {
      md += `- **${h.header_name}**: \`${h.header_value || '未設置'}\`\n`;
      if (h.recommendation) md += `  - 💡 ${h.recommendation}\n`;
    });

    md += `\n### ⚠️ 缺失或不安全的標頭 (${insecureHeaders.length})\n\n`;
    insecureHeaders.forEach((h: any) => {
      md += `- **${h.header_name}**: ${h.is_present ? `\`${h.header_value}\`` : '❌ 未設置'}\n`;
      if (h.recommendation) md += `  - 💡 建議: ${h.recommendation}\n`;
    });

    md += '\n---\n\n';
  }

  // SSL/TLS 分析
  if (report.ssl_analysis) {
    const ssl = report.ssl_analysis;
    md += `## 🔒 SSL/TLS 安全分析

- **安全等級**: ${ssl.grade || 'N/A'}
- **證書主體**: ${ssl.certificate_subject || 'N/A'}
- **TLS 版本**: ${ssl.tls_versions?.join(', ') || 'N/A'}
`;
    if (ssl.vulnerabilities && ssl.vulnerabilities.length > 0) {
      md += `\n### ⚠️ 發現的問題\n\n`;
      ssl.vulnerabilities.forEach((v: string) => {
        md += `- ${v}\n`;
      });
    }
    md += '\n---\n\n';
  }

  // 技術棧檢測
  if (report.technologies && report.technologies.length > 0) {
    md += `## 🔧 檢測到的技術 (${report.technologies.length})\n\n`;
    const grouped = report.technologies.reduce((acc: any, tech: any) => {
      const cat = tech.category || 'other';
      if (!acc[cat]) acc[cat] = [];
      acc[cat].push(tech);
      return acc;
    }, {});

    Object.entries(grouped).forEach(([category, techs]: [string, any]) => {
      md += `### ${category.toUpperCase()}\n\n`;
      techs.forEach((tech: any) => {
        md += `- **${tech.technology_name}**`;
        if (tech.technology_version) md += ` v${tech.technology_version}`;
        md += ` (信心度: ${tech.confidence}%)\n`;
      });
      md += '\n';
    });

    md += '---\n\n';
  }

  // 漏洞發現
  if (report.vulnerabilities && report.vulnerabilities.length > 0) {
    md += `## 🚨 漏洞發現 (${report.vulnerabilities.length})\n\n`;

    const bySeverity = {
      critical: report.vulnerabilities.filter((v: any) => v.severity === 'critical'),
      high: report.vulnerabilities.filter((v: any) => v.severity === 'high'),
      medium: report.vulnerabilities.filter((v: any) => v.severity === 'medium'),
      low: report.vulnerabilities.filter((v: any) => v.severity === 'low'),
      info: report.vulnerabilities.filter((v: any) => v.severity === 'info'),
    };

    Object.entries(bySeverity).forEach(([severity, vulns]: [string, any]) => {
      if (vulns.length > 0) {
        const emoji = { critical: '🔴', high: '🟠', medium: '🟡', low: '🟢', info: '🔵' }[severity];
        md += `### ${emoji} ${severity.toUpperCase()} (${vulns.length})\n\n`;
        vulns.forEach((vuln: any, idx: number) => {
          md += `#### ${idx + 1}. ${vuln.title}\n\n`;
          if (vuln.description) md += `${vuln.description}\n\n`;
          if (vuln.raw_data) {
            try {
              const data = JSON.parse(vuln.raw_data);
              md += '**詳細資訊**:\n```json\n' + JSON.stringify(data, null, 2) + '\n```\n\n';
            } catch {}
          }
        });
      }
    });

    md += '---\n\n';
  }

  // 總結
  md += `## 📊 掃描總結

- **檢測到的安全標頭**: ${report.headers?.length || 0}
- **檢測到的技術**: ${report.technologies?.length || 0}
- **發現的漏洞**: ${report.vulnerabilities?.length || 0}
${report.ssl_analysis ? `- **SSL 安全等級**: ${report.ssl_analysis.grade || 'N/A'}\n` : ''}

---

*📅 報告生成時間: ${new Date().toLocaleString('zh-TW')}*
*🔧 由 RedForge Scanner 自動生成*
*⚠️ 本報告僅供授權測試使用*
`;

  return md;
};

const downloadReport = async (scan: ScanTask, format: 'json' | 'markdown') => {
  try {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
    const defaultFileName = `scan-report-${scan.id.slice(0, 8)}-${timestamp}.${format === 'json' ? 'json' : 'md'}`;

    // 獲取完整報告
    let fullReport;
    try {
      fullReport = await invoke('get_scan_report', { taskId: scan.id });
    } catch (error) {
      console.warn('無法獲取完整報告，使用基本資訊:', error);
      fullReport = { task: scan, headers: [], technologies: [], vulnerabilities: [] };
    }

    // 生成報告內容
    let content = '';
    if (format === 'json') {
      content = JSON.stringify(fullReport, null, 2);
    } else {
      content = generateMarkdownReport(fullReport);
    }

    // 使用瀏覽器下載
    const blob = new Blob([content], { type: format === 'json' ? 'application/json' : 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = defaultFileName;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    console.log(`報告已下載: ${defaultFileName}`);
  } catch (error) {
    console.error('下載失敗:', error);
    alert('下載失敗: ' + error);
  }
};

const getStatusBadgeClass = (status: string) => {
  const styles = {
    completed: 'bg-success-900/30 text-success-400 border-success-700',
    failed: 'bg-danger-900/30 text-danger-400 border-danger-700',
    running: 'bg-info-900/30 text-info-400 border-info-700',
    pending: 'bg-warning-900/30 text-warning-400 border-warning-700',
  };
  return styles[status as keyof typeof styles] || styles.pending;
};
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h2 class="text-2xl font-bold text-white flex items-center">
        <svg class="w-6 h-6 mr-2 text-danger-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        掃描歷史
      </h2>
      <button
        @click="loadScans"
        class="px-4 py-2 bg-dark-700 hover:bg-dark-600 text-white rounded-lg transition-colors"
      >
        重新載入
      </button>
    </div>

    <div v-if="loading" class="flex items-center justify-center h-64">
      <div class="text-dark-400">載入中...</div>
    </div>

    <div v-else-if="scans.length === 0" class="bg-dark-800 rounded-lg border border-dark-700 p-12 text-center">
      <p class="text-dark-400">尚無掃描記錄</p>
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="scan in scans"
        :key="scan.id"
        class="bg-dark-800 rounded-lg border border-dark-700 p-5 hover:border-dark-600 transition-colors"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1">
            <div class="flex items-center space-x-3 mb-2">
              <span :class="['px-2 py-1 text-xs font-semibold rounded border', getStatusBadgeClass(scan.status)]">
                {{ scan.status.toUpperCase() }}
              </span>
              <span class="text-sm text-dark-400 font-mono">
                ID: {{ scan.id.slice(0, 8) }}
              </span>
            </div>

            <div class="flex items-center space-x-2 mb-3">
              <svg class="w-4 h-4 text-dark-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
              <a
                :href="scan.target_url"
                target="_blank"
                rel="noopener noreferrer"
                class="text-white hover:text-danger-400 font-medium"
              >
                {{ scan.target_url }}
              </a>
            </div>

            <div class="grid grid-cols-3 gap-4 text-sm">
              <div>
                <span class="text-dark-400">掃描類型:</span>
                <span class="text-white ml-2 uppercase font-semibold">
                  {{ scan.scan_type }}
                </span>
              </div>
              <div>
                <span class="text-dark-400">建立時間:</span>
                <span class="text-white ml-2">
                  {{ new Date(scan.created_at).toLocaleString('zh-TW') }}
                </span>
              </div>
              <div v-if="scan.completed_at">
                <span class="text-dark-400">完成時間:</span>
                <span class="text-white ml-2">
                  {{ new Date(scan.completed_at).toLocaleString('zh-TW') }}
                </span>
              </div>
            </div>
          </div>

          <div class="flex space-x-2">
            <div class="relative group">
              <button class="p-2 bg-dark-700 hover:bg-dark-600 rounded-lg transition-colors">
                <svg class="w-4 h-4 text-dark-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                </svg>
              </button>

              <!-- 下載選項下拉菜單 -->
              <div class="absolute right-0 mt-2 w-40 bg-dark-700 border border-dark-600 rounded-lg shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-10">
                <button
                  @click="downloadReport(scan, 'json')"
                  class="w-full px-4 py-2 text-left text-sm text-white hover:bg-dark-600 rounded-t-lg flex items-center space-x-2"
                >
                  <svg class="w-4 h-4 text-info-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                  <span>JSON 格式</span>
                </button>
                <button
                  @click="downloadReport(scan, 'markdown')"
                  class="w-full px-4 py-2 text-left text-sm text-white hover:bg-dark-600 rounded-b-lg flex items-center space-x-2"
                >
                  <svg class="w-4 h-4 text-success-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                  <span>Markdown 格式</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
