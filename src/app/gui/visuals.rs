use eframe::egui;
use super::app_main::MyApp;
use super::components::{render_caster_ui, render_receiver_ui};
use crate::app::capture::CaptureArea;

/// Configura l'aspetto visivo dell'interfaccia utente
pub fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    set_window_visuals(&mut visuals);
    set_text_visuals(&mut visuals);
    set_widget_visuals(&mut visuals);

    ctx.set_visuals(visuals);
}

/// Configura le ombre e gli angoli delle finestre
fn set_window_visuals(visuals: &mut egui::Visuals) {
    visuals.window_shadow.blur = 10.0;
    visuals.window_shadow.offset = egui::Vec2::new(2.0, 2.0);
    visuals.window_rounding = egui::Rounding::same(10.0);
}

/// Configura l'aspetto del testo
fn set_text_visuals(visuals: &mut egui::Visuals) {
    visuals.override_text_color = Some(egui::Color32::from_gray(220));
}

/// Configura i colori di sfondo dei widget in base al loro stato
fn set_widget_visuals(visuals: &mut egui::Visuals) {
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 50, 50);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 60);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 80, 80);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(70, 70, 70);
}

/// Configura l'interfaccia di selezione dell'area
pub fn capture_area_panel(ctx: &egui::Context, app: &mut MyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        render_capture_area_instructions(ui);
        handle_area_selection(ui, app);
        render_selection_panel(ctx, app);
    });

    handle_selection_confirmation_dialog(ctx, app);
}

/// Mostra le istruzioni per la selezione dell'area
fn render_capture_area_instructions(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Select the area by dragging the mouse")
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

        if response.hovered() {
            println!("Mouse hovering over selectable area");
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
}

/// Rendering del pannello dei pulsanti di selezione
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
    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new("Fullscreen")
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(0, 153, 0)), // Verde per l'intero schermo
    ).clicked() {
        let display = scrap::Display::primary().unwrap();
        let fullscreen_area = CaptureArea::new(0, 0, display.width(), display.height());
        app.capture.set_capture_area(Some(fullscreen_area));
        app.ui_state.set_selecting_area(false); // Esci dalla modalità di selezione
        println!("Fullscreen selected");
    }
}

/// Mostra il pulsante per confermare la selezione dell'area
fn render_confirm_selection_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let is_valid_selection = app.capture.get_capture_area().map_or(false, |area| area.is_valid());

    if is_valid_selection {
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new("Confirm Selection")
                    .color(egui::Color32::WHITE)
                    .strong(),
            )
            .fill(egui::Color32::from_rgb(51, 153, 255)), // Blu per confermare
        ).clicked() {
            app.ui_state.set_selecting_area(false); // Esci dalla modalità di selezione
            println!("Area confirmed: {:?}", app.capture.get_capture_area());
        }
    }
}

/// Mostra il pulsante per cancellare la selezione dell'area
fn render_cancel_selection_button(ui: &mut egui::Ui, app: &mut MyApp) {
    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new("Cancel Selection")
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(204, 51, 51)), // Rosso per cancellare
    ).clicked() {
        app.ui_state.set_show_confirmation_dialog(true);
    }
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

/// Rendering del pannello centrale con i controlli dell'interfaccia utente
pub fn central_panel(ctx: &egui::Context, app: &mut MyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            render_header(ui);
            ui.add_space(20.0);
            render_mode_selection_buttons(ui, app);
            ui.add_space(20.0);
            render_mode_ui(ui, app);
        });
    });
}

/// Mostra l'intestazione dell'interfaccia utente
fn render_header(ui: &mut egui::Ui) {
    ui.heading(egui::RichText::new("ScreenStreamX").strong());
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
            egui::RichText::new("Caster")
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
            egui::RichText::new("Receiver")
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(0, 204, 102)), // Verde per Receiver
    ).clicked() {
        app.set_caster(false);
    }
}

/// Rendering dell'interfaccia utente basato sulla modalità selezionata
fn render_mode_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    if app.is_caster() {
        render_caster_ui(ui, app);
    } else {
        render_receiver_ui(ui, app);
    }
}
