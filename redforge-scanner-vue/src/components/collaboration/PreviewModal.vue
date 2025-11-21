<template>
  <BaseModal
    v-model="isOpen"
    title="📋 資料預覽"
    size="xl"
    @close="handleClose"
  >
    <div v-if="data" class="space-y-6">
      <!-- Summary cards -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div class="p-4 bg-gray-800 border border-gray-700 rounded">
          <div class="text-2xl font-bold text-blue-400 font-mono">{{ data.scans.length }}</div>
          <div class="text-sm text-gray-500 mt-1">掃描任務</div>
        </div>
        <div class="p-4 bg-gray-800 border border-gray-700 rounded">
          <div class="text-2xl font-bold text-red-400 font-mono">{{ data.findings.length }}</div>
          <div class="text-sm text-gray-500 mt-1">漏洞</div>
        </div>
        <div class="p-4 bg-gray-800 border border-gray-700 rounded">
          <div class="text-2xl font-bold text-yellow-400 font-mono">
            {{ data.annotations?.length || 0 }}
          </div>
          <div class="text-sm text-gray-500 mt-1">註記</div>
        </div>
        <div class="p-4 bg-gray-800 border border-gray-700 rounded">
          <div class="text-2xl font-bold text-green-400 font-mono">
            {{ data.assets?.length || 0 }}
          </div>
          <div class="text-sm text-gray-500 mt-1">資產</div>
        </div>
      </div>

      <!-- Tabs -->
      <div class="border-b border-gray-800">
        <nav class="flex gap-1">
          <button
            v-for="tab in tabs"
            :key="tab.value"
            :class="activeTab === tab.value ? 'border-blue-500 text-blue-400' : 'border-transparent text-gray-500 hover:text-gray-400'"
            class="px-4 py-2 border-b-2 font-mono text-sm transition-colors"
            @click="activeTab = tab.value"
          >
            {{ tab.label }}
            <span class="ml-2 text-xs">
              ({{ getTabCount(tab.value) }})
            </span>
          </button>
        </nav>
      </div>

      <!-- Tab content -->
      <div class="max-h-96 overflow-y-auto">
        <!-- Scans -->
        <div v-if="activeTab === 'scans'" class="space-y-3">
          <div
            v-for="scan in data.scans"
            :key="scan.id"
            class="p-4 bg-gray-800 border border-gray-700 rounded hover:border-gray-600 transition-colors"
          >
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <h4 class="font-mono font-medium text-gray-300">{{ scan.name }}</h4>
                <p class="text-sm text-gray-500 mt-1">{{ scan.target }}</p>
              </div>
              <span
                :class="getStatusClass(scan.status)"
                class="px-2 py-1 text-xs font-mono rounded"
              >
                {{ scan.status }}
              </span>
            </div>
            <div class="mt-3 flex gap-4 text-xs text-gray-500 font-mono">
              <span>建立: {{ formatDate(scan.created_at) }}</span>
              <span v-if="scan.completed_at">完成: {{ formatDate(scan.completed_at) }}</span>
            </div>
          </div>
        </div>

        <!-- Findings -->
        <div v-if="activeTab === 'findings'" class="space-y-3">
          <div
            v-for="finding in sortedFindings"
            :key="finding.id"
            class="p-4 bg-gray-800 border border-gray-700 rounded hover:border-gray-600 transition-colors"
          >
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span
                    :class="getSeverityClass(finding.severity)"
                    class="px-2 py-0.5 text-xs font-mono rounded font-semibold"
                  >
                    {{ finding.severity.toUpperCase() }}
                  </span>
                  <h4 class="font-mono font-medium text-gray-300">{{ finding.title }}</h4>
                </div>
                <p class="text-sm text-gray-500 mt-2">{{ finding.description }}</p>
                <div v-if="finding.affected_url" class="text-xs text-gray-600 mt-2 font-mono">
                  🔗 {{ finding.affected_url }}
                </div>
              </div>
            </div>
            <div class="mt-3 flex gap-4 text-xs text-gray-500 font-mono">
              <span>發現時間: {{ formatDate(finding.discovered_at) }}</span>
              <span v-if="finding.cvss_score">CVSS: {{ finding.cvss_score }}</span>
              <span v-if="finding.cve_id">{{ finding.cve_id }}</span>
            </div>
          </div>
        </div>

        <!-- Annotations -->
        <div v-if="activeTab === 'annotations'" class="space-y-3">
          <div v-if="!data.annotations || data.annotations.length === 0" class="text-center text-gray-500 py-8">
            無註記資料
          </div>
          <div
            v-for="annotation in data.annotations"
            :key="annotation.id"
            class="p-4 bg-gray-800 border border-gray-700 rounded hover:border-gray-600 transition-colors"
          >
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <div class="flex items-center gap-2 mb-2">
                  <span class="text-sm font-mono text-gray-400">{{ annotation.author }}</span>
                  <span v-if="annotation.is_false_positive" class="px-2 py-0.5 text-xs bg-yellow-900/50 text-yellow-400 rounded">
                    False Positive
                  </span>
                  <span v-if="annotation.priority" :class="getPriorityClass(annotation.priority)" class="px-2 py-0.5 text-xs rounded">
                    {{ annotation.priority }}
                  </span>
                </div>
                <p class="text-sm text-gray-300">{{ annotation.content }}</p>
              </div>
            </div>
            <div class="mt-2 text-xs text-gray-500 font-mono">
              {{ formatDate(annotation.created_at) }}
            </div>
          </div>
        </div>

        <!-- Assets -->
        <div v-if="activeTab === 'assets'" class="space-y-3">
          <div v-if="!data.assets || data.assets.length === 0" class="text-center text-gray-500 py-8">
            無資產資料
          </div>
          <div
            v-for="asset in data.assets"
            :key="asset.id"
            class="p-4 bg-gray-800 border border-gray-700 rounded hover:border-gray-600 transition-colors"
          >
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <h4 class="font-mono font-medium text-gray-300">{{ asset.hostname }}</h4>
                <p v-if="asset.ip_address" class="text-sm text-gray-500 mt-1">
                  IP: {{ asset.ip_address }}
                </p>
                <div v-if="asset.ports && asset.ports.length > 0" class="mt-2">
                  <span class="text-xs text-gray-500">開放端口:</span>
                  <div class="flex flex-wrap gap-1 mt-1">
                    <span
                      v-for="port in asset.ports"
                      :key="port"
                      class="px-2 py-0.5 text-xs bg-gray-700 text-gray-300 rounded font-mono"
                    >
                      {{ port }}
                    </span>
                  </div>
                </div>
                <div v-if="asset.technologies && asset.technologies.length > 0" class="mt-2">
                  <span class="text-xs text-gray-500">技術棧:</span>
                  <div class="flex flex-wrap gap-1 mt-1">
                    <span
                      v-for="tech in asset.technologies"
                      :key="tech"
                      class="px-2 py-0.5 text-xs bg-blue-900/50 text-blue-400 rounded font-mono"
                    >
                      {{ tech }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
            <div class="mt-3 text-xs text-gray-500 font-mono">
              發現時間: {{ formatDate(asset.discovered_at) }}
            </div>
          </div>
        </div>

        <!-- Metadata -->
        <div v-if="activeTab === 'metadata'" class="space-y-4">
          <div class="p-4 bg-gray-800 border border-gray-700 rounded">
            <h4 class="font-mono font-medium text-gray-300 mb-3">檔案資訊</h4>
            <dl class="space-y-2 text-sm font-mono">
              <div class="flex justify-between">
                <dt class="text-gray-500">版本:</dt>
                <dd class="text-gray-300">{{ data.metadata.version }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-gray-500">格式:</dt>
                <dd class="text-gray-300">{{ data.metadata.format }}</dd>
              </div>
              <div v-if="data.metadata.encryption" class="flex justify-between">
                <dt class="text-gray-500">加密:</dt>
                <dd class="text-gray-300">{{ data.metadata.encryption }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-gray-500">匯出者:</dt>
                <dd class="text-gray-300">{{ data.metadata.exported_by }}</dd>
              </div>
              <div v-if="data.metadata.team_id" class="flex justify-between">
                <dt class="text-gray-500">團隊 ID:</dt>
                <dd class="text-gray-300">{{ data.metadata.team_id }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-gray-500">匯出時間:</dt>
                <dd class="text-gray-300">{{ formatDate(data.metadata.exported_at) }}</dd>
              </div>
            </dl>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="ghost" @click="handleClose">
        關閉
      </BaseButton>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { ExportData } from '@/types/offline-collaboration';
import BaseModal from '../ui/BaseModal.vue';
import BaseButton from '../ui/BaseButton.vue';

interface Props {
  modelValue: boolean;
  data: ExportData | null;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const activeTab = ref('scans');

const tabs = [
  { label: '掃描任務', value: 'scans' },
  { label: '漏洞', value: 'findings' },
  { label: '註記', value: 'annotations' },
  { label: '資產', value: 'assets' },
  { label: '檔案資訊', value: 'metadata' },
];

const sortedFindings = computed(() => {
  if (!props.data?.findings) return [];

  const severityOrder: Record<string, number> = { critical: 0, high: 1, medium: 2, low: 3, info: 4 };

  return [...props.data.findings].sort((a, b) => {
    return (severityOrder[a.severity] || 999) - (severityOrder[b.severity] || 999);
  });
});

const getTabCount = (tab: string) => {
  if (!props.data) return 0;

  switch (tab) {
    case 'scans':
      return props.data.scans.length;
    case 'findings':
      return props.data.findings.length;
    case 'annotations':
      return props.data.annotations?.length || 0;
    case 'assets':
      return props.data.assets?.length || 0;
    case 'metadata':
      return 1;
    default:
      return 0;
  }
};

const getStatusClass = (status: string) => {
  const classMap: Record<string, string> = {
    pending: 'bg-gray-700 text-gray-300',
    running: 'bg-blue-900/50 text-blue-400',
    completed: 'bg-green-900/50 text-green-400',
    failed: 'bg-red-900/50 text-red-400',
  };
  return classMap[status] || 'bg-gray-700 text-gray-300';
};

const getSeverityClass = (severity: string) => {
  const classMap: Record<string, string> = {
    critical: 'bg-red-900/50 text-red-400',
    high: 'bg-orange-900/50 text-orange-400',
    medium: 'bg-yellow-900/50 text-yellow-400',
    low: 'bg-blue-900/50 text-blue-400',
    info: 'bg-gray-700 text-gray-400',
  };
  return classMap[severity] || 'bg-gray-700 text-gray-400';
};

const getPriorityClass = (priority: string) => {
  const classMap: Record<string, string> = {
    high: 'bg-red-900/50 text-red-400',
    medium: 'bg-yellow-900/50 text-yellow-400',
    low: 'bg-blue-900/50 text-blue-400',
  };
  return classMap[priority] || 'bg-gray-700 text-gray-400';
};

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-TW');
};

const handleClose = () => {
  isOpen.value = false;
};
</script>
