fn main() {
    println!("cargo:rerun-if-changed=packaging/icons/app-icon.ico");

    #[cfg(target_os = "windows")]
    winresource::WindowsResource::new()
        .set_icon("packaging/icons/app-icon.ico")
        .compile()
        .expect("failed to embed the Windows application icon");
}
