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

const TARGET_FRAMERATE: u64 = 10; // Framerate target
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
        .fill(egui::Color32::from_rgb(255, 153, 102)), // Colore arancione per la selezione dell'area
    ).clicked() {
        println!("Select Capture Area clicked");
        app.set_selecting_area(true); // Passa alla modalità di selezione dell'area
    }
    ui.add_space(10.0);
}

/// Funzione per il rendering del pulsante di avvio/arresto della trasmissione
pub fn render_broadcast_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let (button_label, button_color) = if app.is_broadcasting() {
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
    let (button_label, button_color) = if app.is_recording() {
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
    if app.is_broadcasting() {
        stop_broadcast(app); // Ferma la trasmissione se è in corso
    } else {
        start_broadcast(app); // Avvia la trasmissione
    }
}

/// Gestione del clic sul pulsante di avvio/arresto della registrazione
fn handle_record_button_click(app: &mut MyApp) {
    if app.is_recording() {
        stop_record(app); // Ferma la registrazione se è in corso
    } else {
        start_record(app); // Avvia la registrazione
    }
}

/// Funzione per avviare la trasmissione dello schermo
fn start_broadcast(app: &mut MyApp) {
    println!("Starting broadcast...");
    app.set_broadcasting(true);

    let capture_area = app.get_capture_area().cloned().filter(|area| area.is_valid());
    let broadcast_flag = Arc::new(Mutex::new(true));

    let (tx, rx) = mpsc::channel();
    app.set_broadcast_stop_tx(Some(tx));

    
    thread::spawn(move || {
        start_broadcast_thread(broadcast_flag, rx, capture_area);
    });
}

/// Funzione per avviare la registrazione dello schermo
fn start_record(app: &mut MyApp) {
    println!("Starting recording...");
    app.set_recording(true);

    let capture_area = app.get_capture_area().cloned().filter(|area| area.is_valid());
    let record_flag = Arc::new(Mutex::new(true));

    let (tx, rx) = mpsc::channel();
    app.set_record_stop_tx(Some(tx));

    let (width, height) = get_capture_dimensions(&capture_area);

    create_recording_directory("recordings");

    thread::spawn(move || {
        start_record_thread(record_flag, rx, capture_area, width, height);
    });
}

/// Funzione per fermare la trasmissione dello schermo
fn stop_broadcast(app: &mut MyApp) {
    println!("Stopping broadcast...");
    app.set_broadcasting(false);

    if let Some(tx) = app.get_broadcast_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal: {:?}", e);
        }
    }
}

/// Funzione per fermare la registrazione dello schermo
fn stop_record(app: &mut MyApp) {
    println!("Stopping recording...");
    app.set_recording(false);

    if let Some(tx) = app.get_record_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal: {:?}", e);
        }
    }
}

/// Funzione per avviare il thread per la trasmissione dello schermo
fn start_broadcast_thread(
    broadcast_flag: Arc<Mutex<bool>>,
    rx: mpsc::Receiver<()>,
    capture_area: Option<CaptureArea>,
) {
    println!("Broadcast thread started");

    // Crea un listener TCP che ascolta su una specifica porta (es: 8080)
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to address");

    println!("Waiting for receiver to connect...");

    // Aspettiamo una connessione da parte di un receiver
    if let Ok((mut stream, _addr)) = listener.accept() {
        println!("Receiver connected, starting broadcast...");

        let mut screen_capturer = ScreenCapturer::new(capture_area);

        while *broadcast_flag.lock().unwrap() {
            if rx.try_recv().is_ok() {
                println!("Received stop signal, stopping broadcast...");
                *broadcast_flag.lock().unwrap() = false;
                break;
            }

            // Cattura il frame e trasmettilo
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
            } else {
                println!("Failed to capture frame.");
            }

            sync_frame_rate(Instant::now());
        }

        println!("Broadcast thread exiting");
    } else {
        println!("Failed to accept connection from receiver.");
    }
}

/// Funzione per avviare il thread per la registrazione dello schermo
fn start_record_thread(
    record_flag: Arc<Mutex<bool>>,
    rx: mpsc::Receiver<()>,
    capture_area: Option<CaptureArea>,
    width: usize,
    height: usize,
) {
    println!("Record thread started");

    // Lancia `ffmpeg` per salvare lo stream video in un file MP4 nella cartella "recordings"
    let mut child = Command::new("ffmpeg")
        .args(&[
            "-f", "rawvideo",
            "-pixel_format", "bgra",
            "-video_size", &format!("{}x{}", width, height),
            "-framerate", &TARGET_FRAMERATE.to_string(),
            "-i", "-", // Legge dallo stdin
            "-vf", "format=yuv420p", // Filtra per convertire in un formato compatibile
            "-codec:v", "libx264", // Codec video
            "-preset", "medium",  // Preset di codifica più compatibile
            "-profile:v", "high",  // Profilo H.264 compatibile con QuickTime
            "-level", "4.0",       // Livello compatibile con QuickTime
            "-pix_fmt", "yuv420p", // Formato pixel compatibile
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
            *record_flag.lock().unwrap() = false;
            break;
        }

        // Cattura il frame e scrivilo nel file video
        if let Some(frame) = screen_capturer.capture_frame() {
            let stride = frame.data.len() / height;
            let rowlen = 4 * width;
            for row in frame.data.chunks(stride) {
                let row = &row[..rowlen];
                out.write_all(row).expect("Failed to write frame to ffmpeg");
            }
        }

        sync_frame_rate(Instant::now());
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
fn get_capture_dimensions(capture_area: &Option<CaptureArea>) -> (usize, usize) {
    match capture_area {
        Some(area) => (area.width, area.height),
        None => {
            let display = scrap::Display::primary().unwrap();
            (display.width(), display.height())
        }
    }
}

/// Crea la directory per la registrazione se non esiste
fn create_recording_directory(dir: &str) {
    fs::create_dir_all(dir).expect("Failed to create recording directory");
}

/// Funzione per il rendering del pulsante di attivazione/disattivazione degli strumenti di annotazione
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
