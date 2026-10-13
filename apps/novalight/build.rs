//! Windows only: embed the app icon and version info (VERSIONINFO) into `novalight.exe`, so
//! Explorer, the taskbar, the Start menu and Alt-Tab show the lynx.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `NOVALIGHT_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/novalight.ico");
    println!("cargo:rerun-if-env-changed=NOVALIGHT_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/novalight.ico")
        .set("ProductName", "Nova Light")
        .set("FileDescription", "Nova Light photo library and raw developer")
        .set("LegalCopyright", "Copyright (c) the Nova Light authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "novalight.exe")
        .set("InternalName", "novalight");
    if let Err(e) = res.compile() {
        if std::env::var_os("NOVALIGHT_REQUIRE_WINRES").is_some() {
            panic!("embedding Windows resources failed: {e}");
        }
        println!("cargo:warning=novalight.exe built without icon/version resources: {e}");
    }
}
