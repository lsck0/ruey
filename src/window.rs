use eframe::{NativeOptions, egui::ViewportBuilder};

pub fn get_window_options() -> NativeOptions {
    let window_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Ruey")
            .with_inner_size([1080.0, 720.0])
            .with_min_inner_size([1080.0, 720.0]),
        ..Default::default()
    };

    return window_options;
}
