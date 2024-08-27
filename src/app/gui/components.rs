use eframe::egui;
use crate::app::gui::caster_ui;
use crate::app::gui::receiver_ui;
use super::app_main::MyApp;

/// Funzione principale per il rendering della UI del Caster
pub fn render_caster_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Caster mode selected").strong());
        ui.add_space(10.0);

        caster_ui::render_capture_area_button(ui, app);
        caster_ui::render_annotation_toggle_button(ui, app);
        caster_ui::render_multi_monitor_support_button(ui, app);
        caster_ui::render_broadcast_button(ui, app); // Pulsante per gestire il broadcasting
        caster_ui::render_record_button(ui, app); // Pulsante per gestire la registrazione
    });
}

/// Funzione principale per il rendering della UI del Receiver
pub fn render_receiver_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.vertical_centered(|ui| {
        ui.label("Receiver mode selected");
        
        ui.add_space(10.0);  // Spazio tra il titolo e il pulsante
        
        if ui.add_sized([200.0, 50.0], egui::Button::new("Start Receiving").fill(egui::Color32::RED)).clicked() {
            // Logica per avviare la ricezione
        }
    });
}
