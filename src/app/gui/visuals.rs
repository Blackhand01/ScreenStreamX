use eframe::egui;
use super::app_main::{MyApp, Theme};
use super::components::{render_caster_ui, render_receiver_ui};
use super::receiver_ui::render_receiving_button;
use crate::app::capture::CaptureArea;
use crate::utils::multi_monitor::get_available_monitors;


/// Pannello di selezione del monitor con un pulsante "Indietro" migliorato e pulsanti accattivanti per la selezione del monitor.
pub fn monitor_selection_panel(ctx: &egui::Context, app: &mut MyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Pulsante "Indietro" in alto a sinistra
        egui::TopBottomPanel::top("back_button_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add_sized(
                    [120.0, 45.0],  // Dimensione aumentata per maggiore visibilità
                    egui::Button::new(
                        egui::RichText::new("⬅ Back")
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgba_unmultiplied(255, 140, 0, 200))  // Gradiente arancione
                    .rounding(10.0)  // Arrotondamento agli angoli
                    .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE)),  // Contorno bianco
                ).clicked() {
                    app.ui_state.set_showing_monitor_selection(false);  // Torna al menu principale
                }
            });
        });

        // Menu di selezione dei monitor al centro
        ui.vertical_centered(|ui| {
            ui.heading(egui::RichText::new("🖥️ Select Monitor").strong().size(24.0));
            ui.add_space(20.0);

            let monitors = get_available_monitors();
            for (index, display) in monitors.iter().enumerate() {
                let monitor_name = format!("Monitor {}: {}x{}", index + 1, display.width(), display.height());

                if ui.add_sized(
                    [300.0, 60.0],  // Dimensione aumentata per un impatto visivo maggiore
                    egui::Button::new(
                        egui::RichText::new(&monitor_name)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgba_unmultiplied(70, 130, 180, 220))  // Colore blu accattivante
                    .rounding(10.0)  // Angoli arrotondati
                    .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE)),  // Contorno bianco
                ).clicked() {
                    app.capture.set_capture_area(Some(CaptureArea::new(0, 0, display.width(), display.height())));
                    app.ui_state.set_showing_monitor_selection(false);  // Torna al menu principale dopo la selezione
                }

                ui.add_space(15.0);  // Spazio tra i pulsanti
            }
        });
    });
}

/// Configura l'aspetto visivo dell'interfaccia utente
pub fn configure_visuals(ctx: &egui::Context, app: &MyApp) {
    let visuals = match app.user_settings.get_theme() {
        Theme::Light => egui::Visuals::light(),
        Theme::Dark => egui::Visuals::dark(),
    };

    ctx.set_visuals(visuals);
}


/// Pannello di selezione dell'area di cattura
pub fn capture_area_panel(ctx: &egui::Context, app: &mut MyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        render_capture_area_instructions(ui);
        handle_area_selection(ui, app);
        render_selection_panel(ctx, app);
    });

    handle_selection_confirmation_dialog(ctx, app);

    // Aggiorna il flag `is_fullscreen` in base all'area selezionata
    if let Some(capture_area) = app.capture.get_capture_area() {
        let display = scrap::Display::primary().unwrap();
        app.capture.set_fullscreen(
            capture_area.x == 0
                && capture_area.y == 0
                && capture_area.width == display.width()
                && capture_area.height == display.height(),
        );
    } else {
        app.capture.set_fullscreen(true);
    }
}


/// Mostra le istruzioni per la selezione dell'area
fn render_capture_area_instructions(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("🎯 Select the area by dragging the mouse")
                .color(egui::Color32::YELLOW)
                .strong(),
        );
    });
}

/// Gestisce la selezione dell'area tramite drag-and-drop
fn handle_area_selection(ui: &mut egui::Ui, app: &mut MyApp) {
    let response = ui.allocate_rect(ui.max_rect(), egui::Sense::click_and_drag());

    if let Some(capture_area) = app.capture.get_capture_area_mut() {
        if response.drag_started() {
            capture_area.drag_state.start_pos = Some(response.interact_pointer_pos().unwrap());
            println!("Drag started at {:?}", capture_area.drag_state.start_pos);
        }

        if response.dragged() {
            capture_area.drag_state.end_pos = Some(response.interact_pointer_pos().unwrap());
        }

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

    // Una volta rilasciato il primo borrow mutabile, possiamo accedere di nuovo a `app.capture`
    if let Some(capture_area) = app.capture.get_capture_area() {
        let display = scrap::Display::primary().unwrap();
        app.capture.set_fullscreen(
            capture_area.x == 0
                && capture_area.y == 0
                && capture_area.width == display.width()
                && capture_area.height == display.height(),
        );
    }
}


fn render_selection_panel(ctx: &egui::Context, app: &mut MyApp) {
    egui::TopBottomPanel::bottom("selection_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {

            render_fullscreen_button(ui, app);
            ui.add_space(10.0);
            render_confirm_selection_button(ui, app);
            ui.add_space(10.0);
            render_cancel_selection_button(ui, app);
        });
    });
}

/// Mostra il pulsante per selezionare l'intero schermo
fn render_fullscreen_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let is_fullscreen = app.capture.is_fullscreen();

    ui.add_enabled_ui(!is_fullscreen, |ui| {
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new("🖥️ Fullscreen")
                    .color(if is_fullscreen { egui::Color32::GRAY } else { egui::Color32::WHITE })
                    .strong(),
            )
            .fill(if is_fullscreen { egui::Color32::from_rgb(100, 100, 100) } else { egui::Color32::from_rgb(0, 153, 0) }),
        ).clicked() {
            let display = scrap::Display::primary().unwrap();
            let fullscreen_area = CaptureArea::new(0, 0, display.width(), display.height());
            app.capture.set_capture_area(Some(fullscreen_area));
            app.ui_state.set_selecting_area(false); // Esci dalla modalità di selezione
            println!("Fullscreen selected");
        }
    });
}



/// Mostra il pulsante per confermare la selezione dell'area
fn render_confirm_selection_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let is_valid_selection = app.capture.get_capture_area().map_or(false, |area| area.is_valid());

    ui.add_enabled_ui(is_valid_selection, |ui| {
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new("✅ Confirm Selection")
                    .color(if is_valid_selection { egui::Color32::WHITE } else { egui::Color32::GRAY })
                    .strong(),
            )
            .fill(if is_valid_selection { egui::Color32::from_rgb(51, 153, 255) } else { egui::Color32::from_rgb(100, 100, 100) }),
        ).clicked() {
            app.ui_state.set_selecting_area(false); // Esci dalla modalità di selezione
            println!("Area confirmed: {:?}", app.capture.get_capture_area());
        }
    });
}


/// Mostra il pulsante per cancellare la selezione dell'area
fn render_cancel_selection_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let is_selecting_area = app.ui_state.is_selecting_area();

    ui.add_enabled_ui(is_selecting_area, |ui| {
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new("❌ Cancel Selection")
                    .color(if is_selecting_area { egui::Color32::WHITE } else { egui::Color32::GRAY })
                    .strong(),
            )
            .fill(if is_selecting_area { egui::Color32::from_rgb(204, 51, 51) } else { egui::Color32::from_rgb(100, 100, 100) }),
        ).clicked() {
            app.ui_state.set_show_confirmation_dialog(true);
        }
    });
}


/// Gestisce la finestra di conferma quando si tenta di annullare la selezione
fn handle_selection_confirmation_dialog(ctx: &egui::Context, app: &mut MyApp) {
    if app.ui_state.show_confirmation_dialog() {
        egui::Window::new("Are you sure you want to cancel?")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("No").clicked() {
                        app.ui_state.set_show_confirmation_dialog(false);
                    }

                    if ui.button("Yes").clicked() {
                        if let Some(capture_area) = app.capture.get_capture_area_mut() {
                            capture_area.drag_state.reset();
                        }
                        app.ui_state.set_selecting_area(false);
                        app.ui_state.set_show_confirmation_dialog(false);
                        println!("Selection cancelled");
                    }
                });
            });
    }
}

pub fn central_panel(ctx: &egui::Context, app: &mut MyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Barra superiore per i controlli della finestra e il cambio tema
        ui.horizontal(|ui| {

            // Posiziona il pulsante per il cambio tema a destra
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_theme_toggle_button(ui, app);
            });
        });

        ui.add_space(10.0);

        ui.vertical_centered(|ui| {
            // Renderizza l'intestazione dell'app
            render_header(ui);

            ui.add_space(10.0);

            // Mostra il messaggio di stato
            render_status_message(ui, app);

            ui.add_space(20.0);

            // Selezione della modalità e pulsante per mostrare le scorciatoie
            ui.horizontal(|ui| {
                render_mode_selection_buttons(ui, app);
                render_show_shortcuts_button(ui, app);
            });

            ui.add_space(20.0);

            // Renderizza l'interfaccia specifica basata sulla modalità selezionata
            render_mode_ui(ui, app);
        });

        // Renderizza la finestra delle scorciatoie se necessario
        render_shortcuts_window(ctx, app);
    });
}


fn render_show_shortcuts_button(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button("Show Shortcuts").clicked() {
            app.ui_state.set_showing_shortcuts_menu(true);
        }
    });
}

fn render_shortcuts_window(ctx: &egui::Context, app: &mut MyApp) {
    if app.ui_state.is_showing_shortcuts_menu() {
        egui::Window::new("Available Shortcuts")
            .collapsible(false)
            .show(ctx, |ui| {
                render_shortcuts_menu(ui);

                ui.add_space(10.0);
                
                if ui.button("Close").clicked() {
                    app.ui_state.set_showing_shortcuts_menu(false);
                }
            });
    }
}

fn render_shortcuts_menu(ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new("Shortcuts").heading());

        let shortcuts = get_shortcuts();
        for shortcut in shortcuts {
            ui.horizontal(|ui| {
                ui.label(shortcut.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(shortcut.key_combination).monospace());
                });
            });
        }
    });
}

struct Shortcut {
    name: &'static str,
    key_combination: &'static str,
}

fn get_shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut { name: "Start/Pause Broadcast", key_combination: "Ctrl+Shift+B" },
        Shortcut { name: "Start/Stop Recording", key_combination: "Ctrl+Shift+R" },
        Shortcut { name: "Lock/Unlock Screen", key_combination: "Ctrl+Shift+L" },
        Shortcut { name: "Toggle Annotation", key_combination: "Ctrl+Shift+A" },
        Shortcut { name: "Quick Capture Selection", key_combination: "Ctrl+Shift+S" },
        Shortcut { name: "End Session", key_combination: "Ctrl+Shift+Q" },
        Shortcut { name: "Switch Monitor", key_combination: "Ctrl+Shift+M" },
    ]
}



// Nuova funzione per visualizzare il pulsante di cambio tema
fn render_theme_toggle_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let button_label = match app.user_settings.get_theme() {
        Theme::Light => "🌙 Switch to Dark Theme",
        Theme::Dark => "🔆 Switch to Light Theme",
    };

    if ui.add(egui::Button::new(button_label)).clicked() {
        app.toggle_theme();
        configure_visuals(ui.ctx(), app); // Aggiorna l'interfaccia con il nuovo tema
    }
}

// Nuova funzione per visualizzare il messaggio di stato
fn render_status_message(ui: &mut egui::Ui, app: &MyApp) {
    let status = app.get_status_message();
    let color = match status.as_str() {
        "Idle" => egui::Color32::LIGHT_GRAY,
        "Broadcasting" => egui::Color32::GREEN,
        "Recording" => egui::Color32::YELLOW,
        "Receiving" => egui::Color32::BLUE,
        _ => egui::Color32::WHITE, // Per eventuali stati combinati
    };

    ui.label(
        egui::RichText::new(format!("Status: {}", status))
            .color(color)
            .strong(),
    );
}

/// Mostra l'intestazione dell'interfaccia utente
fn render_header(ui: &mut egui::Ui) {
    ui.heading(egui::RichText::new("🖥️ ScreenStreamX").strong());
}

/// Mostra i pulsanti per selezionare la modalità Caster o Receiver
fn render_mode_selection_buttons(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.horizontal(|ui| {
        render_caster_button(ui, app);
        render_receiver_button(ui, app);
    });
}

/// Mostra il pulsante "Caster"
fn render_caster_button(ui: &mut egui::Ui, app: &mut MyApp) {
    if ui.add_sized(
        [150.0, 50.0],
        egui::Button::new(
            egui::RichText::new("🎥 Caster")
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(0, 102, 204)), // Blu per Caster
    ).clicked() {
        app.set_caster(true);
    }
}

/// Mostra il pulsante "Receiver"
fn render_receiver_button(ui: &mut egui::Ui, app: &mut MyApp) {
    if ui.add_sized(
        [150.0, 50.0],
        egui::Button::new(
            egui::RichText::new("📡 Receiver")
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(0, 204, 102)), // Verde per Receiver
    ).clicked() {
        app.set_caster(false);
    }
}

/// Rendering dell'interfaccia utente basato sulla modalità selezionata
// src/app/gui/central_panel.rs

fn render_mode_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    if app.is_caster() {
        render_caster_ui(ui, app);
    } else {
        render_receiver_ui(ui, app);

        if app.flags.is_receiving() {
            ui.separator();
            render_receiving_screen(ui, app);
        }
    }
}


pub fn render_receiving_screen(ui: &mut egui::Ui, app: &mut MyApp) {
    if let Some(ref texture) = app.texture {
        let texture_size = texture.size_vec2();
        let available_size = ui.available_size();

        // Calcola la scala per adattare l'immagine alla finestra disponibile mantenendo le proporzioni
        let scale = (available_size.x / texture_size.x).min(available_size.y / texture_size.y);
        let scaled_size = texture_size * scale;

        // Calcola il rettangolo dell'immagine centrato
        let image_rect = egui::Rect::from_min_size(ui.min_rect().min, scaled_size);

        // Disegna l'immagine ridimensionata nel rettangolo calcolato
        ui.allocate_ui_at_rect(image_rect, |ui| {
            ui.image(texture);
        });
    } else {
        ui.label("No image received yet.");
    }
}


pub fn render_screen_lock_overlay(ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Disegna un rettangolo nero che copre l'intero schermo
        ui.painter().rect_filled(ui.max_rect(), 0.0, egui::Color32::BLACK);

        // Mostra un messaggio al centro dello schermo
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("Screen is Locked")
                    .color(egui::Color32::WHITE)
                    .strong(),
            );
        });
    });
}

