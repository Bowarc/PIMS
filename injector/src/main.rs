#[macro_use]
extern crate log;

mod injection;
mod window;
// mod ui;

fn main() {
    let cfg = logger::LoggerConfig::new()
        .set_level(log::LevelFilter::Debug)
        .add_filter("egui", log::LevelFilter::Error)
        .add_filter("egui_glow", log::LevelFilter::Error)
        .add_filter("egui_winit", log::LevelFilter::Error)
        .add_filter("eframe", log::LevelFilter::Error);
    logger::init(cfg, None);

    window::run();
}