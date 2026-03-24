use tauri::{Manager, menu::{Menu, MenuItem}, tray::TrayIconBuilder};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.get_webview_window("main")
                .map(|window| {
                    #[cfg(target_os = "macos")]
                    app.set_activation_policy(tauri::ActivationPolicy::Regular).unwrap_or(());
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();

                    #[cfg(target_os = "macos")]
                    {
                        let icon_bytes = include_bytes!("../icons/128x128.png");
                        if let Ok(img) = tauri::image::Image::from_bytes(icon_bytes) {
                            let _ = window.set_icon(img);
                        }
                    }
                });
        }))
        .setup(|app| {
            // --- 系统托盘架构 (System Tray) ---
            let icon_bytes = include_bytes!("../icons/32x32.png");
            let img = tauri::image::Image::from_bytes(icon_bytes)?;

            let quit_i = MenuItem::with_id(app, "quit", "退出 (Quit)", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "显示主窗口 (Show)", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(img)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            #[cfg(target_os = "macos")]
                            app.set_activation_policy(tauri::ActivationPolicy::Regular).unwrap_or(());
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();

                            #[cfg(target_os = "macos")]
                            {
                                let icon_bytes = include_bytes!("../icons/128x128.png");
                                if let Ok(img) = tauri::image::Image::from_bytes(icon_bytes) {
                                    let _ = window.set_icon(img);
                                }
                            }
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            #[cfg(target_os = "macos")]
                            app.set_activation_policy(tauri::ActivationPolicy::Regular).unwrap_or(());
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();

                            #[cfg(target_os = "macos")]
                            {
                                let icon_bytes = include_bytes!("../icons/128x128.png");
                                if let Ok(img) = tauri::image::Image::from_bytes(icon_bytes) {
                                    let _ = window.set_icon(img);
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            // --- 后端服务初始化 (Axum Server) ---
            tauri::async_runtime::spawn(async move {
                tracing::info!("Starting bundled Axum server from Tauri...");
                if let Err(e) = cli_server::run_server(Some(5173)).await {
                    tracing::error!("Failed to start Axum server: {}", e);
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // 拦截关闭按钮，改为隐藏至后台（托盘）
                let _ = window.hide();
                #[cfg(target_os = "macos")]
                window.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory).unwrap_or(());
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![greet])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(window) = app_handle.get_webview_window("main") {
                    #[cfg(target_os = "macos")]
                    app_handle.set_activation_policy(tauri::ActivationPolicy::Regular).unwrap_or(());
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();

                    #[cfg(target_os = "macos")]
                    {
                        let icon_bytes = include_bytes!("../icons/128x128.png");
                        if let Ok(img) = tauri::image::Image::from_bytes(icon_bytes) {
                            let _ = window.set_icon(img);
                        }
                    }
                }
            }
        });
}
