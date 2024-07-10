use eframe::egui::Ui;

#[derive(Debug, Default)]
pub struct WindowMenuData {}

pub fn draw_menu(data: &mut WindowMenuData, ui: &mut Ui) {
    ui.checkbox(&mut true, "Test checkbox");
}
