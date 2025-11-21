<script setup lang="ts">
import { ref } from 'vue';
import Scanner from './components/Scanner.vue';
import ScanHistory from './components/ScanHistory.vue';
import Dashboard from './components/Dashboard.vue';

type TabType = 'scanner' | 'history' | 'dashboard';

const activeTab = ref<TabType>('scanner');

const tabs = [
  { id: 'scanner' as TabType, label: '掃描器', icon: '🔍' },
  { id: 'history' as TabType, label: '歷史記錄', icon: '📋' },
  { id: 'dashboard' as TabType, label: '儀表板', icon: '📊' },
];
</script>

<template>
  <div class="min-h-screen bg-dark-900 flex flex-col">
    <!-- Header -->
    <header class="bg-dark-800 border-b border-dark-700 sticky top-0 z-50">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16">
          <div class="flex items-center space-x-3">
            <svg class="w-8 h-8 text-danger-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
            <div>
              <h1 class="text-xl font-bold text-white">RedForge Scanner</h1>
              <p class="text-xs text-dark-400">紅隊安全掃描系統 (Vue3)</p>
            </div>
          </div>

          <nav class="flex space-x-1">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              @click="activeTab = tab.id"
              :class="[
                'px-4 py-2 rounded-lg font-medium transition-colors',
                activeTab === tab.id
                  ? 'bg-danger-600 text-white'
                  : 'text-dark-300 hover:bg-dark-700 hover:text-white'
              ]"
            >
              <span class="mr-2">{{ tab.icon }}</span>
              {{ tab.label }}
            </button>
          </nav>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <Scanner v-if="activeTab === 'scanner'" />
      <ScanHistory v-if="activeTab === 'history'" />
      <Dashboard v-if="activeTab === 'dashboard'" />
    </main>

    <!-- Footer -->
    <footer class="bg-dark-800 border-t border-dark-700 py-4">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center text-dark-400 text-sm">
        <p>⚠️ 僅用於授權測試 | Target: wchung.tw</p>
      </div>
    </footer>
  </div>
</template>
