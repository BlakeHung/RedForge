/**
 * Database Service
 *
 * Handles SQLite database operations using tauri-plugin-sql
 * Provides persistence for scan tasks, results, and collaboration data
 */

import Database from '@tauri-apps/plugin-sql';

let db: Database | null = null;

/**
 * Initialize database connection
 */
export async function initDatabase(): Promise<void> {
  try {
    db = await Database.load('sqlite:redforge.db');
    console.log('✅ Database initialized successfully');
  } catch (error) {
    console.error('❌ Failed to initialize database:', error);
    throw error;
  }
}

/**
 * Get database instance
 */
function getDb(): Database {
  if (!db) {
    throw new Error('Database not initialized. Call initDatabase() first.');
  }
  return db;
}

// =============================================================================
// Scan Tasks Operations
// =============================================================================

export interface DbScanTask {
  id: string;
  target_url: string;
  scan_type: string;
  status: string;
  started_at: string | null;
  completed_at: string | null;
  created_at: string;
  created_by: string;
}

/**
 * Insert scan task into database
 */
export async function insertScanTask(task: {
  id: string;
  target_url: string;
  scan_type: string;
  status: string;
  created_at: string;
  started_at?: string;
  completed_at?: string;
}): Promise<void> {
  const database = getDb();

  await database.execute(
    `INSERT INTO scan_tasks (id, target_url, scan_type, status, started_at, completed_at, created_at)
     VALUES ($1, $2, $3, $4, $5, $6, $7)`,
    [
      task.id,
      task.target_url,
      task.scan_type,
      task.status,
      task.started_at || null,
      task.completed_at || null,
      task.created_at,
    ]
  );

  console.log(`✅ Inserted scan task: ${task.id}`);
}

/**
 * Update scan task status
 */
export async function updateScanTaskStatus(
  taskId: string,
  status: string,
  startedAt?: string,
  completedAt?: string
): Promise<void> {
  const database = getDb();

  await database.execute(
    `UPDATE scan_tasks
     SET status = $1, started_at = $2, completed_at = $3
     WHERE id = $4`,
    [status, startedAt || null, completedAt || null, taskId]
  );

  console.log(`✅ Updated scan task ${taskId} status to: ${status}`);
}

/**
 * Get all scan tasks
 */
export async function getAllScanTasks(): Promise<DbScanTask[]> {
  const database = getDb();

  const result = await database.select<DbScanTask[]>(
    'SELECT * FROM scan_tasks ORDER BY created_at DESC'
  );

  return result;
}

/**
 * Get scan task by ID
 */
export async function getScanTask(taskId: string): Promise<DbScanTask | null> {
  const database = getDb();

  const result = await database.select<DbScanTask[]>(
    'SELECT * FROM scan_tasks WHERE id = $1',
    [taskId]
  );

  return result[0] || null;
}

/**
 * Delete scan task
 */
export async function deleteScanTask(taskId: string): Promise<void> {
  const database = getDb();

  await database.execute('DELETE FROM scan_tasks WHERE id = $1', [taskId]);

  console.log(`✅ Deleted scan task: ${taskId}`);
}

// =============================================================================
// Scan Results Operations
// =============================================================================

export interface DbScanResult {
  id: string;
  task_id: string;
  result_type: string;
  severity?: string;
  title: string;
  description?: string;
  raw_data?: string;
  created_at: string;
}

/**
 * Insert scan result
 */
export async function insertScanResult(result: {
  id: string;
  task_id: string;
  result_type: string;
  severity?: string;
  title: string;
  description?: string;
  raw_data?: string;
  created_at: string;
}): Promise<void> {
  const database = getDb();

  await database.execute(
    `INSERT INTO scan_results (id, task_id, result_type, severity, title, description, raw_data, created_at)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
    [
      result.id,
      result.task_id,
      result.result_type,
      result.severity || null,
      result.title,
      result.description || null,
      result.raw_data || null,
      result.created_at,
    ]
  );
}

/**
 * Batch insert scan results
 */
export async function insertScanResults(results: DbScanResult[]): Promise<void> {
  for (const result of results) {
    await insertScanResult(result);
  }
  console.log(`✅ Inserted ${results.length} scan results`);
}

/**
 * Get scan results by task ID
 */
export async function getScanResultsByTask(taskId: string): Promise<DbScanResult[]> {
  const database = getDb();

  const results = await database.select<DbScanResult[]>(
    'SELECT * FROM scan_results WHERE task_id = $1 ORDER BY created_at DESC',
    [taskId]
  );

  return results;
}

/**
 * Get database statistics
 */
export async function getDatabaseStats(): Promise<{
  totalScans: number;
  totalFindings: number;
  criticalFindings: number;
  highFindings: number;
  mediumFindings: number;
  lowFindings: number;
  infoFindings: number;
}> {
  const database = getDb();

  // Count scans
  const scanCount = await database.select<Array<{ count: number }>>(
    'SELECT COUNT(*) as count FROM scan_tasks'
  );

  // Count all findings
  const findingCount = await database.select<Array<{ count: number }>>(
    'SELECT COUNT(*) as count FROM scan_results'
  );

  // Count by severity
  const severityCounts = await database.select<Array<{ severity: string; count: number }>>(
    'SELECT severity, COUNT(*) as count FROM scan_results WHERE severity IS NOT NULL GROUP BY severity'
  );

  const stats = {
    totalScans: scanCount[0]?.count || 0,
    totalFindings: findingCount[0]?.count || 0,
    criticalFindings: 0,
    highFindings: 0,
    mediumFindings: 0,
    lowFindings: 0,
    infoFindings: 0,
  };

  severityCounts.forEach((row: { severity: string; count: number }) => {
    switch (row.severity) {
      case 'critical':
        stats.criticalFindings = row.count;
        break;
      case 'high':
        stats.highFindings = row.count;
        break;
      case 'medium':
        stats.mediumFindings = row.count;
        break;
      case 'low':
        stats.lowFindings = row.count;
        break;
      case 'info':
        stats.infoFindings = row.count;
        break;
    }
  });

  return stats;
}
