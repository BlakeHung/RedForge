<template>
  <div class="w-full">
    <!-- Label -->
    <label
      v-if="label"
      :for="inputId"
      class="block text-sm font-mono font-medium text-gray-300 mb-2"
    >
      {{ label }}
      <span v-if="required" class="text-red-500">*</span>
    </label>

    <!-- Input wrapper -->
    <div class="relative">
      <input
        :id="inputId"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :required="required"
        :class="inputClasses"
        @input="handleInput"
        @blur="emit('blur')"
        @focus="emit('focus')"
      />

      <!-- Icon (optional) -->
      <div v-if="$slots.icon" class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
        <slot name="icon"></slot>
      </div>
    </div>

    <!-- Helper text or error -->
    <p
      v-if="error || helper"
      :class="error ? 'text-red-400' : 'text-gray-500'"
      class="mt-1.5 text-sm font-mono"
    >
      {{ error || helper }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  modelValue: string | number;
  type?: 'text' | 'password' | 'email' | 'number' | 'tel' | 'url';
  label?: string;
  placeholder?: string;
  helper?: string;
  error?: string;
  disabled?: boolean;
  required?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  disabled: false,
  required: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: string | number];
  blur: [];
  focus: [];
}>();

const inputId = computed(() => `input-${Math.random().toString(36).substr(2, 9)}`);

const inputClasses = computed(() => {
  const baseClasses = 'w-full px-4 py-2 font-mono text-gray-100 bg-gray-800 border rounded transition-colors focus:outline-none focus:ring-2';

  const stateClasses = props.error
    ? 'border-red-500 focus:border-red-400 focus:ring-red-500/50'
    : 'border-gray-700 focus:border-blue-500 focus:ring-blue-500/50';

  const disabledClasses = props.disabled
    ? 'opacity-50 cursor-not-allowed'
    : 'hover:border-gray-600';

  return `${baseClasses} ${stateClasses} ${disabledClasses}`;
});

const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement;
  const value = props.type === 'number' ? Number(target.value) : target.value;
  emit('update:modelValue', value);
};
</script>
