use eframe::egui;
use crate::app::gui::caster_ui;
use crate::app::gui::receiver_ui;
use super::app_main::MyApp;
// Funzione principale per il rendering della UI del Caster
pub fn render_caster_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Caster mode selected").strong());
        ui.add_space(10.0);

        caster_ui::render_capture_area_button(ui, app);
        caster_ui::render_multi_monitor_support_button(ui, app);
        caster_ui::render_broadcast_button(ui, app); // Pulsante per gestire il broadcasting
        caster_ui::render_record_button(ui, app); // Pulsante per gestire la registrazione

        ui.separator();
        if ui.button("Open Preview Window").clicked() {
            app.ui_state.set_showing_caster_preview_window(true);
        }
        caster_ui::render_annotation_toolbar(ui, app);

        ui.add_space(20.0);
        ui.separator();

        //caster_ui::render_caster_preview(ui, app); // Nuovo pannello per l'anteprima del casting
    });
}

/// Funzione principale per il rendering della UI del Receiver
pub fn render_receiver_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Receiver mode selected").strong());
        ui.add_space(10.0);

        receiver_ui::render_receiver_address_input(ui, app);
        receiver_ui::render_receiving_button(ui, app);
    });
}

