# Moltis 架构概览与桌面化分析

## 1. 项目定位

Moltis 是 [OpenClaw](https://docs.openclaw.ai) 的 Rust 重写版，定位为**个人 AI 网关（Personal AI Gateway）**。它将多个 LLM 提供商、聊天通道（Telegram/Discord/Slack/WhatsApp/Teams）、工具执行、语音、记忆/RAG、定时任务等统一编排，通过 Web UI 或原生客户端交互。

## 2. 整体架构

```
┌─────────────────────────────────────────────────────┐
│                    客户端层                           │
│  Web SPA (JS/Preact)  │  macOS (SwiftUI)  │  iOS    │
└──────────┬────────────┴───────┬────────────┴────────┘
           │ WebSocket (v4 协议)  │ FFI (swift-bridge)
           ▼                    ▼
┌─────────────────────────────────────────────────────┐
│              moltis-gateway (核心服务)                │
│  HTTP/WS 路由 · RPC 分发 · 认证 · 广播 · 限流        │
├─────────────────────────────────────────────────────┤
│  moltis-chat        对话编排、上下文组装、消息持久化     │
│  moltis-agents      Agent 运行时、流式响应、工具调度    │
│  moltis-providers   LLM 提供商 (Anthropic/OpenAI/     │
│                     本地GGUF/Copilot/OpenRouter...)    │
│  moltis-tools       沙箱执行、审批、浏览器、WASM       │
│  moltis-memory      SQLite FTS + tree-sitter 分片     │
│  moltis-sessions    SQLite 会话存储                    │
│  moltis-channels    Telegram/Discord/Slack/WhatsApp   │
│  moltis-voice       TTS (ElevenLabs/OpenAI) + STT     │
│  moltis-cron        定时任务 + CalDAV                  │
│  moltis-mcp         MCP 服务集成                       │
│  moltis-skills      技能注册表 (YAML/tar)              │
│  moltis-plugins     Hook 事件分发                      │
│  moltis-config      TOML/YAML 配置 + 环境变量替换      │
│  moltis-auth        Password + Passkey + OAuth         │
│  moltis-vault       E2E 加密保险库                     │
└─────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│              存储层                                   │
│  SQLite (sessions, auth, cron, memory, projects)     │
│  文件系统 (config, provider_keys, WASM bundles)       │
└─────────────────────────────────────────────────────┘
```

## 3. Workspace 结构 (54 crates)

| 分类 | 核心 Crate | 说明 |
|------|-----------|------|
| **入口** | `moltis` (cli) | CLI 入口，子命令：`gateway`, `agent`, `auth`, `config`, `sandbox` |
| **服务核心** | `moltis-gateway` (36K LoC) | HTTP/WS 服务器 (axum)，RPC 分发，认证中间件 |
| **对话** | `moltis-chat`, `moltis-agents` | 对话编排、Agent 运行循环、流式响应 |
| **模型** | `moltis-providers` (17K LoC) | Anthropic、OpenAI、本地 GGUF、genai、Copilot 等 |
| **工具** | `moltis-tools` (22K LoC) | Docker/Apple Container 沙箱、Shell、浏览器自动化、WASM |
| **Web** | `moltis-web` | SPA 路由、静态资源（dev 磁盘/release `include_dir!`） |
| **通道** | `moltis-telegram/discord/slack/whatsapp/msteams` | 各聊天平台集成 |
| **原生** | `moltis-swift-bridge` | Rust staticlib → Swift FFI |
| **其他** | `moltis-memory`, `moltis-cron`, `moltis-voice`, `moltis-mcp`, `moltis-graphql` | 功能模块 |

## 4. 通信协议

**Protocol v4**：JSON 帧 over WebSocket，三种帧类型：

- `RequestFrame`：`{ type: "req", id, method, params }`
- `ResponseFrame`：`{ type: "res", id, ok, payload }`
- `EventFrame`：`{ type: "event", event, payload, seq, stream, done }`

Web 客户端和 iOS 客户端均通过此协议与 gateway 通信。macOS 原生应用则通过 **FFI (swift-bridge)** 直接调用 Rust 库，内嵌 gateway。

## 5. 现有原生应用方案

### macOS 应用 (`apps/macos/`)
- **SwiftUI** 原生界面
- 通过 `moltis-swift-bridge`（Rust staticlib）直接嵌入整个 gateway
- Rust→Swift 回调：日志、会话事件、网络审计
- 包含：聊天、设置、Provider 配置、日志查看、Onboarding

### iOS 应用 (`apps/ios/`)
- **SwiftUI** 原生界面
- 通过 **GraphQL + WebSocket** 连接远程 gateway（客户端模式）
- Apollo 生成 GraphQL 类型
- Bonjour 局域网发现
- Widget + Live Activity 支持

### 架构差异
| | macOS | iOS |
|---|---|---|
| 部署模式 | 嵌入式（gateway 运行在本地） | 远程（连接已有 gateway） |
| 通信方式 | FFI (swift-bridge) | GraphQL + WebSocket |
| 沙箱执行 | 本地 Docker/Apple Container | 依赖远程 gateway |

---

## 6. 桌面化方案分析

### 当前状态

项目已有三条路径与桌面相关：
1. **Web UI** — 可直接在浏览器使用（`localhost` 模式）
2. **macOS SwiftUI 应用** — 已有成熟实现，FFI 内嵌 gateway
3. CLI 模式 — 终端使用

### 方案一：基于现有 Web UI 的 WebView 壳（推荐 — Windows/Linux）

**思路**：用 [Tauri](https://tauri.app/) 或 [Wry](https://github.com/nicehash/nicewry) 包装已有 Web UI，进程内启动 gateway。

```
┌────────────────────────────────┐
│  Tauri 窗口 (WebView2/WebKitGTK) │
│  ┌──────────────────────────┐  │
│  │  现有 Web UI (JS/CSS/HTML) │  │
│  └───────────┬──────────────┘  │
│              │ localhost WS     │
│  ┌───────────▼──────────────┐  │
│  │  moltis-gateway (嵌入)    │  │
│  │  (tokio runtime)         │  │
│  └──────────────────────────┘  │
└────────────────────────────────┘
```

**优势**：
- 复用全部现有 Web UI 代码（JS/CSS/HTML），零重写
- 复用全部 Rust 后端（gateway 进程内启动）
- Tauri 的系统托盘、菜单、自动更新、安装器均可用
- 跨 Windows/Linux/macOS
- 开发成本最低

**实施步骤**：
1. 在 `apps/desktop/` 下初始化 Tauri 项目
2. Tauri `setup` hook 中用 `tokio::spawn` 启动 gateway（参考 `moltis-swift-bridge` 的启动逻辑）
3. WebView 加载 `http://127.0.0.1:{port}`
4. 添加系统托盘（最小化到托盘、通知）
5. 打包：Tauri 自带 `.msi`(Win) / `.deb`/`.AppImage`(Linux) / `.dmg`(macOS)

**关键代码参考**：
- gateway 启动：`crates/swift-bridge/src/lib.rs` 中的 `start_gateway()` 逻辑
- Web 资源嵌入：`crates/web/` 的 `embedded-assets` feature（`include_dir!`）

**注意**：
- macOS 可继续使用现有 SwiftUI 原生应用（更好体验）
- Tauri 主要解决 **Windows + Linux** 桌面需求
- 沙箱执行需要 Docker（Windows/Linux 无 Apple Container）

### 方案二：原生 UI 扩展（Swift → Kotlin/C++）

仿照 macOS 应用的 FFI 模式，为 Windows 写 WinUI 3 / WPF 原生界面。

**优势**：最佳原生体验
**劣势**：每个平台单独维护 UI（成本极高），不推荐除非有专门的前端团队

### 方案三：Electron

**不推荐**。项目已有完整 Rust 后端和轻量 Web 前端，引入 Electron 增加 ~200MB 内存开销且无必要。Tauri 使用系统 WebView 即可。

### 推荐策略

```
macOS  → 继续使用现有 SwiftUI 应用（已完善）
Windows/Linux → 新增 Tauri 壳（复用 Web UI + 嵌入 gateway）
iOS    → 继续使用现有 SwiftUI 客户端
```

### 方案一的具体工作量估算

| 工作项 | 复杂度 | 说明 |
|--------|--------|------|
| Tauri 项目脚手架 | 低 | `cargo tauri init` |
| Gateway 嵌入启动 | 中 | 参考 swift-bridge，适配 Tauri lifecycle |
| 系统托盘 + 菜单 | 低 | Tauri 内置 API |
| 自动更新 | 低 | Tauri updater plugin |
| Windows 安装器 | 低 | Tauri 自带 WiX/NSIS |
| Linux 打包 | 低 | Tauri 自带 deb/AppImage |
| 沙箱适配 | 中 | Windows 上需确保 Docker Desktop 或 WSL 可用 |
| 测试 | 中 | E2E 测试复用现有 Playwright 套件 |

总体来说，由于后端和前端代码完全复用，桌面化的核心工作是 **Tauri 集成层**，代码量预计在 500-1000 行 Rust + 配置文件。
