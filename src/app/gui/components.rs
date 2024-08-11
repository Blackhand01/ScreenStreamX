use eframe::egui;
use crate::app::gui::caster_ui;
use crate::app::gui::receiver_ui;
use super::app_main::MyApp;

// Funzione principale per il rendering della UI del Caster
pub fn render_caster_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        println!("Rendering Caster UI...");
        ui.label(egui::RichText::new("Caster mode selected").strong());
        ui.add_space(10.0);

        caster_ui::render_capture_area_button(ui, app);
        caster_ui::render_annotation_toggle_button(ui, app);
        caster_ui::render_multi_monitor_support_button(ui);
        caster_ui::render_broadcast_button(ui, app);
    });
}

// Funzione principale per il rendering della UI del Receiver
pub fn render_receiver_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        println!("Rendering Receiver UI...");
        ui.label(egui::RichText::new("Receiver mode selected").strong());
        ui.add_space(10.0);

        receiver_ui::render_receiver_address_input(ui, app);
        receiver_ui::render_recording_button(ui, app);
    });
}
