# Tauri 桌面应用实施方案

> 目标：在 `apps/desktop/` 下新建一个 Tauri v2 项目，不修改任何现有文件。
> 覆盖平台：Windows、Linux（macOS 继续使用现有 SwiftUI 应用）。

## 1. 架构设计

```
apps/desktop/
├── Cargo.toml              # Rust 后端 crate（依赖 moltis-gateway + moltis-web）
├── src/
│   └── lib.rs              # Tauri setup：启动 gateway，WebView 加载 localhost
├── tauri.conf.json          # Tauri 配置（窗口、图标、打包）
├── capabilities/
│   └── default.json         # Tauri v2 权限声明
├── icons/                   # 应用图标（各尺寸 PNG + ICO）
└── build.rs                 # Tauri build script（codegen）
```

核心思路：**不自带前端资源**。Tauri WebView 直接加载 `http://127.0.0.1:{port}`，
由嵌入的 `moltis-gateway` + `moltis-web` 提供完整 Web UI。

```
┌─────────────────────────────────────────┐
│           Tauri 进程                      │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │    WebView (WebView2 / WebKitGTK)  │  │
│  │    加载 http://127.0.0.1:{port}    │  │
│  └───────────────┬────────────────────┘  │
│                  │ HTTP/WS                │
│  ┌───────────────▼────────────────────┐  │
│  │    moltis-gateway (tokio runtime)  │  │
│  │    + moltis-web (静态资源 + API)    │  │
│  └────────────────────────────────────┘  │
│                                          │
│  系统托盘 · 菜单 · 自动更新 · 通知       │
└─────────────────────────────────────────┘
```

## 2. 文件清单

### 2.1 `apps/desktop/Cargo.toml`

```toml
[package]
name    = "moltis-desktop"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
# Tauri v2 要求 lib crate
name = "moltis_desktop_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri            = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell       = "2"
tauri-plugin-notification = "2"
serde            = { features = ["derive"], workspace = true }
serde_json       = { workspace = true }
tokio            = { workspace = true }
anyhow           = { workspace = true }
tracing          = { workspace = true }
tracing-subscriber = { workspace = true }

# 嵌入 gateway + web UI（与 swift-bridge 相同的依赖模式）
moltis-gateway   = { workspace = true, default-features = true, features = ["web-ui", "metrics", "file-watcher"] }
moltis-web       = { workspace = true }
moltis-config    = { workspace = true }
moltis-sessions  = { workspace = true }
moltis-projects  = { workspace = true }
moltis-tools     = { workspace = true, features = ["embedded-wasm"] }

[features]
default  = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]  # release 模式

[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]
tauri-plugin-single-instance = "2"
```

### 2.2 `apps/desktop/build.rs`

```rust
fn main() {
    tauri_build::build();
}
```

### 2.3 `apps/desktop/src/lib.rs`（核心 ~120 行）

```rust
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::oneshot;

/// Gateway 启动后返回的信息
struct GatewayHandle {
    port: u16,
    _state: Arc<moltis_gateway::state::GatewayState>,
}

/// 在后台 tokio runtime 启动 gateway，返回实际监听端口
async fn boot_gateway() -> anyhow::Result<GatewayHandle> {
    let config = moltis_config::discover_and_load();
    let bind = config.server.bind.clone();
    let port = config.server.port;

    // 使用 prepare_gateway_embedded（与 macOS swift-bridge 相同路径）
    let prepared = moltis_gateway::server::prepare_gateway_embedded(
        &bind,
        port,
        true, // no_tls — Tauri WebView 走 localhost，不需要 TLS
        None, // log_buffer
        None, // config_dir — 使用默认
        None, // data_dir — 使用默认
        Some(moltis_web::web_routes), // 挂载完整 Web UI
        None, // session_event_bus
    )
    .await?;

    let state = prepared.state.clone();
    let addr: std::net::SocketAddr = format!("{bind}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();

    // 后台运行 axum server
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, prepared.app)
            .await
        {
            tracing::error!("Gateway server error: {e}");
        }
    });

    Ok(GatewayHandle {
        port: actual_port,
        _state: state,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 构建 tokio runtime（gateway 需要）
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    // 先启动 gateway，拿到端口
    let (tx, rx) = oneshot::channel();
    runtime.spawn(async move {
        match boot_gateway().await {
            Ok(handle) => { let _ = tx.send(Ok(handle)); },
            Err(e) => { let _ = tx.send(Err(e)); },
        }
    });

    let gateway = runtime.block_on(rx).expect("gateway channel closed").expect("gateway boot failed");
    let url = format!("http://127.0.0.1:{}", gateway.port);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 聚焦已有窗口
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }))
        .setup(move |app| {
            // 创建主窗口，加载 gateway URL
            let win = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External(url.parse().expect("invalid gateway URL")),
            )
            .title("Moltis")
            .inner_size(1200.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .build()?;

            // 系统托盘（关闭窗口时最小化到托盘）
            use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState};
            use tauri::menu::{MenuBuilder, MenuItemBuilder};

            let show = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

            let _tray = TrayIconBuilder::new()
                .tooltip("Moltis")
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        },
                        "quit" => { app.exit(0); },
                        _ => {},
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2.4 `apps/desktop/src/main.rs`

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    moltis_desktop_lib::run();
}
```

### 2.5 `apps/desktop/tauri.conf.json`

```json
{
  "$schema": "https://raw.githubusercontent.com/nicehash/nicewry/refs/heads/main/nicewry-schema.json",
  "productName": "Moltis",
  "version": "0.1.0",
  "identifier": "org.moltis.desktop",
  "build": {
    "frontendDist": false
  },
  "app": {
    "withGlobalTauri": false,
    "windows": []
  },
  "bundle": {
    "active": true,
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "targets": ["nsis", "deb", "appimage"],
    "windows": {
      "nsis": {
        "installMode": "currentUser"
      }
    },
    "linux": {
      "deb": {
        "depends": ["libwebkit2gtk-4.1-0", "libgtk-3-0"]
      }
    }
  }
}
```

### 2.6 `apps/desktop/capabilities/default.json`

```json
{
  "identifier": "default",
  "description": "Default capabilities for Moltis desktop",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "notification:default"
  ]
}
```

## 3. Workspace 集成（唯一需要改动的外部文件）

在根 `Cargo.toml` 的 `members` 和 `default-members` 中加一行：

```toml
"apps/desktop",
```

> 如果严格要求不改任何现有文件，可以在开发阶段用 `cargo build -p moltis-desktop --manifest-path apps/desktop/Cargo.toml` 单独构建，但最终集成时需要加入 workspace。

## 4. 开发与构建流程

```bash
# 一次性安装 Tauri CLI
cargo install tauri-cli --version "^2"

# 开发模式（热重载 WebView，gateway 自动启动）
cd apps/desktop
cargo tauri dev

# 生产构建
cargo tauri build
# 产物：
#   Windows: apps/desktop/target/release/bundle/nsis/Moltis_0.1.0_x64-setup.exe
#   Linux:   apps/desktop/target/release/bundle/deb/moltis_0.1.0_amd64.deb
#            apps/desktop/target/release/bundle/appimage/Moltis_0.1.0_amd64.AppImage
```

## 5. 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 前端资源来源 | gateway 动态提供 | 复用现有 Web UI，零重复；`frontendDist: false` |
| TLS | 关闭 (`no_tls: true`) | 本地 loopback 不需要 TLS |
| Gateway 启动方式 | `prepare_gateway_embedded` | 与 macOS 应用一致的已验证路径 |
| 单实例 | `tauri-plugin-single-instance` | 防止多开冲突（共享 SQLite） |
| 系统托盘 | 内置 | 关闭窗口时最小化，后台保持 gateway 运行 |
| 窗口创建 | `WebviewUrl::External` | 直接加载 gateway HTTP URL |

## 6. 后续增强（可选）

| 功能 | 实现方式 | 优先级 |
|------|---------|--------|
| 自动更新 | `tauri-plugin-updater` + GitHub Releases | P1 |
| 桌面通知 | `tauri-plugin-notification`（已集成） | P1 |
| 全局快捷键 | `tauri-plugin-global-shortcut` | P2 |
| 深度链接 | `tauri-plugin-deep-link` (`moltis://`) | P2 |
| 开机自启 | `tauri-plugin-autostart` | P3 |
| 图标替换 | 从现有 `apps/macos/Assets.xcassets` 导出 | P1 |
| CI 打包 | GitHub Actions + `cargo tauri build` | P1 |

## 7. 注意事项

1. **WebView2 (Windows)**：Tauri v2 依赖 WebView2，Windows 10 1803+ 自带，老版本需要安装器 bootstrapper（NSIS 安装包默认处理）。
2. **SQLite 并发**：gateway 使用 WAL 模式，单实例插件防止多进程冲突。
3. **沙箱执行**：Windows 上需要 Docker Desktop 或 WSL2；Linux 上需要 Docker。
4. **端口冲突**：如果默认端口被占用，gateway 会报错。可考虑自动选择空闲端口（传 port=0）。
5. **Cargo workspace**：`apps/desktop` 必须加入 workspace `members`（唯一需要改的现有文件）。如果暂时不想动根 Cargo.toml，可以先用独立 workspace 开发。
