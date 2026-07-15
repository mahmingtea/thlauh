mod file_server;
mod mdns;
mod network;
mod portal;

use file_server::{get_shared_files, start_sharing_server, stop_sharing_server, StartServerResult};
use network::{get_all_ip_addresses, NetworkInterfaceInfo};
use portal::FileItem;
use serde::Serialize;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};
use tauri_plugin_dialog::DialogExt;

// ─── Tauri Commands ───────────────────────────────────────────────────

#[tauri::command]
fn get_ips() -> Vec<NetworkInterfaceInfo> {
    get_all_ip_addresses()
}

#[derive(Serialize)]
struct ServerInfo {
    path: String,
    #[serde(rename = "serverUrl")]
    server_url: String,
    #[serde(rename = "mdnsUrl")]
    mdns_url: String,
    #[serde(rename = "passwordHash")]
    password_hash: Option<String>,
}

#[tauri::command]
async fn select_folder(app: tauri::AppHandle) -> Option<String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder();

    folder.map(|f| f.to_string())
}

#[tauri::command]
fn get_files(path: String) -> Vec<FileItem> {
    get_shared_files(&path)
}

#[tauri::command]
async fn start_server(
    folder_path: String,
    chosen_ip: String,
    password: Option<String>,
    custom_mdns_host: Option<String>,
) -> Option<ServerInfo> {
    let pwd = password.as_deref();
    let mdns_host = custom_mdns_host.as_deref();

    let result: StartServerResult =
        start_sharing_server(&folder_path, &chosen_ip, 4000, pwd, mdns_host);

    Some(ServerInfo {
        path: folder_path.clone(),
        server_url: format!("http://{}:4000", chosen_ip),
        mdns_url: result.mdns_url,
        password_hash: if result.password_hash.is_empty() {
            None
        } else {
            Some(result.password_hash)
        },
    })
}

#[tauri::command]
async fn cmd_stop_server() -> bool {
    stop_sharing_server();
    true
}

#[tauri::command]
async fn cmd_set_theme(theme: String) -> bool {
    file_server::set_theme(&theme).await;
    true
}

// ─── App setup ────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_ips,
            select_folder,
            get_files,
            start_server,
            cmd_stop_server,
            cmd_set_theme,
        ])
        .setup(|app| {
            // ── System Tray ──
            let show_item = MenuItemBuilder::new("Show App")
                .id("show")
                .build(app)?;
            let quit_item = MenuItemBuilder::new("Quit")
                .id("quit")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(Image::from_bytes(include_bytes!("../icons/32x32.png"))?)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            stop_sharing_server();
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Window close → hide is handled by on_window_event below

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
