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

use crate::app::gui::app_main::{AnnotationTool};
use crate::utils::annotations::Annotation;
use eframe::emath::RectTransform;
use eframe::egui::Shape;

pub const TARGET_FRAMERATE: u64 = 20; // Framerate target
pub const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FRAMERATE as u64);

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
//creare nuova annotazione o aggiornarla

fn create_new_annotation(tool: AnnotationTool, start: egui::Pos2) -> Annotation {
    use crate::utils::annotations::Annotation;

    let color = egui::Color32::RED;        // colore di default
    let fill_color = egui::Color32::TRANSPARENT;
    let width = 2.0;
    let font_size = 24.0;

    match tool {
        AnnotationTool::Segment => Annotation::segment(start, color, width),
        AnnotationTool::Circle  => Annotation::circle(start, color, width, fill_color),
        AnnotationTool::Rectangle => Annotation::rect(start, color, fill_color, width),
        AnnotationTool::Arrow   => Annotation::arrow(start, color, width),
        AnnotationTool::Pencil  => Annotation::pencil(start, color, width),
        AnnotationTool::Highlighter => Annotation::highlighter(start, egui::Color32::from_rgba_premultiplied(255, 255, 0, 100), width),
        AnnotationTool::Text    => Annotation::text(start, color, font_size),
        AnnotationTool::Crop    => Annotation::crop(start),
        AnnotationTool::Eraser  => {
            // L'eraser di solito va gestito diversamente (cancellazione),
            // ma per semplicità creiamo un "segment" invisibile
            Annotation::segment(start, egui::Color32::TRANSPARENT, 0.0)
        }
    }
}

fn update_annotation_in_progress(ann: &mut Annotation, current_pos: egui::Pos2) {
    match ann {
        Annotation::Segment(seg) => {
            seg.update_ending(current_pos);
        }
        Annotation::Circle(c) => {
            c.update_radius(current_pos);
        }
        Annotation::Rect(r) => {
            r.update_p2(current_pos);
        }
        Annotation::Arrow(a) => {
            a.update_ending(current_pos);
        }
        Annotation::Pencil(p) => {
            p.update_points(current_pos);
        }
        Annotation::Highlighter(h) => {
            h.update_points(current_pos);
        }
        Annotation::Text(_t) => {
            // In questo caso potresti voler aprire una finestra di input testuale 
            // oppure accumulare i caratteri in input. 
            // (Molti progetti gestiscono la TextAnnotation in modo diverso)
        }
        Annotation::Crop(crop) => {
            crop.update(current_pos);
        }
        Annotation::Eraser(_e) => {
            // Da definire come vuoi gestirlo: es. controllare se clicchiamo su un’annotazione esistente
            // e la rimuoviamo dalla app.annotations
        }
    }
}

//render annotazioni


pub fn render_all_annotations(ui: &mut egui::Ui, app: &MyApp, rect: &egui::Rect) {
    let painter = ui.painter();
    
    // Costruiamo una trasformazione (src -> dst) 
    // dove src è (0,0) -> (texture_width, texture_height)
    // e dst è rect
    // (Qui semplifico: se il tuo frame è 1920x1080, e rect è 800x600, la trasformazione farà il resize)
    let Some(texture) = &app.texture else { return; };
    let src_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, texture.size_vec2());
    
    let transform = RectTransform::from_to(src_rect, *rect);

    // Disegna ogni annotazione
    for ann in &app.annotations {
        // `render` ritorna uno Shape
        let shape = ann.render(
            1.0,           // se vuoi passare uno scaling personalizzato
            transform,
            painter,
            false          // 'editing' false perché non stiamo editando in diretta
        );
        painter.add(shape);
    }

    // Se c'è un'annotazione in corso di creazione, disegnala in overlay
    if let Some(ann_in_prog) = &app.annotation_in_progress {
        let shape = ann_in_prog.render(1.0, transform, painter, true);
        painter.add(shape);
    }
}


pub fn handle_annotation_input(
    ui: &mut egui::Ui,
    app: &mut MyApp,
    response: &egui::Response,
    rect: &egui::Rect,
) {
    // Se non abbiamo selezionato nessun tool, esci subito
    let Some(tool) = app.selected_tool else { return; };

    // Controlliamo se l'utente ha iniziato il drag
    if response.drag_started() {
        let start_pos = response.interact_pointer_pos().unwrap();
        
        // Calcoliamo la posizione in coordinate "originali" (dipende da come hai impostato la trasformazione)
        // Se semplifichiamo, consideriamo rect come (0,0)->(width,height).
        // Se vuoi essere accurato, inverti il RectTransform definito sopra.
        let local_start = start_pos - rect.min.to_vec2();

        app.annotation_in_progress = Some(create_new_annotation(tool, local_start));
    }

    // Se l’utente sta trascinando e abbiamo un’annotazione in corso:
    if response.dragged() {
        if let Some(ann) = &mut app.annotation_in_progress {
            let current_pos = response.interact_pointer_pos().unwrap();
            let local_pos = current_pos - rect.min.to_vec2();
            update_annotation_in_progress(ann, local_pos);
        }
    }

    // Se l’utente rilascia il mouse (drag stop), spostiamo l’annotazione in `annotations`
    if response.drag_stopped() {
        if let Some(ann) = app.annotation_in_progress.take() {
            app.annotations.push(ann);
        }
    }
}


/// Mostra una finestra (Window) ridimensionabile con la preview del Caster
pub fn render_caster_preview_window(ctx: &egui::Context, app: &mut MyApp) {
    // 1) Estrarre l'ID della texture (se esiste) fuori dalla closure
    let maybe_texture_id = app.texture.as_ref().map(|t| t.id());
    let maybe_texture_size = app.texture.as_ref().map(|t| t.size_vec2());

    egui::Window::new("Caster Preview")
        .default_width(800.0)
        .default_height(600.0)
        .resizable(true)
        .show(ctx, |ui| {
            // 2) Dentro la closure usiamo *solo* i dati locali (texture_id, etc.),
            //    evitando di tenere un prestito immutabile di 'app'
            if let (Some(texture_id), Some(texture_size)) = (maybe_texture_id, maybe_texture_size) {
                // Esempio: allocheremo uno spazio grande quanto la finestra,
                // e lo rendiamo "draggabile" per le annotazioni.
                let available_size = ui.available_size();
                let resp = ui.allocate_response(available_size, egui::Sense::click_and_drag());
                let rect = resp.rect;

                // Disegno dell'immagine
                ui.painter().image(
                    texture_id,
                    rect,
                    egui::Rect::from_min_size(egui::Pos2::ZERO, texture_size),
                    egui::Color32::WHITE,
                );

                // Disegno annotazioni 
                render_all_annotations(ui, app, &rect);

                // Input annotazioni
                handle_annotation_input(ui, app, &resp, &rect);

            } else {
                ui.label("No preview available.");
            }
        });
}







pub fn render_annotation_toolbar(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.label("Strumenti annotazione:");

    // Pulsante Segment
    if ui.button("Segment").clicked() {
        app.selected_tool = Some(AnnotationTool::Segment);
    }
    if ui.button("Circle").clicked() {
        app.selected_tool = Some(AnnotationTool::Circle);
    }
    if ui.button("Rect").clicked() {
        app.selected_tool = Some(AnnotationTool::Rectangle);
    }
    if ui.button("Arrow").clicked() {
        app.selected_tool = Some(AnnotationTool::Arrow);
    }
    if ui.button("Pencil").clicked() {
        app.selected_tool = Some(AnnotationTool::Pencil);
    }
    if ui.button("Highlighter").clicked() {
        app.selected_tool = Some(AnnotationTool::Highlighter);
    }
    if ui.button("Text").clicked() {
        app.selected_tool = Some(AnnotationTool::Text);
    }
    if ui.button("Crop").clicked() {
        app.selected_tool = Some(AnnotationTool::Crop);
    }
    // In base alle tue necessità, potresti voler gestire l'Eraser in modo diverso
    if ui.button("Eraser").clicked() {
        app.selected_tool = Some(AnnotationTool::Eraser);
    }

    ui.separator();
}

