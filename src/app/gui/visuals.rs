use eframe::egui;
use super::app_main::MyApp;
use super::components::{render_caster_ui, render_receiver_ui};
use crate::app::capture::CaptureArea;

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

/// Funzione per configurare l'interfaccia di selezione dell'area
pub fn capture_area_panel(ctx: &egui::Context, app: &mut MyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("Select the area by dragging the mouse")
                .color(egui::Color32::YELLOW)
                .strong());
        });

        let response = ui.allocate_rect(ui.max_rect(), egui::Sense::click_and_drag());

        // Gestione della selezione dell'area
        if let Some(capture_area) = app.get_capture_area_mut() {
            if response.drag_started() {
                capture_area.drag_state.start_pos = Some(response.interact_pointer_pos().unwrap());
                println!("Drag started at {:?}", capture_area.drag_state.start_pos);
            }

            if response.hovered() {
                println!("Mouse hovering over selectable area");
            }

            if response.dragged() {
                capture_area.drag_state.end_pos = Some(response.interact_pointer_pos().unwrap());
            }

            // Se è stata rilasciata l'area, mantieni il rettangolo sullo schermo
            if let (Some(start), Some(end)) = (&capture_area.drag_state.start_pos, &capture_area.drag_state.end_pos) {
                let rect = egui::Rect::from_two_pos(*start, *end);
                ui.painter().rect_stroke(rect, 0.0, (2.0, egui::Color32::RED));

                if response.drag_stopped() {
                    capture_area.x = start.x as usize;
                    capture_area.y = start.y as usize;
                    capture_area.width = (end.x - start.x).abs() as usize;
                    capture_area.height = (end.y - start.y).abs() as usize;
                    println!("Area selected: {:?}", capture_area);
                }
            }
        }

        // Aggiungi il pannello dei pulsanti in basso
        egui::TopBottomPanel::bottom("selection_panel").show(ctx, |ui| {
            ui.horizontal(|ui| { // Disposizione in orizzontale

                // Mostra il pulsante "Fullscreen"
                if ui.add_sized(
                    [200.0, 40.0],
                    egui::Button::new(
                        egui::RichText::new("Fullscreen")
                            .color(egui::Color32::WHITE)
                            .strong()
                    ).fill(egui::Color32::from_rgb(0, 153, 0)) // Verde per indicare la selezione dell'intero schermo
                ).clicked() {
                    let display = scrap::Display::primary().unwrap();
                    let fullscreen_area = CaptureArea::new(0, 0, display.width(), display.height());
                    app.set_capture_area(Some(fullscreen_area));
                    app.set_selecting_area(false); // Esci dalla modalità di selezione
                    println!("Fullscreen selected");
                }

                ui.add_space(10.0);

                // Verifica la validità dell'area selezionata
                let is_valid_selection = app.get_capture_area().map_or(false, |area| area.is_valid());

                // Mostra il pulsante "Confirm Selection" se l'area è stata selezionata
                if is_valid_selection {
                    if ui.add_sized(
                        [200.0, 40.0],
                        egui::Button::new(
                            egui::RichText::new("Confirm Selection")
                                .color(egui::Color32::WHITE)
                                .strong()
                        ).fill(egui::Color32::from_rgb(51, 153, 255)) // Blu per confermare
                    ).clicked() {
                        app.set_selecting_area(false); // Esci dalla modalità di selezione
                        println!("Area confirmed: {:?}", app.get_capture_area());
                    }
                }

                ui.add_space(10.0);

                // Mostra il pulsante "Cancel Selection"
                if ui.add_sized(
                    [200.0, 40.0],
                    egui::Button::new(
                        egui::RichText::new("Cancel Selection")
                            .color(egui::Color32::WHITE)
                            .strong()
                    ).fill(egui::Color32::from_rgb(204, 51, 51)) // Rosso per cancellare
                ).clicked() {
                    app.set_show_confirmation_dialog(true);
                }
            });
        });
    });

    // Mostra la finestra di conferma quando si tenta di annullare
    if app.show_confirmation_dialog() {
        egui::Window::new("Are you sure you want to cancel?")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("No").clicked() {
                        app.set_show_confirmation_dialog(false);
                    }

                    if ui.button("Yes").clicked() {
                        if let Some(capture_area) = app.get_capture_area_mut() {
                            capture_area.drag_state.reset();
                        }
                        app.set_selecting_area(false);
                        app.set_show_confirmation_dialog(false);
                        println!("Selection cancelled");
                    }
                });
            });
    }
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