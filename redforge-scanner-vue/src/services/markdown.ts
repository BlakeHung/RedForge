/**
 * Markdown Generation Service
 *
 * Handles generation and parsing of encrypted Markdown export files
 * Format: YAML frontmatter + encrypted JSON block + human-readable summary
 */

import type {
  ExportData,
  EncryptedData,
  MarkdownExport,
  ExportMetadata,
  ExportSummary,
} from '../types/offline-collaboration';

export class MarkdownService {
  /**
   * Generate Markdown export file with optional encryption
   *
   * @param data - Export data
   * @param encrypted - Optional encrypted data block
   * @returns Formatted Markdown string
   */
  generateMarkdown(data: ExportData, encrypted?: EncryptedData): string {
    const lines: string[] = [];

    // YAML Frontmatter
    lines.push('---');
    lines.push(`version: "${data.metadata.version}"`);
    lines.push(`format: "${data.metadata.format}"`);
    if (encrypted) {
      lines.push(`encryption: "${data.metadata.encryption}"`);
    }
    lines.push(`exported_by: "${data.metadata.exported_by}"`);
    if (data.metadata.team_id) {
      lines.push(`team_id: "${data.metadata.team_id}"`);
    }
    lines.push(`exported_at: "${data.metadata.exported_at}"`);
    if (data.metadata.checksum) {
      lines.push(`checksum: "${data.metadata.checksum}"`);
    }
    lines.push('---');
    lines.push('');

    // Title
    lines.push('# RedForge 掃描資料匯出');
    lines.push('');

    // Encrypted block (if present)
    if (encrypted) {
      lines.push('## 🔐 加密資料區塊');
      lines.push('');
      lines.push('```encrypted');
      lines.push(encrypted.ciphertext);
      lines.push('```');
      lines.push('');
      lines.push(`**IV**: \`${encrypted.iv}\``);
      lines.push('');
      lines.push(`**Salt**: \`${encrypted.salt}\``);
      lines.push('');
      lines.push('---');
      lines.push('');
    }

    // Summary section (always human-readable)
    lines.push('## 📊 資料摘要');
    lines.push('');
    const summary = this.generateSummary(data);
    lines.push(`- **掃描任務數**: ${summary.totalScans}`);
    lines.push(`- **漏洞總數**: ${summary.totalFindings}`);
    lines.push('  - 🔴 Critical: ' + summary.findingsBySeverity.critical);
    lines.push('  - 🟠 High: ' + summary.findingsBySeverity.high);
    lines.push('  - 🟡 Medium: ' + summary.findingsBySeverity.medium);
    lines.push('  - 🟢 Low: ' + summary.findingsBySeverity.low);
    lines.push('  - ℹ️  Info: ' + summary.findingsBySeverity.info);

    if (summary.totalAnnotations !== undefined && summary.totalAnnotations > 0) {
      lines.push(`- **註記數**: ${summary.totalAnnotations}`);
    }

    if (summary.totalAssets !== undefined && summary.totalAssets > 0) {
      lines.push(`- **資產數**: ${summary.totalAssets}`);
    }

    if (summary.dateRange) {
      lines.push(`- **時間範圍**: ${summary.dateRange.from} ~ ${summary.dateRange.to}`);
    }
    lines.push('');

    // Scans overview
    if (data.scans.length > 0) {
      lines.push('## 🎯 掃描任務列表');
      lines.push('');
      data.scans.forEach((scan, index) => {
        lines.push(`### ${index + 1}. ${scan.name}`);
        lines.push(`- **Target**: ${scan.target}`);
        lines.push(`- **Status**: ${this.getStatusEmoji(scan.status)} ${scan.status}`);
        lines.push(`- **Created**: ${new Date(scan.created_at).toLocaleString('zh-TW')}`);
        if (scan.completed_at) {
          lines.push(`- **Completed**: ${new Date(scan.completed_at).toLocaleString('zh-TW')}`);
        }
        lines.push('');
      });
    }

    // Footer
    lines.push('---');
    lines.push('');
    lines.push('*此檔案由 RedForge Scanner 生成*');
    lines.push('');
    if (encrypted) {
      lines.push('⚠️ **注意**: 此檔案包含加密資料，需要正確的密碼才能解密導入。');
    } else {
      lines.push('ℹ️ **提示**: 此檔案未加密，包含明文掃描資料。');
    }

    return lines.join('\n');
  }

  /**
   * Parse Markdown export file
   *
   * @param markdown - Markdown content to parse
   * @returns Parsed export data
   */
  parseMarkdown(markdown: string): MarkdownExport {
    const lines = markdown.split('\n');

    // Parse frontmatter
    const frontMatter = this.parseFrontMatter(lines);

    // Parse encrypted block if present
    const encryptedBlock = frontMatter.encryption
      ? this.parseEncryptedBlock(lines)
      : undefined;

    // Parse summary (for display purposes)
    const summary = this.parseSummary(lines);

    return {
      frontMatter,
      encryptedBlock,
      summary,
      rawContent: markdown,
    };
  }

  /**
   * Parse YAML frontmatter
   */
  private parseFrontMatter(lines: string[]): ExportMetadata {
    const frontMatterLines: string[] = [];
    let inFrontMatter = false;

    for (let i = 0; i < lines.length; i++) {
      if (lines[i].trim() === '---') {
        if (!inFrontMatter) {
          inFrontMatter = true;
        } else {
          break;
        }
      } else if (inFrontMatter) {
        frontMatterLines.push(lines[i]);
      }
    }

    const metadata: Partial<ExportMetadata> = {};

    frontMatterLines.forEach(line => {
      const match = line.match(/^(\w+):\s*"?(.+?)"?$/);
      if (match) {
        const [, key, value] = match;
        (metadata as any)[key] = value;
      }
    });

    return metadata as ExportMetadata;
  }

  /**
   * Parse encrypted data block
   */
  private parseEncryptedBlock(lines: string[]): EncryptedData | undefined {
    let inEncryptedBlock = false;
    let ciphertext = '';
    let iv = '';
    let salt = '';

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();

      if (line === '```encrypted') {
        inEncryptedBlock = true;
        continue;
      }

      if (line === '```' && inEncryptedBlock) {
        inEncryptedBlock = false;
        continue;
      }

      if (inEncryptedBlock) {
        ciphertext += line;
      }

      if (line.startsWith('**IV**:')) {
        const match = line.match(/`(.+)`/);
        if (match) iv = match[1];
      }

      if (line.startsWith('**Salt**:')) {
        const match = line.match(/`(.+)`/);
        if (match) salt = match[1];
      }
    }

    if (ciphertext && iv && salt) {
      return { ciphertext, iv, salt };
    }

    return undefined;
  }

  /**
   * Parse summary section
   */
  private parseSummary(lines: string[]): ExportSummary {
    const summary: ExportSummary = {
      totalScans: 0,
      totalFindings: 0,
      findingsBySeverity: {
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
        info: 0,
      },
    };

    for (const line of lines) {
      const scanMatch = line.match(/掃描任務數[*:：]+\s*(\d+)/);
      if (scanMatch) summary.totalScans = parseInt(scanMatch[1]);

      const findingsMatch = line.match(/漏洞總數[*:：]+\s*(\d+)/);
      if (findingsMatch) summary.totalFindings = parseInt(findingsMatch[1]);

      const criticalMatch = line.match(/Critical:\s*(\d+)/);
      if (criticalMatch) summary.findingsBySeverity.critical = parseInt(criticalMatch[1]);

      const highMatch = line.match(/High:\s*(\d+)/);
      if (highMatch) summary.findingsBySeverity.high = parseInt(highMatch[1]);

      const mediumMatch = line.match(/Medium:\s*(\d+)/);
      if (mediumMatch) summary.findingsBySeverity.medium = parseInt(mediumMatch[1]);

      const lowMatch = line.match(/Low:\s*(\d+)/);
      if (lowMatch) summary.findingsBySeverity.low = parseInt(lowMatch[1]);

      const infoMatch = line.match(/Info:\s*(\d+)/);
      if (infoMatch) summary.findingsBySeverity.info = parseInt(infoMatch[1]);
    }

    return summary;
  }

  /**
   * Generate summary from export data
   */
  private generateSummary(data: ExportData): ExportSummary {
    const summary: ExportSummary = {
      totalScans: data.scans.length,
      totalFindings: data.findings.length,
      findingsBySeverity: {
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
        info: 0,
      },
    };

    // Count findings by severity
    data.findings.forEach(finding => {
      summary.findingsBySeverity[finding.severity]++;
    });

    // Optional counts
    if (data.annotations && data.annotations.length > 0) {
      summary.totalAnnotations = data.annotations.length;
    }

    if (data.assets && data.assets.length > 0) {
      summary.totalAssets = data.assets.length;
    }

    // Date range
    if (data.scans.length > 0) {
      const dates = data.scans
        .map(s => new Date(s.created_at).getTime())
        .sort((a, b) => a - b);

      summary.dateRange = {
        from: new Date(dates[0]).toLocaleDateString('zh-TW'),
        to: new Date(dates[dates.length - 1]).toLocaleDateString('zh-TW'),
      };
    }

    return summary;
  }

  /**
   * Get emoji for scan status
   */
  private getStatusEmoji(status: string): string {
    const emojiMap: Record<string, string> = {
      'pending': '⏸️',
      'running': '▶️',
      'completed': '✅',
      'failed': '❌',
    };
    return emojiMap[status] || '❓';
  }

  /**
   * Calculate MD5 checksum of data
   */
  async calculateChecksum(data: string): Promise<string> {
    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(data);
    const hashBuffer = await crypto.subtle.digest('SHA-256', dataBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  }
}

// Export singleton instance
export const markdownService = new MarkdownService();
