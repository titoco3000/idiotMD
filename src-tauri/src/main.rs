// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::{path::PathBuf, sync::Mutex};
use tauri::Window;
use tauri::{api::dialog::blocking::FileDialogBuilder, Manager};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn set_save_status(app_handle: tauri::AppHandle, saved:bool){
    for (_key, window) in app_handle.windows().into_iter() {
        if saved{
            window.set_title("iMD").unwrap();
        }
        else {
            window.set_title("iMD 🞄").unwrap();
            
        }
    }
}

#[tauri::command]
fn parse(
    app_handle: tauri::AppHandle,
    file_info: tauri::State<'_, Mutex<Option<(PathBuf, u64)>>>,
    input: &str,
) -> String {
    set_save_status(app_handle, calculate_hash(&input) == file_info.lock().unwrap().as_ref().unwrap_or(&(PathBuf::new(),calculate_hash(&""))).1);

    parse_markdown(input)
}

fn parse_markdown(input: &str) -> String {
    markdown::to_html_with_options(
        input,
        &markdown::Options {
            parse: markdown::ParseOptions {
                constructs: markdown::Constructs {
                    math_text: true,
                    ..markdown::Constructs::default()
                },
                ..markdown::ParseOptions::default()
            },
            ..markdown::Options::default()
        },
    )
    .unwrap_or(String::from("Error"))
    .replace(
        "<code class=\"language-math math-inline\">",
        "<span class=\"math-part\">$$",
    )
    .replace("</code>", "$$</span>")
}

#[tauri::command]
fn openinitialfile(app_handle: tauri::AppHandle, file_info: tauri::State<'_, Mutex<Option<(PathBuf, u64)>>>) -> (String, String) {
    if let Some(ref_to_file_info) = file_info.lock().unwrap().as_mut() {
        match std::fs::read_to_string(&ref_to_file_info.0) {
            Ok(content) => {
                let hash = calculate_hash(&content);
                *ref_to_file_info = (ref_to_file_info.0.clone(), hash);
                set_save_status(app_handle, true);
                let parsed = parse_markdown(&content);
                (content, parsed)
            }
            Err(_) => {
                (String::new(), String::new())
            },
        }
    } else {
        (String::new(), String::new())
    }
}

#[tauri::command]
fn openfiledialog(app_handle: tauri::AppHandle,file_info: tauri::State<'_, Mutex<Option<(PathBuf, u64)>>>) -> (String, String) {
    match FileDialogBuilder::new()
    .pick_file() {
        Some(file_path) => match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                set_save_status(app_handle, true);
                let parsed = parse_markdown(&content);
                *file_info.lock().unwrap() = Some((file_path, calculate_hash(&content)));
                (content, parsed)
            }
            Err(_) => (String::new(), String::new()),
        },
        None => (String::new(), String::new()),
    }
}

#[tauri::command]
fn savefile(
    app_handle: tauri::AppHandle,
    file_info: tauri::State<'_, Mutex<Option<(PathBuf, u64)>>>,
    content: &str,
    force_dialog: Option<bool>,
) {
    println!("Save file. Force: {:?}",force_dialog);
    //if should open dialog
    if file_info.lock().unwrap().is_none() || (force_dialog.is_some() && force_dialog.unwrap()) {
        match FileDialogBuilder::new()
        .add_filter("Markdown files", &["md"])
        .add_filter("Text files", &["txt"])
        .add_filter("Any", &[""])
        .save_file() {
            None => {
                return;
            }
            Some(p) => {
                *file_info.lock().unwrap() = Some((p, calculate_hash(&content)));
            }
        }
    }

    if let Some(ref_to_file_info) = file_info.lock().unwrap().as_ref() {
        std::fs::write(&ref_to_file_info.0, content).expect("Unable to write file");
    }
    let updated_file_info = Some((file_info.lock().unwrap().as_ref().unwrap().0.clone(), calculate_hash(&content)));
    *file_info.lock().unwrap() = updated_file_info;
    set_save_status(app_handle, true);
}

#[tauri::command]
fn newfile(
    app_handle: tauri::AppHandle,
    file_info: tauri::State<'_, Mutex<Option<(PathBuf, u64)>>>,
) {
    *file_info.lock().unwrap() = None;
    set_save_status(app_handle, true);
}

fn configure_webview_shortcuts(app: &tauri::AppHandle) {
    use windows::core::Interface;
    for (_, window) in app.windows().into_iter() {
        window
            .with_webview(|webview| unsafe {
                let core_webview: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2 =
                    webview
                        .controller()
                        .CoreWebView2()
                        .unwrap()
                        .cast::<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2>()
                        .unwrap();
                let settings = core_webview
                    .Settings()
                    .unwrap()
                    .cast::<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3>()
                    .unwrap();
                settings.SetAreBrowserAcceleratorKeysEnabled(false).unwrap();
            })
            .unwrap();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file_info: Mutex<Option<(PathBuf, u64)>> = Mutex::new(if args.len() > 1 {
        Some((PathBuf::from(&args[1]), 0))
    } else {
        None
    });

    let window: Arc<Option<HashMap<String, Window>>> = Arc::new(None);

    tauri::Builder::default()
        .setup(|app| {
            configure_webview_shortcuts(&app.app_handle());
            Ok(())
        })
        .manage(file_info)
        .manage(window)
        .invoke_handler(tauri::generate_handler![
            parse,
            openinitialfile,
            openfiledialog,
            savefile,
            newfile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
