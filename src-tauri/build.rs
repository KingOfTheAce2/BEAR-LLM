fn main() {
    // Ensure WebView2Loader.dll exists for Windows builds
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        let dll_path = Path::new("WebView2Loader.dll");
        if !dll_path.exists() {
            println!("cargo:warning=WebView2Loader.dll not found in src-tauri directory!");
            println!("cargo:warning=The application will not run without this file.");
            println!("cargo:warning=Download it from the Microsoft.Web.WebView2 NuGet package.");
            std::process::exit(1);
        } else {
            // Mark the DLL for cargo to track changes
            println!("cargo:rerun-if-changed=WebView2Loader.dll");
        }
    }

    tauri_build::build()
}