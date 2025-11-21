<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { saveScanToDatabase } from '@/composables/useScanPersistence';

interface ScanTask {
  id: string;
  target_url: string;
  scan_type: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  started_at?: string;
  completed_at?: string;
  created_at: string;
}

const url = ref('https://wchung.tw');
const scanType = ref('full');
const isScanning = ref(false);
const currentTask = ref<ScanTask | null>(null);

const scanTypes = [
  { id: 'quick', label: '快速掃描', desc: '基本安全檢查' },
  { id: 'full', label: '完整掃描', desc: 'Headers + SSL + 漏洞' },
  { id: 'vulnerability', label: '漏洞掃描', desc: 'OWASP Top 10' },
];

const startScan = async () => {
  if (!url.value) {
    alert('請輸入目標 URL');
    return;
  }

  isScanning.value = true;

  try {
    const taskId = await invoke<string>('start_scan', {
      url: url.value,
      scanType: scanType.value,
    });

    console.log('🚀 掃描已啟動:', taskId);

    // 輪詢掃描狀態
    const pollInterval = setInterval(async () => {
      try {
        const task = await invoke<ScanTask>('get_scan_status', {
          taskId,
        });

        currentTask.value = task;

        if (task.status === 'completed' || task.status === 'failed') {
          clearInterval(pollInterval);
          isScanning.value = false;

          if (task.status === 'completed') {
            console.log('✅ 掃描完成，開始保存到資料庫...');

            // 自動保存到資料庫
            try {
              await saveScanToDatabase(taskId);
              console.log('✅ 掃描結果已保存到資料庫');
            } catch (dbError) {
              console.error('⚠️  保存到資料庫失敗（掃描結果仍在記憶體中）:', dbError);
            }
          } else {
            console.log('⚠️  掃描失敗:', task.status);
          }
        }
      } catch (err) {
        console.error('Failed to poll status:', err);
      }
    }, 1000);
  } catch (error) {
    console.error('Failed to start scan:', error);
    isScanning.value = false;
    alert('掃描啟動失敗: ' + error);
  }
};

const getStatusColor = (status: string) => {
  switch (status) {
    case 'running':
      return 'text-info-500';
    case 'completed':
      return 'text-success-500';
    case 'failed':
      return 'text-danger-500';
    default:
      return 'text-warning-500';
  }
};
</script>

<template>
  <div class="space-y-6">
    <!-- Scan Configuration Card -->
    <div class="bg-dark-800 rounded-lg border border-dark-700 p-6">
      <h2 class="text-2xl font-bold text-white mb-6 flex items-center">
        <svg class="w-6 h-6 mr-2 text-danger-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        啟動掃描
      </h2>

      <div class="space-y-4">
        <!-- URL Input -->
        <div>
          <label class="block text-sm font-medium text-dark-300 mb-2">
            目標 URL
          </label>
          <input
            v-model="url"
            type="url"
            placeholder="https://example.com"
            class="w-full px-4 py-3 bg-dark-700 border border-dark-600 rounded-lg text-white placeholder-dark-400 focus:outline-none focus:ring-2 focus:ring-danger-500"
          />
        </div>

        <!-- Scan Type Selection -->
        <div>
          <label class="block text-sm font-medium text-dark-300 mb-2">
            掃描類型
          </label>
          <div class="grid grid-cols-3 gap-3">
            <button
              v-for="type in scanTypes"
              :key="type.id"
              @click="scanType = type.id"
              :class="[
                'p-4 rounded-lg border-2 text-left transition-all',
                scanType === type.id
                  ? 'border-danger-500 bg-danger-900/20'
                  : 'border-dark-600 bg-dark-700 hover:border-dark-500'
              ]"
            >
              <div class="font-semibold text-white">{{ type.label }}</div>
              <div class="text-xs text-dark-400 mt-1">{{ type.desc }}</div>
            </button>
          </div>
        </div>

        <!-- Start Button -->
        <button
          @click="startScan"
          :disabled="isScanning"
          :class="[
            'w-full py-3 rounded-lg font-semibold transition-all flex items-center justify-center',
            isScanning
              ? 'bg-dark-600 text-dark-400 cursor-not-allowed'
              : 'bg-danger-600 hover:bg-danger-700 text-white'
          ]"
        >
          <svg v-if="isScanning" class="w-5 h-5 mr-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <svg v-else class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          {{ isScanning ? '掃描中...' : '開始掃描' }}
        </button>
      </div>
    </div>

    <!-- Current Task Status -->
    <div v-if="currentTask" class="bg-dark-800 rounded-lg border border-dark-700 p-6">
      <h3 class="text-lg font-semibold text-white mb-4 flex items-center">
        <svg :class="['w-5 h-5', getStatusColor(currentTask.status), currentTask.status === 'running' ? 'animate-spin' : '']" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path v-if="currentTask.status === 'running'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          <path v-else-if="currentTask.status === 'completed'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          <path v-else-if="currentTask.status === 'failed'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
          <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <span class="ml-2">掃描狀態</span>
      </h3>

      <div class="space-y-2">
        <div class="flex justify-between text-sm">
          <span class="text-dark-400">任務 ID:</span>
          <span class="text-white font-mono">{{ currentTask.id.slice(0, 8) }}...</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-dark-400">目標:</span>
          <span class="text-white">{{ currentTask.target_url }}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-dark-400">掃描類型:</span>
          <span class="text-white uppercase">{{ currentTask.scan_type }}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-dark-400">狀態:</span>
          <span :class="['font-semibold uppercase', getStatusColor(currentTask.status)]">
            {{ currentTask.status }}
          </span>
        </div>
        <div v-if="currentTask.started_at" class="flex justify-between text-sm">
          <span class="text-dark-400">開始時間:</span>
          <span class="text-white">{{ new Date(currentTask.started_at).toLocaleString('zh-TW') }}</span>
        </div>
      </div>

      <!-- Progress Animation -->
      <div v-if="currentTask.status === 'running'" class="mt-4">
        <div class="h-2 bg-dark-700 rounded-full overflow-hidden">
          <div class="h-full bg-gradient-to-r from-danger-600 to-danger-400 animate-pulse" style="width: 60%"></div>
        </div>
      </div>
    </div>

    <!-- Warning Card -->
    <div class="bg-warning-900/20 border border-warning-700 rounded-lg p-4">
      <div class="flex items-start">
        <svg class="w-5 h-5 text-warning-500 mr-3 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <div class="text-sm text-warning-200">
          <p class="font-semibold mb-1">重要提醒</p>
          <p>請確保您有權限掃描目標網站。未經授權的安全測試可能違反法律。</p>
        </div>
      </div>
    </div>
  </div>
</template>
