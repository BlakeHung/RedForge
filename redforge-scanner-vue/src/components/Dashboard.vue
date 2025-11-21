<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import ExportDialog from './collaboration/ExportDialog.vue';
import ImportDialog from './collaboration/ImportDialog.vue';
import { loadScanHistory } from '@/composables/useScanPersistence';

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
const showExportDialog = ref(false);
const showImportDialog = ref(false);

onMounted(async () => {
  try {
    // 1. 從資料庫載入歷史記錄
    try {
      const dbScans = await loadScanHistory();
      // 轉換資料庫格式到前端格式
      const historicalScans: ScanTask[] = dbScans.map(scan => ({
        id: scan.id,
        target_url: scan.target_url,
        scan_type: scan.scan_type,
        status: scan.status as 'pending' | 'running' | 'completed' | 'failed',
        started_at: scan.started_at || undefined,
        completed_at: scan.completed_at || undefined,
        created_at: scan.created_at,
      }));
      scans.value = historicalScans;
      console.log(`✅ 從資料庫載入 ${historicalScans.length} 筆歷史記錄`);
    } catch (dbError) {
      console.warn('無法從資料庫載入歷史:', dbError);
    }

    // 2. 從記憶體獲取當前正在執行的掃描
    try {
      const runtimeScans = await invoke<ScanTask[]>('list_scans');

      // 合併資料：runtime 掃描優先（可能有更新的狀態）
      const scanMap = new Map<string, ScanTask>();

      // 先加入歷史記錄
      scans.value.forEach(scan => scanMap.set(scan.id, scan));

      // runtime 掃描覆蓋（如果有的話）
      runtimeScans.forEach(scan => scanMap.set(scan.id, scan));

      scans.value = Array.from(scanMap.values());
      console.log(`✅ 總共載入 ${scans.value.length} 筆掃描記錄`);
    } catch (runtimeError) {
      console.warn('無法從記憶體載入掃描:', runtimeError);
    }
  } catch (error) {
    console.error('載入掃描失敗:', error);
  }
});

const stats = computed(() => ({
  total: scans.value.length,
  completed: scans.value.filter(s => s.status === 'completed').length,
  failed: scans.value.filter(s => s.status === 'failed').length,
  running: scans.value.filter(s => s.status === 'running').length,
}));

const scanTypeStats = computed(() => {
  const types = scans.value.reduce((acc, scan) => {
    acc[scan.scan_type] = (acc[scan.scan_type] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);
  return Object.entries(types).map(([name, value]) => ({ name, value }));
});

const statusDistribution = computed(() => [
  { name: '已完成', value: stats.value.completed, color: 'bg-success-500' },
  { name: '失敗', value: stats.value.failed, color: 'bg-danger-500' },
  { name: '執行中', value: stats.value.running, color: 'bg-info-500' },
]);

const recentScans = computed(() =>
  [...scans.value]
    .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    .slice(0, 5)
);
</script>

<template>
  <div class="space-y-6">
    <h2 class="text-2xl font-bold text-white flex items-center">
      <svg class="w-6 h-6 mr-2 text-danger-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
      統計儀表板
    </h2>

    <!-- 協作工具列 -->
    <div class="flex items-center gap-3 p-4 bg-gray-800 border border-gray-700 rounded-lg">
      <div class="flex-1">
        <h3 class="font-mono font-semibold text-gray-300 text-sm">🤝 離線協作</h3>
        <p class="text-xs text-gray-500 mt-1">匯出/匯入加密資料進行團隊協作</p>
      </div>
      <button
        @click="showExportDialog = true"
        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white border border-blue-500 rounded font-mono text-sm transition-colors"
      >
        🔐 匯出資料
      </button>
      <button
        @click="showImportDialog = true"
        class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-100 border border-gray-600 rounded font-mono text-sm transition-colors"
      >
        📥 匯入資料
      </button>
    </div>

    <!-- 統計卡片 -->
    <div class="grid grid-cols-4 gap-4">
      <div class="bg-dark-800 rounded-lg border border-dark-700 p-6">
        <div class="text-dark-400 text-sm mb-1">總掃描次數</div>
        <div class="text-3xl font-bold text-white">{{ stats.total }}</div>
      </div>
      <div class="bg-dark-800 rounded-lg border border-success-700 p-6">
        <div class="text-dark-400 text-sm mb-1">已完成</div>
        <div class="text-3xl font-bold text-success-500">{{ stats.completed }}</div>
      </div>
      <div class="bg-dark-800 rounded-lg border border-danger-700 p-6">
        <div class="text-dark-400 text-sm mb-1">失敗</div>
        <div class="text-3xl font-bold text-danger-500">{{ stats.failed }}</div>
      </div>
      <div class="bg-dark-800 rounded-lg border border-info-700 p-6">
        <div class="text-dark-400 text-sm mb-1">執行中</div>
        <div class="text-3xl font-bold text-info-500">{{ stats.running }}</div>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-6">
      <!-- 狀態分布 -->
      <div class="bg-dark-800 rounded-lg border border-dark-700 p-6">
        <h3 class="text-lg font-semibold text-white mb-4">狀態分布</h3>
        <div class="space-y-3">
          <div
            v-for="item in statusDistribution"
            :key="item.name"
            class="flex items-center justify-between"
          >
            <div class="flex items-center space-x-3">
              <div :class="['w-3 h-3 rounded-full', item.color]"></div>
              <span class="text-white">{{ item.name }}</span>
            </div>
            <span class="text-dark-400 font-mono">{{ item.value }}</span>
          </div>
        </div>
      </div>

      <!-- 掃描類型統計 -->
      <div class="bg-dark-800 rounded-lg border border-dark-700 p-6">
        <h3 class="text-lg font-semibold text-white mb-4">掃描類型統計</h3>
        <div class="space-y-3">
          <div
            v-for="item in scanTypeStats"
            :key="item.name"
            class="flex items-center justify-between"
          >
            <span class="text-white uppercase">{{ item.name }}</span>
            <div class="flex items-center space-x-2">
              <div class="w-32 bg-dark-700 rounded-full h-2">
                <div
                  class="bg-danger-500 h-2 rounded-full"
                  :style="{ width: `${(item.value / stats.total) * 100}%` }"
                ></div>
              </div>
              <span class="text-dark-400 font-mono w-8 text-right">{{ item.value }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 最近掃描 -->
    <div class="bg-dark-800 rounded-lg border border-dark-700 p-6">
      <h3 class="text-lg font-semibold text-white mb-4">最近掃描</h3>
      <div class="space-y-3">
        <div
          v-for="scan in recentScans"
          :key="scan.id"
          class="flex items-center justify-between p-3 bg-dark-700 rounded-lg"
        >
          <div class="flex-1">
            <div class="text-white font-medium">{{ scan.target_url }}</div>
            <div class="text-sm text-dark-400">
              {{ new Date(scan.created_at).toLocaleString('zh-TW') }}
            </div>
          </div>
          <span
            :class="[
              'px-2 py-1 text-xs font-semibold rounded',
              scan.status === 'completed' ? 'bg-success-900/30 text-success-400' :
              scan.status === 'failed' ? 'bg-danger-900/30 text-danger-400' :
              scan.status === 'running' ? 'bg-info-900/30 text-info-400' :
              'bg-warning-900/30 text-warning-400'
            ]"
          >
            {{ scan.status.toUpperCase() }}
          </span>
        </div>
        <div v-if="recentScans.length === 0" class="text-center text-dark-400 py-4">
          暫無掃描記錄
        </div>
      </div>
    </div>

    <!-- Collaboration Dialogs -->
    <ExportDialog v-model="showExportDialog" />
    <ImportDialog v-model="showImportDialog" />
  </div>
</template>
