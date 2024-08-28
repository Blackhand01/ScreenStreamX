// src/app/gui/caster_ui.rs

use eframe::egui;
use crate::app::capture::{CaptureArea, ScreenCapturer};
use std::sync::{mpsc, Arc, Mutex};
use std::{thread, fs};
use std::process::{Command, Stdio};
use std::io::Write;
use std::time::{Duration, Instant};
use super::app_main::MyApp;
use std::net::{TcpListener, TcpStream, Shutdown};
use lazy_static::lazy_static;

const TARGET_FRAMERATE: u64 = 10; // Framerate target
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FRAMERATE as u64);

lazy_static! {
    static ref RECEIVERS: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    static ref LISTENER: Arc<Mutex<Option<TcpListener>>> = Arc::new(Mutex::new(None));
}

/// Funzione per il rendering del pulsante di selezione dell'area di cattura
pub fn render_capture_area_button(ui: &mut egui::Ui, app: &mut MyApp) {
    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new("Select Capture Area")
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(255, 153, 102)), // Colore arancione per la selezione dell'area
    ).clicked() {
        println!("Select Capture Area clicked");
        app.ui_state.set_selecting_area(true); // Passa alla modalità di selezione dell'area
    }
    ui.add_space(10.0);
}

/// Funzione per il rendering del pulsante di avvio/arresto della trasmissione
pub fn render_broadcast_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let (button_label, button_color) = if app.flags.is_broadcasting() {
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

/// Funzione per il rendering del pulsante di avvio/arresto della registrazione
pub fn render_record_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let (button_label, button_color) = if app.flags.is_recording() {
        ("Stop Recording", egui::Color32::from_rgb(102, 0, 0)) // Rosso per indicare la registrazione attiva
    } else {
        ("Start Recording", egui::Color32::from_rgb(204, 51, 51)) // Rosso più chiaro per avviare la registrazione
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
        handle_record_button_click(app);
    }
    ui.add_space(10.0);
}

/// Gestione del clic sul pulsante di avvio/arresto della trasmissione
fn handle_broadcast_button_click(app: &mut MyApp) {
    if app.flags.is_broadcasting() {
        stop_broadcast(app); // Ferma la trasmissione se è in corso
    } else {
        start_broadcast(app); // Avvia la trasmissione
    }
}

/// Gestione del clic sul pulsante di avvio/arresto della registrazione
fn handle_record_button_click(app: &mut MyApp) {
    if app.flags.is_recording() {
        stop_record(app); // Ferma la registrazione se è in corso
    } else {
        start_record(app); // Avvia la registrazione
    }
}

/// Funzione per avviare la trasmissione dello schermo
pub fn start_broadcast(app: &mut MyApp) {
    println!("Starting broadcast...");
    app.flags.set_broadcasting(true);

    let capture_area = app.capture.get_capture_area().cloned().filter(|area| area.is_valid());
    let broadcast_flag = Arc::new(Mutex::new(true));
    let broadcast_flag_clone = Arc::clone(&broadcast_flag);

    let (tx, rx) = mpsc::channel();
    app.network.set_broadcast_stop_tx(Some(tx));

    if LISTENER.lock().unwrap().is_none() {
        let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to address");
        *LISTENER.lock().unwrap() = Some(listener.try_clone().unwrap());

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        println!("Receiver connected.");
                        RECEIVERS.lock().unwrap().push(stream);
                    }
                    Err(e) => {
                        println!("Failed to accept connection: {:?}", e);
                        break;
                    }
                }
            }
            println!("Listener thread exiting.");
        });
    }

    thread::spawn(move || {
        start_broadcast_thread(broadcast_flag_clone, rx, capture_area);
    });
}

/// Funzione per avviare la registrazione dello schermo
fn start_record(app: &mut MyApp) {
    println!("Starting recording...");
    app.flags.set_recording(true);

    let capture_area = app.capture.get_capture_area().cloned().filter(|area| area.is_valid());
    let record_flag = Arc::new(Mutex::new(true));

    let (tx, rx) = mpsc::channel();
    app.network.set_record_stop_tx(Some(tx));

    let (width, height) = get_capture_dimensions(&capture_area);

    create_recording_directory("recordings");

    thread::spawn(move || {
        start_record_thread(record_flag, rx, capture_area, width, height);
    });
}

/// Funzione per fermare la trasmissione dello schermo
pub fn stop_broadcast(app: &mut MyApp) {
    println!("Stopping broadcast...");
    app.flags.set_broadcasting(false);

    let mut receivers = RECEIVERS.lock().unwrap();
    for stream in receivers.iter_mut() {
        if let Err(e) = stream.write_all(&[0]) { // Segnale speciale per indicare la chiusura
            println!("Failed to send stop signal to receiver: {:?}", e);
        } else {
            println!("Stop signal sent to receiver and connection closed.");
        }

        if let Err(e) = stream.shutdown(Shutdown::Both) {
            println!("Failed to shutdown receiver connection: {:?}", e);
        }
    }
    receivers.clear();
    println!("All receivers disconnected.");

    if let Some(tx) = app.network.get_broadcast_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal: {:?}", e);
        }
    }
}

/// Funzione per fermare la registrazione dello schermo
pub fn stop_record(app: &mut MyApp) {
    println!("Stopping recording...");
    app.flags.set_recording(false);

    if let Some(tx) = app.network.get_record_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal: {:?}", e);
        }
    }
}

/// Funzione per avviare il thread per la trasmissione dello schermo
pub fn start_broadcast_thread(
    broadcast_flag: Arc<Mutex<bool>>,
    rx: mpsc::Receiver<()>,
    capture_area: Option<CaptureArea>,
) {
    println!("Broadcast thread started");

    let mut screen_capturer = ScreenCapturer::new(capture_area);

    while *broadcast_flag.lock().unwrap() {
        if rx.try_recv().is_ok() {
            println!("Received stop signal, stopping broadcast...");
            break;
        }
        // Cattura il frame e trasmettilo
        if let Some(frame) = screen_capturer.capture_frame() {
            //println!("Captured a frame, preparing to send..."); per debug

            // Serializzare il frame
            let serialized_frame = bincode::serialize(&frame).expect("Failed to serialize frame");

            let mut receivers = RECEIVERS.lock().unwrap();
                receivers.retain(|mut stream| {
                    let length = serialized_frame.len() as u32;
                    let length_bytes = length.to_be_bytes();

                    if stream.write_all(&length_bytes).is_err() {
                        println!("Failed to send frame length.");
                        return false;
                    }

                    if stream.write_all(&serialized_frame).is_err() {
                        println!("Failed to send frame data.");
                        return false;
                    }

                    //println!("Frame sent to receiver."); per debug
                    true // Mantieni la connessione attiva
                });
        } else {
            println!("Failed to capture frame.");
        }

        sync_frame_rate(Instant::now());
    }

    println!("Broadcast thread exiting");
    
}

/// Funzione per avviare il thread per la registrazione dello schermo



pub fn start_record_thread(
    record_flag: Arc<Mutex<bool>>,
    rx: mpsc::Receiver<()>,
    capture_area: Option<CaptureArea>,
    width: usize,
    height: usize,
) {
    println!("Record thread started");

    let mut child = Command::new("ffmpeg")
        .args(&[
            "-loglevel", "quiet",
            "-f", "rawvideo",
            "-pixel_format", "rgb0", // Usare 'rgb0' per i colori corretti
            "-video_size", &format!("{}x{}", width, height),
            "-framerate", &TARGET_FRAMERATE.to_string(), // Specifica il framerate corretto
            "-i", "-", // Legge dallo stdin per il video
            "-c:v", "libx264", // Codec video
            "-preset", "ultrafast",  // Preset per ridurre la compressione e mantenere velocità reale
            "-crf", "0", // Impostare il Constant Rate Factor (CRF) a 0 per qualità massima (nessuna compressione)
            "-pix_fmt", "yuv420p", // Formato pixel compatibile
            "-r", &TARGET_FRAMERATE.to_string(), // Forza il framerate in uscita
            "-y", "recordings/recorded_video.mp4" // Nome del file di output con percorso
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start ffmpeg process");

    let mut out = child.stdin.take().expect("Failed to open stdin");

    let mut screen_capturer = ScreenCapturer::new(capture_area);

    while *record_flag.lock().unwrap() {
        if rx.try_recv().is_ok() {
            println!("Received stop signal, stopping recording...");
            break;
        }

        if let Some(frame) = screen_capturer.capture_frame() {
            let stride = frame.data.len() / height;
            let rowlen = 4 * width;
            for row in frame.data.chunks(stride) {
                let row = &row[..rowlen];
                if out.write_all(row).is_err() {
                    println!("Failed to write frame to ffmpeg");
                    break;
                }
            }
        }
    }

    println!("Flushing and closing ffmpeg...");
    drop(out); // Chiude lo stdin di ffmpeg per consentirgli di terminare correttamente

    match child.wait() {
        Ok(status) => {
            println!("ffmpeg exited with status: {:?}", status);
        }
        Err(e) => {
            println!("Failed to wait on ffmpeg child process: {}", e);
        }
    }

    println!("Record thread exiting");
}






/// Sincronizza il framerate per evitare sovraccarichi
fn sync_frame_rate(start_time: Instant) {
    let elapsed = start_time.elapsed();
    if elapsed < FRAME_DURATION {
        thread::sleep(FRAME_DURATION - elapsed);
    } else {
        println!("Warning: Frame took too long to capture, possible overload");
    }
}

/// Ottieni le dimensioni dell'area di cattura
pub fn get_capture_dimensions(capture_area: &Option<CaptureArea>) -> (usize, usize) {
    match capture_area {
        Some(area) => (area.width, area.height),
        None => {
            let display = scrap::Display::primary().unwrap();
            (display.width(), display.height())
        }
    }
}

/// Crea la directory per la registrazione se non esiste
pub fn create_recording_directory(dir: &str) {
    fs::create_dir_all(dir).expect("Failed to create recording directory");
}



pub fn render_multi_monitor_support_button(ui: &mut egui::Ui, app: &mut MyApp) {
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
        app.ui_state.set_showing_monitor_selection(true); // Mostra la selezione monitor
    }
}

/// Funzione per il rendering del pannello di anteprima del Caster
pub fn render_caster_preview(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Caster Preview").strong());

        ui.add_space(10.0);

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
            ui.label("No preview available.");
        }
    });
}
