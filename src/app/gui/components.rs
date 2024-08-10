use eframe::egui;
use super::app_main::MyApp;
use crate::app::capture::{ScreenCapturer, CaptureArea};
use crate::utils::annotations::toggle_annotation_tools;
use crate::utils::multi_monitor::multi_monitor_support;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

pub fn render_caster_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        println!("Rendering Caster UI...");
        ui.label(egui::RichText::new("Caster mode selected").strong());
        ui.add_space(10.0);

        // Pulsante per la selezione dell'area di cattura
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new("Select Capture Area")
                    .color(egui::Color32::WHITE)
                    .strong()
            ).fill(egui::Color32::from_rgb(255, 153, 102))
        ).clicked() {
            println!("Select Capture Area clicked");
            app.set_capture_area(Some(CaptureArea {
                x: 100,  // Posizione X
                y: 100,  // Posizione Y
                width: 800,  // Larghezza dell'area di cattura
                height: 600,  // Altezza dell'area di cattura
            }));
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
            ).fill(egui::Color32::from_rgb(153, 0, 153))
        ).clicked() {
            println!("Toggle Annotation Tools clicked");
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
            ).fill(egui::Color32::from_rgb(102, 204, 255))
        ).clicked() {
            println!("Multi-Monitor Support clicked");
            multi_monitor_support();
        }

        ui.add_space(10.0);

        // Pulsante per avviare o fermare la trasmissione dello schermo
        if ui.add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new(if app.is_recording() {
                    "Stop Broadcasting"
                } else {
                    "Start Broadcasting"
                })
                    .color(egui::Color32::WHITE)
                    .strong()
            ).fill(egui::Color32::from_rgb(
                if app.is_recording() { 102 } else { 204 },
                if app.is_recording() { 0 } else { 51 },
                if app.is_recording() { 0 } else { 51 }
            ))
        ).clicked() {
            if app.is_recording() {
                println!("Stopping broadcast...");
                app.set_recording(false);

                // Usa il canale per inviare un segnale di stop al thread
                if let Some(tx) = app.get_stop_tx() {
                    if let Err(e) = tx.send(()) {
                        println!("Failed to send stop signal: {:?}", e);
                    }
                }
            } else {
                println!("Starting broadcast...");
                app.set_recording(true);

                // Clone o copia i dati necessari
                let capture_area = app.get_capture_area().cloned();
                let recording_flag = Arc::new(Mutex::new(true));
                let recording_flag_clone = Arc::clone(&recording_flag);

                let (tx, rx) = mpsc::channel();
                app.set_stop_tx(Some(tx));

                // Avvia un nuovo thread per catturare e trasmettere i frame dello schermo
                thread::spawn(move || {
                    println!("Broadcast thread started");
                    let mut screen_capturer = ScreenCapturer::new(capture_area);

                    while *recording_flag_clone.lock().unwrap() {
                        if rx.try_recv().is_ok() {
                            println!("Received stop signal, stopping broadcast...");
                            *recording_flag_clone.lock().unwrap() = false;
                            break;
                        }

                        if let Some(frame) = screen_capturer.capture_frame() {
                            println!("Captured a frame, saving...");
                            // Qui dovrebbe essere implementata la logica di trasmissione
                            // Al momento viene salvato come screenshot singolo per esempio
                            frame.save("broadcast_frame.png").expect("Failed to save image");
                        }
                    }
                    println!("Broadcast thread exiting");
                });
            }
        }
    });
}

pub fn render_receiver_ui(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        println!("Rendering Receiver UI...");
        ui.label(egui::RichText::new("Receiver mode selected").strong());
        ui.add_space(10.0);

        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Caster address:").strong());
                ui.text_edit_singleline(&mut app.get_address());
            });
        });

        ui.add_space(10.0);

        // Pulsante per avviare o fermare la registrazione della trasmissione
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
            ))
        ).clicked() {
            if app.is_recording() {
                println!("Stopping recording...");
                app.set_recording(false);

                // Usa il canale per inviare un segnale di stop al thread
                if let Some(tx) = app.get_stop_tx() {
                    if let Err(e) = tx.send(()) {
                        println!("Failed to send stop signal: {:?}", e);
                    }
                }
            } else {
                println!("Starting recording...");
                app.set_recording(true);

                // Implementa la logica di registrazione della trasmissione
                let recording_flag = Arc::new(Mutex::new(true));
                let recording_flag_clone = Arc::clone(&recording_flag);

                let (tx, rx) = mpsc::channel();
                app.set_stop_tx(Some(tx));

                // Avvia un nuovo thread per ricevere e registrare la trasmissione
                thread::spawn(move || {
                    println!("Recording thread started");
                    while *recording_flag_clone.lock().unwrap() {
                        if rx.try_recv().is_ok() {
                            println!("Received stop signal for recording, stopping...");
                            *recording_flag_clone.lock().unwrap() = false;
                            break;
                        }

                        // Implementare la logica di ricezione e registrazione del frame
                        // Qui potrebbe essere salvato come immagine per esempio
                        // frame.save("received_frame.png").expect("Failed to save image");
                    }
                    println!("Recording thread exiting");
                });
            }
        }
    });
}
