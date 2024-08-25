// src/app/gui/caster_ui.rs
use eframe::egui;
use crate::app::capture::{CaptureArea, ScreenCapturer};
use crate::utils::annotations::toggle_annotation_tools;
use crate::utils::multi_monitor::multi_monitor_support;
use std::sync::{mpsc, Arc, Mutex};
use std::{thread, fs};
use std::process::{Command, Stdio};
use std::io::Write;
use std::time::{Duration, Instant};
use super::app_main::MyApp;
use std::net::TcpListener;

const TARGET_FRAMERATE: u64 = 10; // Riduci il framerate target per evitare velocizzazioni
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FRAMERATE as u64);


const TARGET_FRAMERATE: u64 = 10; // Riduci il framerate target per evitare sovraccarichi
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FRAMERATE as u64);

/// Funzione per il rendering del pulsante di selezione dell'area di cattura
pub fn render_capture_area_button(ui: &mut egui::Ui, app: &mut MyApp) {
    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new("Select Capture Area")
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(255, 153, 102)), // Arancione per indicare la selezione dell'area
    ).clicked() {
        println!("Select Capture Area clicked");
        app.set_selecting_area(true); // Passa alla modalità di selezione dell'area
    }
    ui.add_space(10.0);
}



// Funzione per il rendering del pulsante di avvio/arresto della trasmissione
pub fn render_broadcast_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let (button_label, button_color) = if app.is_recording() {
        ("Stop Broadcasting", egui::Color32::from_rgb(102, 0, 0)) // Rosso per indicare la trasmissione attiva
    } else {
        ("Start Broadcasting", egui::Color32::from_rgb(204, 51, 51)) // Rosso più chiaro per avviare la trasmissione
    };

    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new(button_label)
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(button_color),
    ).clicked() {
        handle_broadcast_button_click(app);
    }
    ui.add_space(10.0);
}

/// Gestione del clic sul pulsante di avvio/arresto della trasmissione
fn handle_broadcast_button_click(app: &mut MyApp) {
    if app.is_recording() {
        stop_broadcast(app); // Ferma la trasmissione se è in corso
    } else {
        start_broadcast(app); // Avvia la trasmissione
    }
}

/// Funzione per avviare la trasmissione dello schermo
fn start_broadcast(app: &mut MyApp) {
    println!("Starting broadcast...");
    app.set_recording(true);

    let capture_area = app.get_capture_area().cloned().filter(|area| area.is_valid());
    let recording_flag = Arc::new(Mutex::new(true));

    let (tx, rx) = mpsc::channel();
    app.set_stop_tx(Some(tx));

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
                frame.save("broadcast_frame.png").expect("Failed to save image");
            }
        }
        println!("Broadcast thread exiting");
    });
}

/// Funzione per fermare la trasmissione dello schermo
fn stop_broadcast(app: &mut MyApp) {
    println!("Stopping broadcast...");
    app.set_recording(false);

    if let Some(tx) = app.get_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal: {:?}", e);
        }
    }
}

/// Funzione per il rendering del pulsante di attivazione/disattivazione degli strumenti di annotazione
pub fn  render_annotation_toggle_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let button_label = if app.is_annotation_tools_active() {
        "Disable Annotation Tools"
    } else {
        "Enable Annotation Tools"
    };

    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new(button_label)
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(153, 0, 153)), // Viola per indicare le annotazioni
    ).clicked() {
        println!("Toggle Annotation Tools clicked");
        toggle_annotation_tools(app);
    }
    ui.add_space(10.0);
}

/// Funzione per il rendering del pulsante di supporto multi-monitor
pub fn render_multi_monitor_support_button(ui: &mut egui::Ui) {
    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new("Multi-Monitor Support")
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(102, 204, 255)), // Azzurro per il supporto multi-monitor
    ).clicked() {
        println!("Multi-Monitor Support clicked");
        multi_monitor_support();
    }
    ui.add_space(10.0);
}
