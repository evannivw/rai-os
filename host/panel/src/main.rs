mod app;
#[path = "../../shared/model.rs"]
mod model;
#[path = "../../shared/qemu.rs"]
mod qemu;
mod worker;

fn main() -> eframe::Result {
    // A local macOS bundle lives in <repo>/target/*.app. Finder does not start
    // it in the repository directory, so resolve that layout before any threads.
    if let Ok(executable) = std::env::current_exe()
        && let Some(bundle) = executable
            .ancestors()
            .find(|path| path.extension().is_some_and(|extension| extension == "app"))
        && let Some(root) = bundle.parent().and_then(|target| target.parent())
        && let Err(error) = std::env::set_current_dir(root)
    {
        eprintln!("Cannot open repository directory: {error}");
        std::process::exit(1);
    }
    let kernel = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/aarch64-unknown-none-softfloat/debug/rai-os".to_owned());
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 800.0])
            .with_min_inner_size([900.0, 740.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RAI-OS | Device Lab",
        options,
        Box::new(move |cc| Ok(Box::new(app::Panel::new(cc, kernel)))),
    )
}
