fn main() {
    // Only build on Windows
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-cfg=windows_build");
    } else {
        println!("cargo:warning=This library is designed for Windows and ACC. Building stub version for cross-platform compatibility.");
    }
}