// SPDX-License-Identifier: MIT

fn main() {
    println!("cargo:rerun-if-changed=assets/vamender.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon("assets/vamender.ico")
        .set("CompanyName", "VaMender")
        .set(
            "FileDescription",
            "VaMender VAR repair and dependency cleanup",
        )
        .set("InternalName", "vamender")
        .set("LegalCopyright", "Copyright (c) 2026 VaMender contributors")
        .set("OriginalFilename", "vamender.exe")
        .set("ProductName", "VaMender");
    resource
        .compile()
        .expect("failed to compile the VaMender Windows icon and metadata");
}
