use eframe::egui;

use super::app_main::MyApp;
use super::components::{render_caster_ui, render_receiver_ui};

pub fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_shadow.blur = 10.0;
    visuals.window_shadow.offset = egui::Vec2::new(2.0, 2.0);
    visuals.window_rounding = egui::Rounding::same(10.0);
    visuals.override_text_color = Some(egui::Color32::from_gray(220));
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 50, 50);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 60);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 80, 80);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(70, 70, 70);
    ctx.set_visuals(visuals);
}

pub fn central_panel(ctx: &egui::Context, app: &mut MyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Centrare verticalmente e orizzontalmente il contenuto
        ui.vertical_centered(|ui| {
            // Aggiungi un'intestazione all'interfaccia utente
            ui.heading(egui::RichText::new("ScreenStreamX").strong());

            // Aggiungi uno spazio tra l'intestazione e i pulsanti
            ui.add_space(20.0);

            // Crea un layout orizzontale per i pulsanti Caster e Receiver
            ui.horizontal(|ui| {
                // Crea un pulsante "Caster" e verifica se è stato cliccato
                if ui.add_sized(
                    [150.0, 50.0],
                    egui::Button::new(
                        egui::RichText::new("Caster")
                            .color(egui::Color32::WHITE)
                            .strong()
                    ).fill(egui::Color32::from_rgb(0, 102, 204))
                ).clicked() {
                    // Se il pulsante è stato cliccato, imposta is_caster su true
                    app.set_caster(true);
                }
                // Crea un pulsante "Receiver" e verifica se è stato cliccato
                if ui.add_sized(
                    [150.0, 50.0],
                    egui::Button::new(
                        egui::RichText::new("Receiver")
                            .color(egui::Color32::WHITE)
                            .strong()
                    ).fill(egui::Color32::from_rgb(0, 204, 102))
                ).clicked() {
                    // Se il pulsante è stato cliccato, imposta is_caster su false
                    app.set_caster(false);
                }
            });

            // Aggiungi uno spazio tra i pulsanti e la sezione successiva
            ui.add_space(20.0);

            // Verifica se la modalità caster è attiva
            if app.is_caster() {
                render_caster_ui(ui, app);
            } else {
                render_receiver_ui(ui, app);
            }
        });
    });
}
