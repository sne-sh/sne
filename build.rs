// Puts the product name, version and icon inside sne.exe on Windows.
//
// Windows shows these in a file's properties, and code signing services require them. A program
// without them also looks more suspicious to antivirus tools.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/sne.ico");

    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource
            .set_icon("assets/sne.ico")
            .set("ProductName", "sne")
            .set("FileDescription", "sne: Node.js version manager")
            .set("CompanyName", "sne contributors")
            .set("LegalCopyright", "MIT License, sne contributors")
            .set("InternalName", "sne")
            .set("OriginalFilename", "sne.exe");
        // sne works without them, so a missing resource compiler must not stop `cargo install`.
        if let Err(err) = resource.compile() {
            println!("cargo:warning=sne.exe was built without version information: {err}");
        }
    }
}
