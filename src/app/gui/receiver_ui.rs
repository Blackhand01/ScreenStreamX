// src/app/gui/receiver_ui.rs
use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use super::app_main::MyApp;

// Funzione per il rendering dell'input per l'indirizzo del Caster
pub fn render_receiver_address_input(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Caster address:").strong());
            ui.text_edit_singleline(&mut app.get_address());
        });
    });
    ui.add_space(10.0);
}

// Funzione per il rendering del pulsante di avvio/arresto della registrazione
pub fn render_recording_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let button_label = if app.is_recording() {
        "Stop Recording"
    } else {
        "Start Recording"
    };

    let button_color = if app.is_recording() {
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
        handle_recording_button_click(app);
    }
}

// Gestione del clic sul pulsante di avvio/arresto della registrazione
fn handle_recording_button_click(app: &mut MyApp) {
    if app.is_recording() {
        stop_recording(app);
    } else {
        start_recording(app);
    }
}

// Funzione per avviare la registrazione della trasmissione
fn start_recording(app: &mut MyApp) {
    println!("Starting recording...");
    app.set_recording(true);

    let recording_flag = Arc::new(Mutex::new(true));
    let recording_flag_clone = Arc::clone(&recording_flag);

    let (tx, rx) = mpsc::channel();
    app.set_stop_tx(Some(tx));

    thread::spawn(move || {
        println!("Recording thread started");
        while *recording_flag_clone.lock().unwrap() {
            if rx.try_recv().is_ok() {
                println!("Received stop signal for recording, stopping...");
                *recording_flag_clone.lock().unwrap() = false;
                break;
            }

            // Implementa qui la logica di ricezione e registrazione del frame
            // Ad esempio, il frame può essere salvato come immagine
            // frame.save("received_frame.png").expect("Failed to save image");
        }
        println!("Recording thread exiting");
    });
}

// Funzione per fermare la registrazione della trasmissione
fn stop_recording(app: &mut MyApp) {
    println!("Stopping recording...");
    app.set_recording(false);

    if let Some(tx) = app.get_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal: {:?}", e);
        }
    }
}
