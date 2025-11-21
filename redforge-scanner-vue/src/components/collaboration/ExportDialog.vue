<template>
  <BaseModal
    v-model="isOpen"
    title="🔐 匯出掃描資料"
    size="lg"
    @close="handleClose"
  >
    <div class="space-y-6">
      <!-- Progress indicator -->
      <div v-if="exportStore.isExporting" class="space-y-3">
        <div class="flex items-center justify-between text-sm font-mono">
          <span class="text-gray-400">匯出進度</span>
          <span class="text-blue-400">{{ exportStore.progress }}%</span>
        </div>
        <div class="w-full bg-gray-800 rounded-full h-2 overflow-hidden">
          <div
            class="bg-blue-600 h-full transition-all duration-300"
            :style="{ width: `${exportStore.progress}%` }"
          ></div>
        </div>
      </div>

      <!-- Error message -->
      <div
        v-if="exportStore.error"
        class="p-4 bg-red-900/30 border border-red-700 rounded text-red-400 font-mono text-sm"
      >
        ❌ {{ exportStore.error }}
      </div>

      <!-- Success message -->
      <div
        v-if="exportSuccess"
        class="p-4 bg-green-900/30 border border-green-700 rounded text-green-400 font-mono text-sm"
      >
        ✅ 匯出成功！檔案已儲存至: {{ exportStore.lastExportPath }}
      </div>

      <!-- Export form -->
      <div v-if="!exportStore.isExporting && !exportSuccess" class="space-y-6">
        <!-- Exporter name -->
        <BaseInput
          v-model="formData.exportedBy"
          label="匯出者名稱"
          placeholder="例如: alice"
          required
          :error="errors.exportedBy"
        />

        <!-- Team ID (optional) -->
        <BaseInput
          v-model="formData.teamId"
          label="團隊 ID (選填)"
          placeholder="例如: devcore-team-alpha"
          helper="用於識別團隊,選填"
        />

        <!-- Encryption toggle -->
        <div class="flex items-center justify-between p-4 bg-gray-800 border border-gray-700 rounded">
          <div>
            <label class="font-mono font-medium text-gray-300">加密資料</label>
            <p class="text-sm text-gray-500 mt-1">使用 AES-256-GCM 加密保護敏感資料</p>
          </div>
          <button
            type="button"
            :class="formData.encrypt ? 'bg-blue-600' : 'bg-gray-700'"
            class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors"
            @click="formData.encrypt = !formData.encrypt"
          >
            <span
              :class="formData.encrypt ? 'translate-x-6' : 'translate-x-1'"
              class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
            />
          </button>
        </div>

        <!-- Passphrase (if encryption enabled) -->
        <div v-if="formData.encrypt" class="space-y-4">
          <BaseInput
            v-model="formData.passphrase"
            type="password"
            label="加密密碼"
            placeholder="至少 8 個字元"
            required
            :error="errors.passphrase"
          >
            <template #icon>
              <svg class="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
            </template>
          </BaseInput>

          <BaseInput
            v-model="formData.passphraseConfirm"
            type="password"
            label="確認密碼"
            placeholder="再次輸入密碼"
            required
            :error="errors.passphraseConfirm"
          />

          <!-- Password strength indicator -->
          <div v-if="passwordStrength" class="space-y-2">
            <div class="flex items-center justify-between text-sm font-mono">
              <span class="text-gray-400">密碼強度</span>
              <span :class="strengthColorClass">{{ strengthText }}</span>
            </div>
            <div class="flex gap-1">
              <div
                v-for="i in 4"
                :key="i"
                class="h-1 flex-1 rounded-full transition-colors"
                :class="i <= strengthLevel ? strengthColorBg : 'bg-gray-800'"
              ></div>
            </div>
            <ul v-if="passwordStrength.suggestions.length > 0" class="text-xs text-gray-500 space-y-1 mt-2">
              <li v-for="(suggestion, index) in passwordStrength.suggestions" :key="index">
                • {{ suggestion }}
              </li>
            </ul>
          </div>
        </div>

        <!-- Export options -->
        <div class="space-y-3">
          <h4 class="font-mono font-medium text-gray-300 text-sm">匯出選項</h4>

          <label class="flex items-start gap-3 p-3 bg-gray-800 border border-gray-700 rounded cursor-pointer hover:border-gray-600 transition-colors">
            <input
              v-model="formData.includeAnnotations"
              type="checkbox"
              class="mt-1 w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
            />
            <div>
              <span class="font-mono text-sm text-gray-300">包含註記</span>
              <p class="text-xs text-gray-500 mt-0.5">匯出漏洞註記和標註</p>
            </div>
          </label>

          <label class="flex items-start gap-3 p-3 bg-gray-800 border border-gray-700 rounded cursor-pointer hover:border-gray-600 transition-colors">
            <input
              v-model="formData.includeAssets"
              type="checkbox"
              class="mt-1 w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
            />
            <div>
              <span class="font-mono text-sm text-gray-300">包含資產資訊</span>
              <p class="text-xs text-gray-500 mt-0.5">匯出主機、IP、服務等資產資料</p>
            </div>
          </label>
        </div>

        <!-- Incremental export option -->
        <div v-if="exportStore.canExportIncremental" class="p-4 bg-blue-900/20 border border-blue-800 rounded">
          <label class="flex items-start gap-3 cursor-pointer">
            <input
              v-model="formData.incremental"
              type="checkbox"
              class="mt-1 w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
            />
            <div>
              <span class="font-mono text-sm text-blue-400">增量匯出</span>
              <p class="text-xs text-gray-500 mt-0.5">
                僅匯出自上次匯出後的新資料 ({{ lastExportTime }})
              </p>
            </div>
          </label>
        </div>
      </div>
    </div>

    <!-- Footer actions -->
    <template #footer>
      <BaseButton
        v-if="!exportStore.isExporting && !exportSuccess"
        variant="ghost"
        @click="handleClose"
      >
        取消
      </BaseButton>
      <BaseButton
        v-if="!exportStore.isExporting && !exportSuccess"
        variant="primary"
        :disabled="!isFormValid"
        @click="handleExport"
      >
        開始匯出
      </BaseButton>
      <BaseButton
        v-if="exportSuccess"
        variant="success"
        @click="handleClose"
      >
        完成
      </BaseButton>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useExportStore } from '@/stores/export';
import { encryptionService } from '@/services/encryption';
import BaseModal from '../ui/BaseModal.vue';
import BaseButton from '../ui/BaseButton.vue';
import BaseInput from '../ui/BaseInput.vue';

interface Props {
  modelValue: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const exportStore = useExportStore();

const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

// Form data
const formData = ref({
  exportedBy: '',
  teamId: '',
  encrypt: true,
  passphrase: '',
  passphraseConfirm: '',
  includeAnnotations: true,
  includeAssets: true,
  incremental: false,
});

// Validation errors
const errors = ref<Record<string, string>>({});

// Success state
const exportSuccess = ref(false);

// Password strength
const passwordStrength = computed(() => {
  if (!formData.value.passphrase) return null;
  return encryptionService.validatePassphrase(formData.value.passphrase);
});

const strengthLevel = computed(() => {
  if (!passwordStrength.value) return 0;
  const strengthMap: Record<string, number> = { weak: 1, medium: 2, strong: 4 };
  return strengthMap[passwordStrength.value.strength] || 0;
});

const strengthText = computed(() => {
  if (!passwordStrength.value) return '';
  const textMap: Record<string, string> = { weak: '弱', medium: '中等', strong: '強' };
  return textMap[passwordStrength.value.strength] || '';
});

const strengthColorClass = computed(() => {
  if (!passwordStrength.value) return '';
  const colorMap: Record<string, string> = {
    weak: 'text-red-400',
    medium: 'text-yellow-400',
    strong: 'text-green-400',
  };
  return colorMap[passwordStrength.value.strength] || '';
});

const strengthColorBg = computed(() => {
  if (!passwordStrength.value) return '';
  const colorMap: Record<string, string> = {
    weak: 'bg-red-500',
    medium: 'bg-yellow-500',
    strong: 'bg-green-500',
  };
  return colorMap[passwordStrength.value.strength] || '';
});

const lastExportTime = computed(() => {
  if (!exportStore.lastExportInfo.timestamp) return '';
  return new Date(exportStore.lastExportInfo.timestamp).toLocaleString('zh-TW');
});

const isFormValid = computed(() => {
  return (
    formData.value.exportedBy.trim() !== '' &&
    (!formData.value.encrypt ||
      (formData.value.passphrase.length >= 8 &&
        formData.value.passphrase === formData.value.passphraseConfirm))
  );
});

// Validate form
const validateForm = (): boolean => {
  errors.value = {};

  if (!formData.value.exportedBy.trim()) {
    errors.value.exportedBy = '請輸入匯出者名稱';
  }

  if (formData.value.encrypt) {
    if (!formData.value.passphrase) {
      errors.value.passphrase = '請輸入加密密碼';
    } else if (formData.value.passphrase.length < 8) {
      errors.value.passphrase = '密碼至少需要 8 個字元';
    }

    if (formData.value.passphrase !== formData.value.passphraseConfirm) {
      errors.value.passphraseConfirm = '密碼不一致';
    }

    if (passwordStrength.value && !passwordStrength.value.valid) {
      errors.value.passphrase = '密碼強度不足';
    }
  }

  return Object.keys(errors.value).length === 0;
};

// Handle export
const handleExport = async () => {
  if (!validateForm()) return;

  try {
    exportSuccess.value = false;

    if (formData.value.incremental) {
      await exportStore.exportIncremental({
        exportedBy: formData.value.exportedBy,
        teamId: formData.value.teamId || undefined,
        encrypt: formData.value.encrypt,
        passphrase: formData.value.encrypt ? formData.value.passphrase : undefined,
        includeAnnotations: formData.value.includeAnnotations,
        includeAssets: formData.value.includeAssets,
      });
    } else {
      await exportStore.exportData({
        exportedBy: formData.value.exportedBy,
        teamId: formData.value.teamId || undefined,
        encrypt: formData.value.encrypt,
        passphrase: formData.value.encrypt ? formData.value.passphrase : undefined,
        includeAnnotations: formData.value.includeAnnotations,
        includeAssets: formData.value.includeAssets,
      });
    }

    exportSuccess.value = true;
  } catch (error) {
    console.error('Export failed:', error);
  }
};

// Handle close
const handleClose = () => {
  if (!exportStore.isExporting) {
    isOpen.value = false;
    setTimeout(() => {
      exportSuccess.value = false;
      exportStore.reset();
      resetForm();
    }, 300);
  }
};

// Reset form
const resetForm = () => {
  formData.value = {
    exportedBy: '',
    teamId: '',
    encrypt: true,
    passphrase: '',
    passphraseConfirm: '',
    includeAnnotations: true,
    includeAssets: true,
    incremental: false,
  };
  errors.value = {};
};

// Watch for modal open to reset state
watch(isOpen, (value) => {
  if (value) {
    exportSuccess.value = false;
    exportStore.reset();
  }
});
</script>
