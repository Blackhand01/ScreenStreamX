use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use crate::app::network::start_client;
use crate::app::capture::ScreenCapture;
use super::app_main::MyApp;
use std::process::Command;
use crate::app::gui::caster_ui::{TARGET_FRAMERATE, FRAME_DURATION};
use std::process::Stdio;
use std::io::Write;
use std::time::Instant;
use std::collections::VecDeque;



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

pub fn start_record_thread_for_receiver(
    record_flag: Arc<Mutex<bool>>,
    rx: mpsc::Receiver<()>,
    frame_buffer: Arc<Mutex<VecDeque<ScreenCapture>>>, // Usa il buffer condiviso
    width: usize,
    height: usize,
) {
    println!("Record thread for receiver started");

    let mut child = Command::new("ffmpeg")
        .args(&[
            "-loglevel", "quiet",
            "-f", "rawvideo",
            "-pixel_format", "rgb0",
            "-video_size", &format!("{}x{}", width, height),
            "-framerate", &TARGET_FRAMERATE.to_string(),
            "-i", "-",
            "-c:v", "libx264",
            "-preset", "ultrafast",
            "-crf", "0",
            "-pix_fmt", "yuv420p",
            "-r", &TARGET_FRAMERATE.to_string(),
            "-y", "recordings/receiver_recorded_video.mp4"
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start ffmpeg process");

    let mut out = child.stdin.take().expect("Failed to open stdin");
    let mut start_time = Instant::now();

    while *record_flag.lock().unwrap() {
        if rx.try_recv().is_ok() {
            println!("Received stop signal, stopping recording...");
            break;
        }

        if let Some(frame) = frame_buffer.lock().unwrap().pop_front() {
            let stride = frame.data.len() / height;
            let rowlen = 4 * width;
            for row in frame.data.chunks(stride) {
                let row = &row[..rowlen];
                if out.write_all(row).is_err() {
                    println!("Failed to write frame to ffmpeg");
                    break;
                }
            }
        } else {
            // Attendi un breve periodo se non ci sono frame disponibili
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Sincronizza il framerate
        let elapsed = start_time.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
        start_time = Instant::now();
    }

    println!("Flushing and closing ffmpeg...");
    drop(out);

    match child.wait() {
        Ok(status) => {
            println!("ffmpeg exited with status: {:?}", status);
        }
        Err(e) => {
            println!("Failed to wait on ffmpeg child process: {}", e);
        }
    }

    println!("Record thread for receiver exiting");
}


