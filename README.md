# RedForge

> 為紅隊打造的協作掃描平台 - 不只是掃描器，而是「掃描器的戰情中心」

[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-green?logo=vue.js)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow)](./LICENSE)

---

## 專案概述

**RedForge** 是一個專業級的紅隊協作平台，專為滲透測試和紅隊演練設計。它不是另一個弱掃工具，而是一個智慧整合多種掃描工具的指揮中心。

### 核心價值

| 特點 | 說明 |
|------|------|
| **多人即時協作** | WebSocket 即時同步，團隊不再單打獨鬥 |
| **智慧整合** | 統一管理 Nmap、Burp、Nuclei 等工具的結果 |
| **攻擊路徑視覺化** | 複雜的攻擊鏈清晰可見 |
| **自動化報告** | 節省 60% 報告撰寫時間 |
| **離線可用** | 跨平台桌面應用，內網環境友善 |

---

## 功能截圖

```
┌─────────────────────────────────────────────────────────────────┐
│  RedForge - Scanner Command Center                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ HTTP Scan   │  │ SSL/TLS     │  │ Vuln Scan   │              │
│  │ ✅ Complete │  │ ✅ Complete │  │ 🔄 Running  │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│                                                                  │
│  Findings: 12 Critical | 8 High | 23 Medium | 45 Low            │
│                                                                  │
│  [Export Report]  [Share with Team]  [View Attack Path]         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 技術架構

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend Layer                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Vue 3 + TypeScript + Pinia + Tailwind CSS              │    │
│  │  • Dashboard (即時狀態)                                  │    │
│  │  • Scanner (掃描控制)                                    │    │
│  │  • History (歷史記錄)                                    │    │
│  │  • Collaboration (團隊協作)                              │    │
│  └─────────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Tauri IPC
┌──────────────────────────▼──────────────────────────────────────┐
│                        Backend Layer (Rust)                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Scanners:                                               │    │
│  │  • http_scanner.rs    - HTTP 安全標頭檢測               │    │
│  │  • ssl_scanner.rs     - SSL/TLS 憑證分析                │    │
│  │  • vuln_scanner.rs    - SQL Injection, XSS 檢測         │    │
│  │  • owasp_scanner.rs   - OWASP Top 10 檢查               │    │
│  │  • tech_detector.rs   - 技術棧識別                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Database: SQLite (本地儲存)                             │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 快速開始

### 系統需求

- **Node.js** 18+
- **Rust** 1.70+
- **npm** 或 **pnpm**

### 安裝與運行

```bash
# 1. Clone 專案
git clone https://github.com/BlakeHung/RedForge.git
cd RedForge

# 2. 進入 Vue 版本目錄
cd redforge-scanner-vue

# 3. 安裝依賴
npm install

# 4. 開發模式運行
npm run tauri dev

# 5. 建置生產版本
npm run tauri build
```

### 專案結構

```
RedForge/
├── README.md                    # 本文件
├── docs/                        # 完整文檔
│   ├── README.md               # 文檔索引
│   ├── ARCHITECTURE.md         # 技術架構
│   ├── FEATURE_ROADMAP.md      # 功能路線圖
│   ├── UI_UX_DESIGN.md         # UI/UX 設計指南
│   └── COLLABORATION_FEATURES.md
├── redforge-scanner-vue/        # Vue 版本（主要開發）
│   ├── src/                    # 前端源碼
│   ├── src-tauri/              # Rust 後端
│   └── package.json
└── redforge-scanner/            # React 版本（備用）
```

---

## 開發路線圖

```
✅ v0.1.0 - 基礎掃描器 MVP
   └─ HTTP 掃描、SSL 分析、基礎漏洞檢測、報告匯出

🔨 v0.2.0 - 戰情儀表板與即時協作（開發中）
   └─ WebSocket 同步、在線成員、目標鎖定、團隊聊天

📅 v0.3.0 - 視覺化與智慧編排
   └─ 攻擊路徑圖、掃描編排器、資產管理

📅 v0.4.0 - 報告系統與 AI 增強
   └─ 智慧報告生成、AI 分析、知識庫

🎯 v1.0.0 - 生產就緒版本
```

---

## 核心功能

### 已完成 (v0.1.0)

- [x] HTTP 安全標頭掃描
- [x] SSL/TLS 憑證分析
- [x] 基礎漏洞檢測（SQL Injection, XSS）
- [x] 技術棧識別
- [x] 掃描歷史管理
- [x] JSON/Markdown 報告匯出
- [x] 暗色主題 UI

### 開發中 (v0.2.0)

- [ ] 即時團隊協作
- [ ] 在線成員狀態
- [ ] 活動任務儀表板
- [ ] 目標鎖定（防止重複測試）
- [ ] 重大發現即時通知
- [ ] 團隊聊天系統

### 規劃中 (v0.3.0+)

- [ ] 攻擊路徑視覺化
- [ ] 智慧掃描編排
- [ ] 外部工具整合（Nmap, Burp, Nuclei）
- [ ] 智慧去重
- [ ] 知識庫系統

---

## 設計理念

### 三大核心原則

#### 1. 駭客美學 (Hacker Aesthetic)
```
暗色為主，高對比度
終端機風格的等寬字體
極簡但資訊密集
專業工具感，非消費級產品
```

#### 2. 實用至上 (Pragmatic First)
```
快捷鍵支援（Vim-like）
最少點擊次數
關鍵資訊優先顯示
避免不必要的動畫
```

#### 3. 即時反饋 (Real-time Feedback)
```
所有操作即時回應
掃描進度實時更新
團隊動態即時顯示
錯誤訊息立即呈現
```

---

## 技術棧

### 前端
- **框架**: Vue 3 (Composition API) + TypeScript
- **狀態管理**: Pinia
- **UI**: Tailwind CSS 3
- **視覺化**: Chart.js, Vue Flow
- **建置工具**: Vite

### 後端
- **框架**: Tauri 2
- **語言**: Rust
- **HTTP 客戶端**: reqwest
- **非同步運行時**: Tokio
- **加密**: rustls, SHA2, AES-256-GCM

### 資料庫
- **類型**: SQLite（本地儲存）

---

## 文檔

完整文檔請參閱 [docs/README.md](./docs/README.md)：

- [系統總體規劃](./docs/DEVCORE_SYSTEM_PLANNING.md)
- [技術架構文件](./docs/ARCHITECTURE.md)
- [功能路線圖](./docs/FEATURE_ROADMAP.md)
- [UI/UX 設計指南](./docs/UI_UX_DESIGN.md)
- [協作功能設計](./docs/COLLABORATION_FEATURES.md)
- [離線協作功能](./docs/OFFLINE_COLLABORATION.md)

---

## 貢獻指南

### 分支策略

```
main          - 生產穩定版本
develop       - 開發主分支
feature/*     - 新功能開發
bugfix/*      - 錯誤修復
```

### Commit 規範

```bash
feat: 新增功能
fix: 錯誤修復
docs: 文檔更新
style: 程式碼格式
refactor: 重構
test: 測試相關
chore: 建置工具
```

---

## 授權

本專案採用 [MIT License](./LICENSE)。

---

## 聯絡資訊

**作者**: Blake Hung
**GitHub**: [@BlakeHung](https://github.com/BlakeHung)

---

*Built with Tauri + Vue + Rust*
