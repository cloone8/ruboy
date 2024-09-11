use eframe::egui::Ui;
use rfd::FileDialog;

use crate::RuboyApp;

#[derive(Debug, Default)]
pub struct RomMenuData {}

pub fn draw_menu(app: &mut RuboyApp, ui: &mut Ui) {
    if ui.button("Open...").clicked() {
        if let Some(path) = FileDialog::new().set_title("Pick a ROM").pick_file() {
            app.rom = Some(path);
            app.ruboy = None;
            ui.close_menu();
        }
    }
}
