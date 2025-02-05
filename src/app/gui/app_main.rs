use eframe::{egui, App, CreationContext};
use crate::app::{gui::caster_ui, hotkey_module::{HotkeyAction, HotkeySettings}};
use crate::app::gui::visuals::{configure_visuals, central_panel, capture_area_panel, monitor_selection_panel, render_screen_lock_overlay};
use std::sync::{mpsc, Arc, Mutex};
use crate::app::state::{
    network_state::NetworkState,
    capture_state::CaptureState,
    ui_state::UIState,
    app_flags::AppFlags,
    user_settings::UserSettings,
};
use eframe::NativeOptions;
use crate::app::capture::ScreenCapture;
use std::thread;
use crate::app::gui::receiver_ui::start_record_thread_for_receiver;
use std::collections::VecDeque;
use crate::utils::annotations::Annotation;
use crate::app::capture::ScreenCapturer;
use std::time::Duration;


pub fn initialize() -> Result<(), eframe::Error> {
    let options = NativeOptions::default();
    eframe::run_native(
        "ScreenStreamX",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

pub enum Theme {
    Light,
    Dark,
}

pub enum AppMode {
    Caster,
    Receiver,
}
#[derive(Debug, Clone, Copy)]
pub enum AnnotationTool {
    Segment,
    Circle,
    Rectangle,
    Arrow,
    Pencil,
    Highlighter,
    Text,
    Eraser,
    Crop,
}
pub struct MyApp {
    pub mode: AppMode,
    pub network: NetworkState,
    pub capture: CaptureState,
    pub ui_state: UIState,
    pub flags: AppFlags,
    pub hotkeys: HotkeySettings,
    pub user_settings: UserSettings,
    pub frame_receiver: Option<mpsc::Receiver<ScreenCapture>>,
    pub texture: Option<egui::TextureHandle>,
    pub receiving_flag: Arc<Mutex<bool>>,

    pub frame_buffer: Arc<Mutex<VecDeque<ScreenCapture>>>, // Buffer condiviso per i frame

    // Lista delle annotazioni create
    pub annotations: Vec<Annotation>,

    // Quando l'utente sta tracciando un'annotazione in tempo reale (drag in corso)
    pub annotation_in_progress: Option<Annotation>,

    // Tipo di strumento di annotazione selezionato
    pub selected_tool: Option<AnnotationTool>,

    // Canale su cui il thread di preview invia i frame al main thread
    caster_preview_rx: Option<mpsc::Receiver<crate::app::capture::ScreenCapture>>,
    // Per fermare il thread di preview
    caster_preview_stop: Option<mpsc::Sender<()>>,
    // Per aspettare che il thread finisca (opzionale)
    caster_preview_handle: Option<thread::JoinHandle<()>>,
}

impl MyApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let hotkeys = HotkeySettings::new().expect("Failed to initialize hotkeys");

        Self {
            mode: AppMode::Receiver,
            network: NetworkState::new(),
            capture: CaptureState::new(),
            ui_state: UIState::new(),
            flags: AppFlags::new(),
            hotkeys,
            user_settings: UserSettings::new(), // Inizializzazione di user_settings
            frame_receiver: None,
            texture: None,
            receiving_flag: Arc::new(Mutex::new(false)),

            frame_buffer: Arc::new(Mutex::new(VecDeque::new())), // Inizializza il buffer
            
            annotations: Vec::new(),
            annotation_in_progress: None,
            selected_tool: None,
            caster_preview_rx: None,
            caster_preview_stop: None,
            caster_preview_handle: None,
        }
    }

    pub fn set_frame_receiver(&mut self, receiver: Option<mpsc::Receiver<ScreenCapture>>) {
        self.frame_receiver = receiver;
    }

    pub fn update_receiver_ui(&mut self, ctx: &egui::Context) {
        if let Some(ref receiver) = self.frame_receiver {
            if let Ok(frame) = receiver.try_recv() {
                let texture = ctx.load_texture(
                    "received_frame",
                    egui::ColorImage::from_rgba_unmultiplied(
                        [frame.width as usize, frame.height as usize],
                        &frame.data,
                    ),
                    egui::TextureOptions::LINEAR,
                );
                self.texture = Some(texture);

                // Aggiungi il frame al buffer
                let mut buffer = self.frame_buffer.lock().unwrap();
                buffer.push_back(frame);
                if buffer.len() > 20 { // Mantieni solo gli ultimi 10 frame
                    buffer.pop_front();
                }
            }
        }
    }

    pub fn render_received_image(&self, ui: &mut egui::Ui) {
        if let Some(ref texture) = self.texture {
            ui.image(texture);
        }
    }

    pub fn toggle_theme(&mut self) {
        self.user_settings.set_theme(match self.user_settings.get_theme() {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        });
    }

    pub fn get_status_message(&self) -> String {
        let mut status = String::new();
    
        if self.flags.is_broadcasting() {
            status.push_str("Broadcasting | ");
        }
    
        if self.flags.is_recording() {
            status.push_str("Recording | ");
        }
    
        if self.flags.is_receiving() {
            status.push_str("Receiving | ");
        }
    
        if self.capture.is_fullscreen() {
            status.push_str("Fullscreen | ");
        } else {
            status.push_str("Custom Area | ");
        }
    
        if status.is_empty() {
            status.push_str("Idle");
        } else {
            status.truncate(status.len() - 3);
        }
    
        status
    }
    
    pub fn set_caster(&mut self, value: bool) {
        self.mode = if value { AppMode::Caster } else { AppMode::Receiver };
    }

    pub fn is_caster(&self) -> bool {
        matches!(self.mode, AppMode::Caster)
    }

    pub fn start_broadcast(&mut self) {
        println!("Starting broadcast...");
        self.flags.set_broadcasting(true);

        let capture_area = self.capture.get_capture_area().cloned().filter(|area| area.is_valid());
        let broadcast_flag = Arc::new(Mutex::new(true));

        let (tx, rx) = mpsc::channel();
        self.network.set_broadcast_stop_tx(Some(tx));

        std::thread::spawn(move || {
            caster_ui::start_broadcast_thread(broadcast_flag, rx, capture_area);
        });
    }

    pub fn stop_broadcast(&mut self) {
        println!("Stopping broadcast...");
        self.flags.set_broadcasting(false);

        if let Some(tx) = self.network.get_broadcast_stop_tx() {
            if let Err(e) = tx.send(()) {
                println!("Failed to send stop signal: {:?}", e);
            }
        }
    }

    pub fn start_recording(&mut self) {
        println!("Starting recording...");
        self.flags.set_recording(true);

        let capture_area = self.capture.get_capture_area().cloned().filter(|area| area.is_valid());
        let record_flag = Arc::new(Mutex::new(true));

        let (tx, rx) = mpsc::channel();
        self.network.set_record_stop_tx(Some(tx));

        let (width, height) = caster_ui::get_capture_dimensions(&capture_area);
        caster_ui::create_recording_directory("recordings");

        std::thread::spawn(move || {
            caster_ui::start_record_thread(record_flag, rx, capture_area, width, height);
        });
    }

    pub fn stop_recording(&mut self) {
        println!("Stopping recording...");
        self.flags.set_recording(false);

        if let Some(tx) = self.network.get_record_stop_tx() {
            if let Err(e) = tx.send(()) {
                println!("Failed to send stop signal: {:?}", e);
            }
        }
    }

    pub fn start_recording_receiver(&mut self) {
        println!("Starting recording in receiver mode...");
        self.flags.set_recording(true);

        let record_flag = Arc::new(Mutex::new(true));
        let (tx, rx) = mpsc::channel();
        self.network.set_record_stop_tx(Some(tx));

        let (width, height) = if let Some(texture) = &self.texture {
            (texture.size()[0], texture.size()[1])
        } else {
            (1920, 1080) // Dimensioni predefinite
        };

        let frame_buffer = Arc::clone(&self.frame_buffer);

        thread::spawn(move || {
            start_record_thread_for_receiver(record_flag, rx, frame_buffer, width, height);
        });
    }
    

    pub fn stop_recording_receiver(&mut self) {
        println!("Stopping recording in receiver mode...");
        self.flags.set_recording(false);

        if let Some(tx) = self.network.get_record_stop_tx() {
            if let Err(e) = tx.send(()) {
                println!("Failed to send stop signal: {:?}", e);
            }
        }
    }

    pub fn stop_receiving(&mut self) {
        println!("Stopping receiving...");
        self.flags.set_receiving(false);

        if let Some(tx) = self.network.get_stop_tx() {
            if let Err(e) = tx.send(()) {
                println!("Failed to send stop signal: {:?}", e);
            }
        }
    }

    fn handle_hotkey_action(&mut self, action: HotkeyAction) {
        match action {
            HotkeyAction::StartPauseBroadcast => {
                if self.flags.is_broadcasting() {
                    self.stop_broadcast();
                } else {
                    self.start_broadcast();
                }
            }
            HotkeyAction::StartStopRecording => {
                if self.flags.is_recording() {
                    self.stop_recording();
                } else {
                    self.start_recording();
                }
            }
            HotkeyAction::LockUnlockScreen => {
                let new_state = !self.flags.is_screen_locked();
                self.flags.set_screen_locked(new_state);
                println!("Screen lock toggled: {}", new_state);
            }
            HotkeyAction::ToggleAnnotation => {
                self.flags.set_annotation_tools_active(!self.flags.is_annotation_tools_active());
            }
            HotkeyAction::QuickCaptureSelection => {
                self.ui_state.set_selecting_area(true);
            }
            HotkeyAction::EndSession => {
                std::process::exit(0);
            }
            HotkeyAction::SwitchMonitor => {
                self.ui_state.set_showing_monitor_selection(true);
            }
        }
    }


    pub fn start_caster_preview_thread(&mut self) {
        // Se stiamo già girando, non ricreare il thread
        if self.caster_preview_rx.is_some() {
            return;
        }

        // Prende l'area di cattura (se impostata)
        let capture_area = self
            .capture
            .get_capture_area()
            .cloned()
            .filter(|area| area.is_valid());

        // Crea un canale per fermare il thread
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        self.caster_preview_stop = Some(stop_tx);

        // Crea un canale per inviare i frame al main thread
        let (frame_tx, frame_rx) = mpsc::channel::<ScreenCapture>();
        self.caster_preview_rx = Some(frame_rx);

        // Avvia il thread vero e proprio
        let handle = std::thread::spawn(move || {
            // Crea un capturer (puoi riutilizzare la stessa area)
            let mut capturer = ScreenCapturer::new(capture_area);

            // Thread loop
            loop {
                // Controlla se c’è un segnale di stop
                if stop_rx.try_recv().is_ok() {
                    println!("Caster preview thread: Received stop signal, exiting...");
                    break;
                }

                // Cattura frame
                if let Some(frame) = capturer.capture_frame() {
                    // println!(
                    //     "Captured preview frame: {}x{}, data.len = {}",
                    //     frame.width,
                    //     frame.height,
                    //     frame.data.len()
                    // );
                    // Manda il frame al main thread
                    if frame_tx.send(frame).is_err() {
                        // Se non c’è più nessuno in ascolto, termina
                        break;
                    }
                }else {
                    println!("Failed to capture frame (preview).");
                }
        

                // Limitiamo a 5 fps (200ms)
                std::thread::sleep(Duration::from_millis(200));
            }

            println!("Caster preview thread: finished");
        });

        // Salviamo l’handle se vogliamo poter fare .join() in futuro
        self.caster_preview_handle = Some(handle);
    }

    pub fn stop_caster_preview_thread(&mut self) {
        if let Some(stop_tx) = self.caster_preview_stop.take() {
            let _ = stop_tx.send(()); // Manda segnale di stop
        }

        if let Some(handle) = self.caster_preview_handle.take() {
            let _ = handle.join(); // aspetta che il thread finisca
        }

        // Svuota il receiver
        self.caster_preview_rx = None;
        //println!("Caster preview thread stopped");
    }




}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.flags.is_screen_locked() {
            render_screen_lock_overlay(ctx);
        } else {

            if self.ui_state.is_showing_caster_preview_window() {
                caster_ui::render_caster_preview_window(ctx, self);
            }

            if self.flags.is_receiving() {
                self.update_receiver_ui(ctx);
            }
    
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.ui_state.is_selecting_area() {
                    capture_area_panel(ctx, self);
                } else if self.ui_state.is_showing_monitor_selection() {
                    monitor_selection_panel(ctx, self);
                } else {
                    configure_visuals(ctx, self);
                    central_panel(ctx, self);
                }
            });
    
            if let Ok(event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
                if let Some(action) = self.hotkeys.hotkey_map.get(&event.id).cloned() {
                    self.handle_hotkey_action(action);
                }
            }
    
            // Aggiungi una piccola pausa per evitare il carico eccessivo della CPU
            std::thread::sleep(std::time::Duration::from_millis(5));

            //logica per mostrare preview con annotazioni lato caster
            if self.is_caster() {
                // Se NON c’è ancora un thread di preview, avvialo
                if self.caster_preview_rx.is_none() {
                    self.start_caster_preview_thread();
                }
            
                // Prova a ricevere un frame
                if let Some(rx) = &self.caster_preview_rx {
                    if let Ok(frame) = rx.try_recv() {
                        // Converti in texture
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [frame.width as usize, frame.height as usize],
                            &frame.data,
                        );
                        let texture = ctx.load_texture(
                            "caster_preview_texture",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.texture = Some(texture);
                    }
                }
            } else {
                // Non sei caster, quindi se c’è un thread di preview attivo, ferma tutto
                self.stop_caster_preview_thread();
            }
            

        }
    }
    
}
