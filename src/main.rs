mod check;
mod parse;
mod ui;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 720.0])
            .with_min_inner_size([1024.0, 720.0]),
            // TODO add icon
        ..Default::default()
    };

    eframe::run_native(
        "Deduct",
        native_options,
        Box::new(|cc| Box::new(ui::Deduct::new(cc))),
    ).unwrap();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(ui::Deduct::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}