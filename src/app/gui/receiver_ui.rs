// src/app/gui/receiver_ui.rs

use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use crate::app::network::start_client;
use crate::app::capture::ScreenCapture;
use super::app_main::MyApp;
use minifb::{Window, WindowOptions};


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

// Funzione per il rendering del pulsante di avvio/arresto della ricezione
pub fn render_recording_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let button_label = if app.is_recording() {
        "Stop Receiving"
    } else {
        "Start Receiving"
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

// Gestione del clic sul pulsante di avvio/arresto della ricezione
fn handle_recording_button_click(app: &mut MyApp) {
    if app.is_recording() {
        stop_receiving(app);
    } else {
        start_receiving(app);
    }
}

// Funzione per avviare la ricezione della trasmissione
fn start_receiving(app: &mut MyApp) {
    println!("Starting receiving...");
    app.set_recording(true);

    let recording_flag = Arc::new(Mutex::new(true));
    let recording_flag_clone = Arc::clone(&recording_flag);

    let (tx, rx) = mpsc::channel();
    app.set_stop_tx(Some(tx));

    let (frame_tx, frame_rx) = mpsc::channel();

    // Clonare l'indirizzo prima di passarlo al thread
    let receiver_address = app.get_address().to_string();

    let mut window = Window::new(
        "Receiver Window",
        1440,
        900,
        WindowOptions {
            resize: true,      // Consente il ridimensionamento della finestra
            borderless: false, // Disattiva la modalità senza bordi per avere i controlli standard della finestra (incluso il pulsante di chiusura)
            title: true,       // Mostra il titolo della finestra (incluso il pulsante di chiusura)
            ..WindowOptions::default()
        },
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Avvia il client in un thread separato
    thread::spawn(move || {
        start_client(&receiver_address, recording_flag_clone, move |frame: ScreenCapture| {
            // Invia il frame attraverso il canale al thread principale
            if frame_tx.send(frame).is_err() {
                println!("Failed to send frame to main thread, exiting client thread.");
                return; // Esce dal thread client se l'invio fallisce
            }        
        });
    });

    println!("Receiving thread started");

    // Ciclo principale per aggiornare la finestra
    while *recording_flag.lock().unwrap() && window.is_open() {
        if let Ok(frame) = frame_rx.try_recv() {
            let buffer: Vec<u32> = frame.data.chunks(4).map(|pixel| {
                let r = pixel[0] as u32;
                let g = pixel[1] as u32;
                let b = pixel[2] as u32;
                (r << 16) | (g << 8) | b
            }).collect();

            // Aggiorna la finestra con il buffer ricevuto
            if window.update_with_buffer(&buffer, frame.width as usize, frame.height as usize).is_err() {
                println!("Failed to update window, stopping.");
                break;
            }
        }

        // Gestione dell'arresto della ricezione
        if rx.try_recv().is_ok() {
            println!("Received stop signal, stopping receiving...");
            break;  // Esce dal ciclo
        }
        // Aggiungi un breve sleep per evitare che il ciclo consumi troppa CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
        
    }

    
    println!("Closing window as requested.");
    // Questo assicura che la finestra sia chiusa correttamente
    app.set_recording(false);
    

    println!("Receiving thread exiting");
}



// Funzione per fermare la ricezione della trasmissione
fn stop_receiving(app: &mut MyApp) {
    println!("Stopping receiving...");
    app.set_recording(false);

    if let Some(tx) = app.get_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal: {:?}", e);
        }
    }
}
