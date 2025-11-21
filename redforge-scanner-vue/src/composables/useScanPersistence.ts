/**
 * Scan Persistence Composable
 *
 * Handles automatic saving and loading of scan results to/from database
 */

import { invoke } from '@tauri-apps/api/core';
import {
  insertScanTask,
  insertScanResults,
  getAllScanTasks,
  type DbScanTask,
  type DbScanResult,
} from '@/services/database';

interface ScanTask {
  id: string;
  target_url: string;
  scan_type: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  started_at?: string;
  completed_at?: string;
  created_at: string;
}

interface ScanReport {
  task: ScanTask;
  headers: any[];
  ssl_analysis: any;
  technologies: any[];
  vulnerabilities: any[];
}

/**
 * Save scan to database when completed
 */
export async function saveScanToDatabase(taskId: string): Promise<void> {
  try {
    console.log(`💾 開始保存掃描結果到資料庫: ${taskId}`);

    // Get scan task and report from backend
    const task = await invoke<ScanTask>('get_scan_status', { taskId });
    const report = await invoke<ScanReport>('get_scan_report', { taskId });

    // Save scan task
    await insertScanTask({
      id: task.id,
      target_url: task.target_url,
      scan_type: task.scan_type,
      status: task.status,
      created_at: task.created_at,
      started_at: task.started_at,
      completed_at: task.completed_at,
    });

    // Save scan results (vulnerabilities)
    if (report.vulnerabilities && report.vulnerabilities.length > 0) {
      const results: DbScanResult[] = report.vulnerabilities.map((vuln: any) => ({
        id: vuln.id,
        task_id: task.id,
        result_type: vuln.result_type || 'vulnerability',
        severity: vuln.severity,
        title: vuln.title,
        description: vuln.description,
        raw_data: vuln.raw_data,
        created_at: vuln.created_at,
      }));

      await insertScanResults(results);
    }

    console.log(`✅ 成功保存掃描結果: ${taskId} (${report.vulnerabilities.length} 個漏洞)`);
  } catch (error) {
    console.error(`❌ 保存掃描結果失敗: ${taskId}`, error);
    throw error;
  }
}

/**
 * Load all scan history from database
 */
export async function loadScanHistory(): Promise<DbScanTask[]> {
  try {
    console.log('📂 從資料庫載入掃描歷史...');

    const scans = await getAllScanTasks();

    console.log(`✅ 成功載入 ${scans.length} 筆掃描記錄`);

    return scans;
  } catch (error) {
    console.error('❌ 載入掃描歷史失敗:', error);
    throw error;
  }
}

/**
 * Poll scan status and save when completed
 */
export async function pollScanAndSave(taskId: string, onUpdate?: (task: ScanTask) => void): Promise<void> {
  const pollInterval = 1000; // 每秒檢查一次
  const maxAttempts = 300; // 最多等待 5 分鐘

  let attempts = 0;

  return new Promise((resolve, reject) => {
    const intervalId = setInterval(async () => {
      attempts++;

      try {
        const task = await invoke<ScanTask>('get_scan_status', { taskId });

        // 通知外部狀態更新
        if (onUpdate) {
          onUpdate(task);
        }

        // 檢查是否完成
        if (task.status === 'completed' || task.status === 'failed') {
          clearInterval(intervalId);

          // 自動保存到資料庫
          if (task.status === 'completed') {
            try {
              await saveScanToDatabase(taskId);
            } catch (error) {
              console.error('保存到資料庫失敗，但掃描已完成:', error);
            }
          }

          resolve();
        }

        // 檢查是否超時
        if (attempts >= maxAttempts) {
          clearInterval(intervalId);
          reject(new Error('掃描超時'));
        }
      } catch (error) {
        clearInterval(intervalId);
        reject(error);
      }
    }, pollInterval);
  });
}

/**
 * Start scan and auto-save when completed
 */
export async function startScanWithAutoSave(
  url: string,
  scanType: string,
  onUpdate?: (task: ScanTask) => void
): Promise<string> {
  try {
    console.log(`🚀 開始掃描: ${url} (${scanType})`);

    // Start scan
    const taskId = await invoke<string>('start_scan', {
      url,
      scanType,
    });

    console.log(`✅ 掃描任務已建立: ${taskId}`);

    // Poll and auto-save in background
    pollScanAndSave(taskId, onUpdate).catch((error) => {
      console.error('掃描輪詢失敗:', error);
    });

    return taskId;
  } catch (error) {
    console.error('啟動掃描失敗:', error);
    throw error;
  }
}
