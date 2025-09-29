use std::fs;
use std::path::Path;

fn main() {
    // Ensure WebView2Loader.dll exists for Windows builds
    #[cfg(target_os = "windows")]
    {
        let dll_path = Path::new("WebView2Loader.dll");
        if !dll_path.exists() {
            println!("cargo:warning=WebView2Loader.dll not found in src-tauri directory!");
            println!("cargo:warning=The application will not run without this file.");
            println!("cargo:warning=Download it from the Microsoft.Web.WebView2 NuGet package.");
        }
    }

    tauri_build::build()
}