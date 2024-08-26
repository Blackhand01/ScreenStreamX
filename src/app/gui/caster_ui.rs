use eframe::egui;
use crate::app::capture::ScreenCapturer;
use crate::utils::annotations::toggle_annotation_tools;
use crate::utils::multi_monitor::multi_monitor_support;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::Write;
use std::time::Duration;
use lazy_static::lazy_static;
use super::app_main::MyApp;

lazy_static! {
    static ref RECEIVERS: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    static ref LISTENER: Arc<Mutex<Option<TcpListener>>> = Arc::new(Mutex::new(None));
}

pub fn render_capture_area_button(ui: &mut egui::Ui, app: &mut MyApp) {
    if ui.add_sized(
        [200.0, 40.0],
        egui::Button::new(
            egui::RichText::new("Select Capture Area")
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(255, 153, 102)),
    )
    .clicked()
    {
        println!("Select Capture Area clicked");
        app.set_selecting_area(true);
    }
    ui.add_space(10.0);
}

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

    if ui
        .add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new(button_label)
                    .color(egui::Color32::WHITE)
                    .strong(),
            )
            .fill(button_color),
        )
        .clicked()
    {
        handle_broadcast_button_click(app);
    }
}

fn handle_broadcast_button_click(app: &mut MyApp) {
    if app.is_recording() {
        stop_broadcast(app);
    } else {
        start_broadcast(app);
    }
}

fn start_broadcast(app: &mut MyApp) {
    println!("Starting broadcast...");
    app.set_recording(true);

    let capture_area = app.get_capture_area().cloned().filter(|area| area.is_valid());
    let recording_flag = Arc::new(Mutex::new(true));
    let recording_flag_clone = Arc::clone(&recording_flag);

    let (tx, rx) = mpsc::channel();
    app.set_stop_tx(Some(tx));

    // Avvio del listener (se non già attivo)
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
        println!("Broadcast thread started");

        let mut screen_capturer = ScreenCapturer::new(capture_area);

        while *recording_flag_clone.lock().unwrap() {
            if rx.try_recv().is_ok() {
                println!("Received stop signal, stopping broadcast...");
                break;
            }

            if let Some(frame) = screen_capturer.capture_frame() {
                println!("Captured a frame, preparing to send...");

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

                    println!("Frame sent to receiver.");
                    true // Mantieni la connessione attiva
                });
            } else {
                println!("Failed to capture frame.");
            }

            thread::sleep(Duration::from_millis(30)); // Piccola pausa per ridurre l'uso della CPU
        }

        println!("Broadcast thread exiting");

        // Non chiudiamo il listener qui, in modo che rimanga attivo per future connessioni
    });
}

fn stop_broadcast(app: &mut MyApp) {
    println!("Stopping broadcast...");
    app.set_recording(false);

    // Invia il segnale di chiusura a tutti i receiver connessi
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

    // Invia un segnale di stop al thread di broadcast
    if let Some(tx) = app.get_stop_tx() {
        if let Err(e) = tx.send(()) {
            println!("Failed to send stop signal via mpsc channel: {:?}", e);
        }
    }

    // Non chiudiamo il listener, quindi i receiver possono riconnettersi
}

pub fn render_annotation_toggle_button(ui: &mut egui::Ui, app: &mut MyApp) {
    let button_label = if app.is_annotation_tools_active() {
        "Disable Annotation Tools"
    } else {
        "Enable Annotation Tools"
    };

    if ui
        .add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new(button_label)
                    .color(egui::Color32::WHITE)
                    .strong(),
            )
            .fill(egui::Color32::from_rgb(153, 0, 153)),
        )
        .clicked()
    {
        println!("Toggle Annotation Tools clicked");
        toggle_annotation_tools(app);
    }
    ui.add_space(10.0);
}

pub fn render_multi_monitor_support_button(ui: &mut egui::Ui) {
    if ui
        .add_sized(
            [200.0, 40.0],
            egui::Button::new(
                egui::RichText::new("Multi-Monitor Support")
                    .color(egui::Color32::WHITE)
                    .strong(),
            )
            .fill(egui::Color32::from_rgb(102, 204, 255)),
        )
        .clicked()
    {
        println!("Multi-Monitor Support clicked");
        multi_monitor_support();
    }
    ui.add_space(10.0);
}
