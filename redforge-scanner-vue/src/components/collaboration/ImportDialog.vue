<template>
  <BaseModal
    v-model="isOpen"
    title="📥 匯入掃描資料"
    size="lg"
    @close="handleClose"
  >
    <div class="space-y-6">
      <!-- Progress indicator -->
      <div v-if="importStore.isImporting" class="space-y-3">
        <div class="flex items-center justify-between text-sm font-mono">
          <span class="text-gray-400">匯入進度</span>
          <span class="text-blue-400">{{ importStore.progress }}%</span>
        </div>
        <div class="w-full bg-gray-800 rounded-full h-2 overflow-hidden">
          <div
            class="bg-blue-600 h-full transition-all duration-300"
            :style="{ width: `${importStore.progress}%` }"
          ></div>
        </div>
      </div>

      <!-- Error message -->
      <div
        v-if="importStore.error"
        class="p-4 bg-red-900/30 border border-red-700 rounded text-red-400 font-mono text-sm"
      >
        ❌ {{ importStore.error }}
      </div>

      <!-- Success message -->
      <div
        v-if="importSuccess && importStore.lastImportStats"
        class="p-4 bg-green-900/30 border border-green-700 rounded space-y-2"
      >
        <div class="flex items-center gap-2 text-green-400 font-mono text-sm">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span>匯入成功！</span>
        </div>
        <div class="text-sm text-gray-400 font-mono space-y-1">
          <div>✅ 已匯入: {{ importStore.lastImportStats.imported }} 筆資料</div>
          <div v-if="importStore.lastImportStats.skipped > 0">
            ⏭️ 已跳過: {{ importStore.lastImportStats.skipped }} 筆重複資料
          </div>
          <div v-if="importStore.lastImportStats.errors > 0" class="text-red-400">
            ❌ 錯誤: {{ importStore.lastImportStats.errors }} 筆
          </div>
        </div>
      </div>

      <!-- Import form -->
      <div v-if="!importStore.isImporting && !importSuccess" class="space-y-6">
        <!-- File selection -->
        <div class="space-y-3">
          <label class="block text-sm font-mono font-medium text-gray-300">
            選擇檔案 <span class="text-red-500">*</span>
          </label>
          <div
            class="border-2 border-dashed rounded-lg p-8 text-center transition-colors"
            :class="selectedFile ? 'border-blue-600 bg-blue-900/10' : 'border-gray-700 hover:border-gray-600'"
          >
            <div v-if="!selectedFile" class="space-y-3">
              <svg class="w-12 h-12 mx-auto text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
              </svg>
              <div>
                <BaseButton variant="ghost" size="sm" @click="selectFile">
                  選擇檔案
                </BaseButton>
                <p class="text-sm text-gray-500 mt-2 font-mono">
                  支援 .md 或 .md.enc 格式
                </p>
              </div>
            </div>
            <div v-else class="space-y-3">
              <svg class="w-12 h-12 mx-auto text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <div>
                <p class="font-mono text-sm text-gray-300">{{ selectedFile }}</p>
                <BaseButton variant="ghost" size="sm" class="mt-2" @click="selectFile">
                  更換檔案
                </BaseButton>
              </div>
            </div>
          </div>
        </div>

        <!-- File verification result -->
        <div v-if="verificationResult" class="space-y-3">
          <div
            v-if="verificationResult.valid"
            class="p-4 bg-green-900/20 border border-green-800 rounded"
          >
            <div class="flex items-center gap-2 text-green-400 font-mono text-sm mb-3">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span>檔案驗證通過</span>
            </div>
            <div v-if="verificationResult.metadata" class="text-xs text-gray-400 space-y-1 font-mono">
              <div>版本: {{ verificationResult.metadata.version }}</div>
              <div>格式: {{ verificationResult.metadata.format }}</div>
              <div v-if="verificationResult.metadata.encryption">
                加密: {{ verificationResult.metadata.encryption }}
              </div>
              <div>匯出者: {{ verificationResult.metadata.exported_by }}</div>
              <div>匯出時間: {{ formatDate(verificationResult.metadata.exported_at) }}</div>
            </div>
          </div>
          <div
            v-else
            class="p-4 bg-red-900/20 border border-red-800 rounded"
          >
            <div class="flex items-center gap-2 text-red-400 font-mono text-sm mb-2">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span>檔案驗證失敗</span>
            </div>
            <ul class="text-xs text-red-400 space-y-1">
              <li v-for="(error, index) in verificationResult.errors" :key="index">
                • {{ error }}
              </li>
            </ul>
          </div>
        </div>

        <!-- Passphrase input (if encrypted) -->
        <div v-if="requiresPassphrase" class="space-y-4">
          <BaseInput
            v-model="formData.passphrase"
            type="password"
            label="解密密碼"
            placeholder="輸入加密密碼"
            required
            :error="errors.passphrase"
          >
            <template #icon>
              <svg class="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
            </template>
          </BaseInput>

          <BaseButton
            v-if="!previewLoaded"
            variant="secondary"
            size="sm"
            :disabled="!formData.passphrase"
            :loading="previewLoading"
            @click="handlePreview"
          >
            預覽資料
          </BaseButton>
        </div>

        <!-- Preview data -->
        <div v-if="previewLoaded && importStore.previewData" class="p-4 bg-gray-800 border border-gray-700 rounded space-y-3">
          <h4 class="font-mono font-medium text-gray-300 text-sm">資料預覽</h4>
          <div class="grid grid-cols-2 gap-4 text-sm font-mono">
            <div>
              <span class="text-gray-500">掃描任務:</span>
              <span class="text-gray-300 ml-2">{{ importStore.previewData.scans.length }}</span>
            </div>
            <div>
              <span class="text-gray-500">漏洞:</span>
              <span class="text-gray-300 ml-2">{{ importStore.previewData.findings.length }}</span>
            </div>
            <div v-if="importStore.previewData.annotations">
              <span class="text-gray-500">註記:</span>
              <span class="text-gray-300 ml-2">{{ importStore.previewData.annotations.length }}</span>
            </div>
            <div v-if="importStore.previewData.assets">
              <span class="text-gray-500">資產:</span>
              <span class="text-gray-300 ml-2">{{ importStore.previewData.assets.length }}</span>
            </div>
          </div>
          <BaseButton variant="ghost" size="sm" @click="openPreviewModal">
            查看詳細資料
          </BaseButton>
        </div>

        <!-- Import options -->
        <div class="space-y-3">
          <h4 class="font-mono font-medium text-gray-300 text-sm">匯入選項</h4>

          <label class="flex items-start gap-3 p-3 bg-gray-800 border border-gray-700 rounded cursor-pointer hover:border-gray-600 transition-colors">
            <input
              v-model="formData.skipDuplicates"
              type="checkbox"
              class="mt-1 w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
            />
            <div>
              <span class="font-mono text-sm text-gray-300">跳過重複資料</span>
              <p class="text-xs text-gray-500 mt-0.5">自動檢測並跳過已存在的資料</p>
            </div>
          </label>

          <div class="space-y-2">
            <label class="block text-sm font-mono font-medium text-gray-300">
              合併策略
            </label>
            <div class="space-y-2">
              <label
                v-for="strategy in mergeStrategies"
                :key="strategy.value"
                class="flex items-start gap-3 p-3 bg-gray-800 border border-gray-700 rounded cursor-pointer hover:border-gray-600 transition-colors"
                :class="formData.mergeStrategy === strategy.value ? 'border-blue-600 bg-blue-900/10' : ''"
              >
                <input
                  v-model="formData.mergeStrategy"
                  type="radio"
                  :value="strategy.value"
                  class="mt-1 w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 focus:ring-blue-500"
                />
                <div>
                  <span class="font-mono text-sm text-gray-300">{{ strategy.label }}</span>
                  <p class="text-xs text-gray-500 mt-0.5">{{ strategy.description }}</p>
                </div>
              </label>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer actions -->
    <template #footer>
      <BaseButton
        v-if="!importStore.isImporting && !importSuccess"
        variant="ghost"
        @click="handleClose"
      >
        取消
      </BaseButton>
      <BaseButton
        v-if="!importStore.isImporting && !importSuccess"
        variant="primary"
        :disabled="!canImport"
        @click="handleImport"
      >
        開始匯入
      </BaseButton>
      <BaseButton
        v-if="importSuccess"
        variant="success"
        @click="handleClose"
      >
        完成
      </BaseButton>
    </template>
  </BaseModal>

  <!-- Preview Modal -->
  <PreviewModal
    v-model="showPreviewModal"
    :data="importStore.previewData"
  />
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useImportStore } from '@/stores/import';
import BaseModal from '../ui/BaseModal.vue';
import BaseButton from '../ui/BaseButton.vue';
import BaseInput from '../ui/BaseInput.vue';
import PreviewModal from './PreviewModal.vue';

interface Props {
  modelValue: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const importStore = useImportStore();

const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

// Form data
const formData = ref({
  passphrase: '',
  skipDuplicates: true,
  mergeStrategy: 'skip' as 'skip' | 'overwrite' | 'merge',
});

// State
const selectedFile = ref('');
const selectedFilePath = ref('');
const verificationResult = ref<any>(null);
const previewLoading = ref(false);
const previewLoaded = ref(false);
const importSuccess = ref(false);
const errors = ref<Record<string, string>>({});
const showPreviewModal = ref(false);

// Merge strategies
const mergeStrategies = [
  {
    value: 'skip',
    label: '跳過 (Skip)',
    description: '遇到重複資料時跳過,保留現有資料',
  },
  {
    value: 'overwrite',
    label: '覆蓋 (Overwrite)',
    description: '遇到重複資料時覆蓋,使用新資料',
  },
  {
    value: 'merge',
    label: '合併 (Merge)',
    description: '智慧合併重複資料,保留最完整的資訊',
  },
];

const requiresPassphrase = computed(() => {
  return verificationResult.value?.metadata?.encryption;
});

const canImport = computed(() => {
  return (
    selectedFile.value &&
    verificationResult.value?.valid &&
    (!requiresPassphrase.value || (formData.value.passphrase && previewLoaded.value))
  );
});

// Select file
const selectFile = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Markdown Files',
          extensions: ['md', 'md.enc'],
        },
      ],
    });

    if (!selected || typeof selected !== 'string') {
      return; // User cancelled
    }

    // Save the selected file path
    selectedFilePath.value = selected;

    // Extract filename from path
    const pathParts = selected.split(/[\\/]/);
    selectedFile.value = pathParts[pathParts.length - 1];

    // Verify file
    await verifyFile();
  } catch (error) {
    console.error('Failed to select file:', error);
    importStore.error = error instanceof Error ? error.message : 'Failed to select file';
  }
};

// Verify file
const verifyFile = async () => {
  try {
    verificationResult.value = await importStore.verifyFile(selectedFilePath.value);
  } catch (error) {
    console.error('File verification failed:', error);
  }
};

// Preview data
const handlePreview = async () => {
  if (!formData.value.passphrase) {
    errors.value.passphrase = '請輸入密碼';
    return;
  }

  try {
    previewLoading.value = true;
    errors.value = {};

    await importStore.previewImport(selectedFilePath.value, formData.value.passphrase);
    previewLoaded.value = true;
  } catch (error) {
    errors.value.passphrase = '密碼錯誤或檔案已損壞';
  } finally {
    previewLoading.value = false;
  }
};

// Handle import
const handleImport = async () => {
  try {
    importSuccess.value = false;

    await importStore.importData(
      {
        passphrase: formData.value.passphrase || undefined,
        skipDuplicates: formData.value.skipDuplicates,
        mergeStrategy: formData.value.mergeStrategy,
      },
      selectedFilePath.value
    );

    importSuccess.value = true;
  } catch (error) {
    console.error('Import failed:', error);
  }
};

// Open preview modal
const openPreviewModal = () => {
  showPreviewModal.value = true;
};

// Handle close
const handleClose = () => {
  if (!importStore.isImporting) {
    isOpen.value = false;
    setTimeout(() => {
      resetForm();
      importStore.reset();
    }, 300);
  }
};

// Reset form
const resetForm = () => {
  selectedFile.value = '';
  selectedFilePath.value = '';
  verificationResult.value = null;
  previewLoaded.value = false;
  importSuccess.value = false;
  formData.value = {
    passphrase: '',
    skipDuplicates: true,
    mergeStrategy: 'skip',
  };
  errors.value = {};
};

// Format date
const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-TW');
};

// Watch for modal open
watch(isOpen, (value) => {
  if (value) {
    importSuccess.value = false;
    importStore.reset();
  }
});
</script>
