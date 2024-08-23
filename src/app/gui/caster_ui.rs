// src/app/gui/caster_ui.rs

use eframe::egui;
use crate::app::capture::ScreenCapturer;
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

// Funzione per il rendering del pulsante di selezione dell'area di cattura
pub fn render_capture_area_button(ui: &mut egui::Ui, app: &mut MyApp) {
    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new("Select Capture Area")
                .color(egui::Color32::WHITE)
                .strong()
        ).fill(egui::Color32::from_rgb(255, 153, 102))
    ).clicked() {
        println!("Select Capture Area clicked");
        app.set_selecting_area(true);
    }
    ui.add_space(10.0);
}

// Funzione per il rendering del pulsante di avvio/arresto della trasmissione
pub fn render_broadcast_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let button_label = if app.is_recording() {
        "Stop Broadcasting"
    } else {
        "Start Broadcasting"
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
        handle_broadcast_button_click(app);
    }
}

// Gestione del clic sul pulsante di avvio/arresto della trasmissione
fn handle_broadcast_button_click(app: &mut MyApp) {
    if app.is_recording() {
        stop_broadcast(app);
    } else {
        start_broadcast(app);
    }
}

// Funzione per avviare la trasmissione dello schermo
fn start_broadcast(app: &mut MyApp) {
    println!("Starting broadcast...");
    app.set_recording(true);

    let capture_area = app.get_capture_area().cloned().filter(|area| area.is_valid());
    let recording_flag = Arc::new(Mutex::new(true));

    let (tx, rx) = mpsc::channel();
    app.set_stop_tx(Some(tx));

    // Creiamo un listener TCP che ascolta su una specifica porta (es: 8080)
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to address");

    // Ottieni la larghezza e l'altezza dello schermo da catturare
    let (width, height) = match &capture_area {
        Some(area) => (area.width, area.height),
        None => {
            let display = scrap::Display::primary().unwrap();
            (display.width(), display.height())
        }
    };

    // Ensure the "recording" directory exists
    let recording_dir = "recording";
    fs::create_dir_all(recording_dir).expect("Failed to create recording directory");

    thread::spawn(move || {
        println!("Broadcast thread started");

        // Lancia `ffmpeg` per salvare lo stream video in un file MP4 nella cartella "recording"
        let mut child = Command::new("ffmpeg")
            .args(&[
                "-f", "rawvideo",
                "-pixel_format", "bgra",
                "-video_size", &format!("{}x{}", width, height),
                "-framerate", &TARGET_FRAMERATE.to_string(),
                "-i", "-", // Legge dallo stdin
                "-vf", "format=yuv420p", // Filtra per convertire in un formato compatibile
                "-codec:v", "libx264", // Codec video
                "-preset", "ultrafast",  // Preset di codifica più compatibile
                "-profile:v", "high",  // Profilo H.264 compatibile con QuickTime
                "-level", "4.0",       // Livello compatibile con QuickTime
                "-pix_fmt", "yuv420p", // Formato pixel compatibile
                "-y", &format!("{}/broadcast_video.mp4", recording_dir) // Nome del file di output con percorso
            ])
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to start ffmpeg process");

        let mut out = child.stdin.take().expect("Failed to open stdin");

        let mut screen_capturer = ScreenCapturer::new(capture_area);

        println!("Waiting for receiver to connect...");

        // Aspettiamo una connessione da parte di un receiver
        if let Ok((mut stream, _addr)) = listener.accept() {
            println!("Receiver connected, starting broadcast...");

            while *recording_flag.lock().unwrap() {
                if rx.try_recv().is_ok() {
                    println!("Received stop signal, stopping broadcast...");
                    *recording_flag.lock().unwrap() = false;
                    break;
                }

                // Sincronizzazione del framerate
                let start_time = Instant::now();

                if let Some(frame) = screen_capturer.capture_frame() {
                    println!("Captured a frame, sending...");

                    // Serializzare il frame
                    let serialized_frame = bincode::serialize(&frame).expect("Failed to serialize frame");

                    // Inviare la lunghezza del frame seguita dal frame stesso
                    let length = serialized_frame.len() as u32;
                    let length_bytes = length.to_be_bytes(); // Converti la lunghezza in big-endian

                    // Invia la lunghezza del frame
                    if let Err(e) = stream.write_all(&length_bytes) {
                        println!("Failed to send frame length: {:?}", e);
                        break;
                    }

                    // Invia i dati del frame
                    if let Err(e) = stream.write_all(&serialized_frame) {
                        println!("Failed to send frame data: {:?}", e);
                        break;
                    }

                    // Scrivi il frame nel file video usando `ffmpeg`
                    let frame_imgbuffer = frame.into_image_buffer();
                    let stride = frame_imgbuffer.len() / height;
                    let rowlen = 4 * width;
                    for row in frame_imgbuffer.chunks(stride) {
                        let row = &row[..rowlen];
                        out.write_all(row).expect("Failed to write frame to ffmpeg");
                    }
                } else {
                    println!("Failed to capture frame.");
                }

                
                let elapsed = start_time.elapsed();
                if elapsed < FRAME_DURATION {
                    thread::sleep(FRAME_DURATION - elapsed);
                } else {
                    println!("Warning: Frame took too long to capture, possible overload");
                }
            }

            println!("Broadcast thread exiting");
        } else {
            println!("Failed to accept connection from receiver.");
        }
    });
}

// Funzione per fermare la trasmissione dello schermo
fn stop_broadcast(app: &mut MyApp) {
    println!("Stopping broadcast...");
    app.set_recording(false);

    if let Some(tx) = app.get_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal: {:?}", e);
        }
    }
}

// Funzione per il rendering del pulsante di attivazione/disattivazione degli strumenti di annotazione
pub fn render_annotation_toggle_button(ui: &mut egui::Ui, app: &mut MyApp) {
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
                .strong()
        ).fill(egui::Color32::from_rgb(153, 0, 153))
    ).clicked() {
        println!("Toggle Annotation Tools clicked");
        toggle_annotation_tools(app);
    }
    ui.add_space(10.0);
}

// Funzione per il rendering del pulsante di supporto multi-monitor
pub fn render_multi_monitor_support_button(ui: &mut egui::Ui) {
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
}
