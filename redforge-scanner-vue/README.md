# RedForge Scanner (Vue)

> RedForge 的主要前端應用 - 使用 Vue 3 + TypeScript + Tauri 2

---

## 專案概述

這是 RedForge 平台的 **Vue 版本**前端應用，採用現代化的技術棧打造跨平台桌面應用。

### 技術棧

| 類別 | 技術 |
|------|------|
| **框架** | Vue 3 (Composition API) |
| **語言** | TypeScript |
| **狀態管理** | Pinia |
| **UI 框架** | Tailwind CSS 3 |
| **視覺化** | Chart.js + vue-chartjs |
| **桌面框架** | Tauri 2 |
| **建置工具** | Vite |

---

## 快速開始

### 環境需求

- Node.js 18+
- Rust 1.70+
- npm 或 pnpm

### 安裝依賴

```bash
npm install
```

### 開發模式

```bash
# 啟動 Tauri 開發伺服器（含熱重載）
npm run tauri dev
```

### 建置生產版本

```bash
# 建置跨平台桌面應用
npm run tauri build
```

### 僅前端開發

```bash
# 只啟動 Vite 開發伺服器（不含 Tauri）
npm run dev
```

---

## 專案結構

```
redforge-scanner-vue/
├── src/                          # 前端源碼
│   ├── App.vue                   # 根組件
│   ├── main.ts                   # 入口點
│   │
│   ├── components/               # Vue 組件
│   │   ├── ui/                   # 基礎 UI 組件
│   │   │   ├── Button.vue
│   │   │   ├── Input.vue
│   │   │   └── Modal.vue
│   │   │
│   │   ├── collaboration/        # 協作功能組件
│   │   │   ├── ExportDialog.vue
│   │   │   ├── ImportDialog.vue
│   │   │   └── PreviewModal.vue
│   │   │
│   │   ├── Dashboard.vue         # 主儀表板
│   │   ├── Scanner.vue           # 掃描介面
│   │   └── ScanHistory.vue       # 歷史記錄
│   │
│   ├── stores/                   # Pinia 狀態管理
│   │   ├── export.ts             # 匯出功能
│   │   ├── import.ts             # 匯入功能
│   │   └── scan.ts               # 掃描狀態
│   │
│   ├── services/                 # 服務層
│   │   ├── api.ts                # Tauri API 封裝
│   │   ├── database.ts           # SQLite 操作
│   │   ├── encryption.ts         # 加密服務
│   │   ├── markdown.ts           # Markdown 生成
│   │   └── websocket.ts          # WebSocket 連線
│   │
│   ├── composables/              # Vue Composables
│   │   └── useScanPersistence.ts
│   │
│   ├── types/                    # TypeScript 型別
│   │   └── offline-collaboration.ts
│   │
│   └── styles/                   # 樣式檔案
│       └── main.css
│
├── src-tauri/                    # Rust 後端
│   ├── src/
│   │   ├── main.rs               # 程式入口
│   │   ├── lib.rs                # 函式庫入口
│   │   │
│   │   ├── commands/             # Tauri IPC 命令
│   │   │   ├── scan.rs           # 掃描相關命令
│   │   │   └── collaboration.rs  # 協作相關命令
│   │   │
│   │   ├── scanners/             # 掃描引擎
│   │   │   ├── mod.rs
│   │   │   ├── http_scanner.rs   # HTTP 安全標頭
│   │   │   ├── ssl_scanner.rs    # SSL/TLS 分析
│   │   │   ├── vulnerability_scanner.rs
│   │   │   ├── owasp_scanner.rs  # OWASP 檢查
│   │   │   └── tech_detector.rs  # 技術偵測
│   │   │
│   │   ├── models/               # 資料模型
│   │   │   └── mod.rs
│   │   │
│   │   └── database/             # 資料庫
│   │       └── mod.rs
│   │
│   ├── Cargo.toml                # Rust 依賴
│   └── tauri.conf.json           # Tauri 設定
│
├── dist/                         # 建置輸出
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

---

## 核心組件

### Dashboard.vue
主儀表板，顯示：
- 掃描統計概覽
- 最近掃描結果
- 漏洞分佈圖表
- 快速操作按鈕

### Scanner.vue
掃描控制介面：
- 目標 URL 輸入
- 掃描類型選擇
- 即時進度顯示
- 掃描結果預覽

### ScanHistory.vue
歷史記錄管理：
- 掃描記錄列表
- 搜尋與篩選
- 報告匯出
- 記錄刪除

---

## 狀態管理 (Pinia Stores)

### scanStore
```typescript
// 掃描狀態管理
const scanStore = useScanStore()
scanStore.startScan(url, options)
scanStore.getScanStatus(scanId)
scanStore.listScans()
```

### exportStore
```typescript
// 匯出功能
const exportStore = useExportStore()
exportStore.exportAsJSON(scanId)
exportStore.exportAsMarkdown(scanId, encrypted)
```

### importStore
```typescript
// 匯入功能
const importStore = useImportStore()
importStore.importFromFile(file)
importStore.deduplicateData(data)
```

---

## Tauri 命令

### 掃描相關

```typescript
import { invoke } from '@tauri-apps/api/core'

// 啟動掃描
await invoke('start_scan', { url, options })

// 取得掃描狀態
await invoke('get_scan_status', { scanId })

// 列出掃描記錄
await invoke('list_scans')

// 取得掃描報告
await invoke('get_scan_report', { scanId })
```

### 協作相關

```typescript
// 匯出掃描資料
await invoke('export_scan_data', { scanId, encrypted })

// 匯入掃描資料
await invoke('import_scan_data', { data })

// 去重匯入資料
await invoke('deduplicate_import_data', { data })
```

---

## 開發指南

### IDE 設定

推薦使用 VS Code 並安裝以下擴充套件：

- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) - Vue 3 支援
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) - Tauri 開發工具
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) - Rust 語言支援
- [Tailwind CSS IntelliSense](https://marketplace.visualstudio.com/items?itemName=bradlc.vscode-tailwindcss) - Tailwind 自動完成

### 程式碼風格

- Vue 組件使用 `<script setup>` 語法
- TypeScript 嚴格模式
- 遵循 Vue 3 Composition API 最佳實踐

### 新增組件

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'

// Props
const props = defineProps<{
  title: string
  count?: number
}>()

// Emits
const emit = defineEmits<{
  (e: 'update', value: string): void
}>()

// State
const isLoading = ref(false)

// Computed
const displayTitle = computed(() => `${props.title} (${props.count ?? 0})`)
</script>

<template>
  <div class="component-wrapper">
    <h2>{{ displayTitle }}</h2>
  </div>
</template>

<style scoped>
/* Scoped styles */
</style>
```

---

## 建置與部署

### 開發建置

```bash
npm run tauri dev
```

### 生產建置

```bash
npm run tauri build
```

建置產物位於：
- **macOS**: `src-tauri/target/release/bundle/dmg/`
- **Windows**: `src-tauri/target/release/bundle/msi/`
- **Linux**: `src-tauri/target/release/bundle/deb/`

### 環境變數

```bash
# .env.development
VITE_API_URL=http://localhost:3000

# .env.production
VITE_API_URL=https://api.redforge.dev
```

---

## 測試

```bash
# 前端單元測試
npm run test

# 前端 E2E 測試
npm run test:e2e

# Rust 後端測試
cd src-tauri && cargo test
```

---

## 相關專案

- [主專案 README](../README.md)
- [React 版本（已凍結）](../redforge-scanner/README.md)

---

## 授權

MIT License - 詳見 [LICENSE](../LICENSE)
