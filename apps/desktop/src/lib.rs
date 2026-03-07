use {tauri::Manager, tokio::sync::oneshot, tracing::info};

struct GatewayHandle {
    port: u16,
    _shutdown_tx: oneshot::Sender<()>,
}

/// Boot the embedded gateway on a tokio runtime, returning the actual listen
/// port and a shutdown handle.
async fn boot_gateway() -> anyhow::Result<GatewayHandle> {
    let config = moltis_config::discover_and_load();
    let bind = config.server.bind.clone();
    let port = config.server.port;

    let prepared = moltis_gateway::server::prepare_gateway_embedded(
        &bind,
        port,
        true, // no_tls — WebView loads localhost, TLS unnecessary
        None, // log_buffer
        None, // config_dir — use default
        None, // data_dir  — use default
        Some(moltis_web::web_routes),
        None, // session_event_bus
    )
    .await?;

    let addr: std::net::SocketAddr = format!("{bind}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let app = prepared.app;

    tokio::spawn(async move {
        let server = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });
        if let Err(e) = server.await {
            tracing::error!("Gateway server error: {e}");
        }
    });

    info!("Gateway listening on 127.0.0.1:{actual_port}");

    Ok(GatewayHandle {
        port: actual_port,
        _shutdown_tx: shutdown_tx,
    })
}

pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Build tokio runtime for the gateway
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap_or_else(|e| panic!("failed to build tokio runtime: {e}"));

    // Start gateway, obtain listen port
    let gateway = runtime
        .block_on(boot_gateway())
        .unwrap_or_else(|e| panic!("failed to start gateway: {e}"));

    let url = format!("http://127.0.0.1:{}", gateway.port);

    // Keep runtime + gateway alive for the lifetime of the app
    let _runtime_guard = runtime;
    let _gateway_guard = gateway;

    let mut builder = tauri::Builder::default().plugin(tauri_plugin_shell::init());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }));
    }

    builder
        .setup(move |app| {
            // Main window — loads the gateway web UI
            let _win = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External(
                    url.parse().unwrap_or_else(|e| panic!("invalid url: {e}")),
                ),
            )
            .title("Moltis")
            .inner_size(1200.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .build()?;

            // System tray
            setup_tray(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| panic!("error while running tauri application: {e}"));
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::{
        menu::{MenuBuilder, MenuItemBuilder},
        tray::{MouseButton, MouseButtonState, TrayIconBuilder},
    };

    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

    let _tray = TrayIconBuilder::new()
        .tooltip("Moltis")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            },
            "quit" => {
                app.exit(0);
            },
            _ => {},
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(w) = tray.app_handle().get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
