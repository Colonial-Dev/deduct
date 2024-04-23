mod check;
mod parse;
mod ui;

fn main() {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 720.0])
            .with_min_inner_size([720.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Deduct",
        native_options,
        Box::new(|cc| Box::new(ui::Deduct::new(cc))),
    ).unwrap();
}