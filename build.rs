fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.set("FileVersion", "2.1.2.0");
        res.set("ProductVersion", "2.1.2.0");
        res.set("ProductName", "PreventSleep");
        res.set("FileDescription", "PreventSleep");
        if let Err(e) = res.compile() {
            eprintln!("winres: {e}");
        }
    }
}
