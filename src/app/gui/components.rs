use eframe::egui;
use super::app_main::MyApp;
use crate::app::capture::select_capture_area;
use crate::utils::annotations::toggle_annotation_tools;
use crate::utils::multi_monitor::multi_monitor_support;

pub fn render_caster_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Caster mode selected").strong());
        ui.add_space(10.0);
        // Pulsante per la selezione dell'area di cattura
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new("Select Capture Area")
                    .color(egui::Color32::WHITE)
                    .strong()
            ).fill(egui::Color32::from_rgb(255, 153, 102)) // Arancione meno intenso
        ).clicked() {
            select_capture_area();
        }
        ui.add_space(10.0);
        // Pulsante per attivare/disattivare gli strumenti di annotazione
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new(if app.is_annotation_tools_active() {
                    "Disable Annotation Tools"
                } else {
                    "Enable Annotation Tools"
                })
                    .color(egui::Color32::WHITE)
                    .strong()
            ).fill(egui::Color32::from_rgb(153, 0, 153)) // Viola meno intenso
        ).clicked() {
            toggle_annotation_tools(app);
        }
        ui.add_space(10.0);
        // Pulsante per supportare più monitor
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new("Multi-Monitor Support")
                    .color(egui::Color32::WHITE)
                    .strong()
            ).fill(egui::Color32::from_rgb(102, 204, 255)) // Azzurro meno intenso
        ).clicked() {
            multi_monitor_support();
        }
    });
}

pub fn render_receiver_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Receiver mode selected").strong());
        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Caster address:").strong());
                ui.text_edit_singleline(&mut app.get_address());
            });
        });
        ui.add_space(10.0);
        // Pulsante per avviare la registrazione del contenuto ricevuto
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new(if app.is_recording() {
                    "Stop Recording"
                } else {
                    "Start Recording"
                })
                    .color(egui::Color32::WHITE)
                    .strong()
            ).fill(egui::Color32::from_rgb(
                if app.is_recording() { 102 } else { 204 },
                if app.is_recording() { 0 } else { 51 },
                if app.is_recording() { 0 } else { 51 }
            )) // Rosso meno intenso
        ).clicked() {
            app.set_recording(!app.is_recording());
        }
    });
}
