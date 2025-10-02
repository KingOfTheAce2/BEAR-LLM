fn main() {
    // Ensure WebView2Loader.dll exists for Windows builds
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        let dll_path = Path::new("WebView2Loader.dll");
        if !dll_path.exists() {
            // Use cargo:warning which doesn't create console windows
            eprintln!("cargo:warning=WebView2Loader.dll not found - will be downloaded at runtime");
        } else {
            // Mark the DLL for cargo to track changes
            println!("cargo:rerun-if-changed=WebView2Loader.dll");
        }
    }

    tauri_build::build()
}
