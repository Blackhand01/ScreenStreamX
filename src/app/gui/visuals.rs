// src/app/gui/visuals.rs
use eframe::egui;
use super::app_main::MyApp;
use super::components::{render_caster_ui, render_receiver_ui};

/// Configura l'aspetto visivo dell'interfaccia utente
pub fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // Configurazione delle ombre delle finestre
    visuals.window_shadow.blur = 10.0;
    visuals.window_shadow.offset = egui::Vec2::new(2.0, 2.0);
    
    // Configurazione degli angoli arrotondati delle finestre
    visuals.window_rounding = egui::Rounding::same(10.0);
    
    // Impostazione del colore del testo
    visuals.override_text_color = Some(egui::Color32::from_gray(220));
    
    // Configurazione dei colori di sfondo dei widget in base al loro stato
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 50, 50);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 60);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 80, 80);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(70, 70, 70);
    
    // Applicazione delle configurazioni visive al contesto
    ctx.set_visuals(visuals);
}

/// Rendering del pannello centrale con i controlli dell'interfaccia utente
pub fn central_panel(ctx: &egui::Context, app: &mut MyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Centrare verticalmente e orizzontalmente il contenuto
        ui.vertical_centered(|ui| {
            // Aggiungi un'intestazione all'interfaccia utente
            ui.heading(egui::RichText::new("ScreenStreamX").strong());

            // Aggiungi uno spazio tra l'intestazione e i pulsanti
            ui.add_space(20.0);

            // Rendering dei pulsanti Caster e Receiver
            render_mode_selection_buttons(ui, app);

            // Aggiungi uno spazio tra i pulsanti e la sezione successiva
            ui.add_space(20.0);

            // Rendering dell'interfaccia utente in base alla modalità selezionata
            if app.is_caster() {
                render_caster_ui(ui, app);
            } else {
                render_receiver_ui(ui, app);
            }
        });
    });
}

/// Rendering dei pulsanti per selezionare la modalità Caster o Receiver
fn render_mode_selection_buttons(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.horizontal(|ui| {
        // Pulsante "Caster"
        if ui.add_sized(
            [150.0, 50.0],
            egui::Button::new(
                egui::RichText::new("Caster")
                    .color(egui::Color32::WHITE)
                    .strong()
            ).fill(egui::Color32::from_rgb(0, 102, 204))
        ).clicked() {
            app.set_caster(true);
        }

        // Pulsante "Receiver"
        if ui.add_sized(
            [150.0, 50.0],
            egui::Button::new(
                egui::RichText::new("Receiver")
                    .color(egui::Color32::WHITE)
                    .strong()
            ).fill(egui::Color32::from_rgb(0, 204, 102))
        ).clicked() {
            app.set_caster(false);
        }
    });
}
