// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        // Fix for WebKit2GTK on Linux (especially with NVIDIA / hybrid GPUs)
        // Resolves "Could not create GBM EGL display: EGL_NOT_INITIALIZED. Aborting..."
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    tauri_app_lib::run()
}
