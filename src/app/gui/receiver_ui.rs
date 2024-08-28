use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use crate::app::network::start_client;
use crate::app::capture::ScreenCapture;
use super::app_main::MyApp;

pub fn render_receiver_address_input(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Caster address:").strong());
            ui.text_edit_singleline(&mut app.network.get_address());
        });
    });
    ui.add_space(10.0);
}

pub fn render_receiving_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let button_label = if app.flags.is_receiving() {
        "Stop Receiving"
    } else {
        "Start Receiving"
    };

    let button_color = if app.flags.is_receiving() {
        egui::Color32::from_rgb(102, 0, 0)
    } else {
        egui::Color32::from_rgb(204, 51, 51)
    };

    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new(button_label)
                .color(egui::Color32::WHITE)
                .strong()
        ).fill(button_color)
    ).clicked() {
        handle_receiving_button_click(app);
    }
}

fn handle_receiving_button_click(app: &mut MyApp) {
    if app.flags.is_receiving() {
        stop_receiving(app);
    } else {
        start_receiving(app);
    }
}

fn start_receiving(app: &mut MyApp) {
    println!("Starting receiving...");
    app.flags.set_receiving(true);

    let receiving_flag = Arc::new(Mutex::new(true));
    let receiving_flag_clone = Arc::clone(&receiving_flag);

    let (frame_tx, frame_rx) = mpsc::channel();
    app.set_frame_receiver(Some(frame_rx));

    let receiver_address = app.network.get_address().to_string();

    thread::spawn(move || {
        start_client(&receiver_address, receiving_flag_clone, move |frame: ScreenCapture| {
            if frame_tx.send(frame).is_err() {
                return;
            }
        });
    });

    println!("Receiving thread started");
}

fn stop_receiving(app: &mut MyApp) {
    println!("Stopping receiving...");
    app.flags.set_receiving(false);

    if let Some(tx) = app.network.get_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal: {:?}", e);
        }
    }
    app.set_frame_receiver(None);
}
