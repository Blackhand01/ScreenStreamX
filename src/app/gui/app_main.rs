// src/app/gui/app_main.rs
use eframe::{egui, App, CreationContext};
use local_ip_address::local_ip;
use crate::app::{capture::CaptureArea, hotkey_module::HotkeyAction};
use crate::app::hotkey_module::HotkeySettings;
use crate::app::gui::caster_ui;  // Importiamo correttamente il modulo caster_ui per utilizzare le funzioni di trasmissione
use super::visuals::{configure_visuals, central_panel, capture_area_panel, monitor_selection_panel};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use crate::app::capture::ScreenCapture;
use super::receiver_ui;

pub fn initialize() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ScreenStreamX",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
/// Stato del network, incluse informazioni su indirizzi e canali di comunicazione.
pub struct NetworkState {
    address: String,
    stop_tx: Option<mpsc::Sender<()>>,
    broadcast_stop_tx: Option<mpsc::Sender<()>>,
    record_stop_tx: Option<mpsc::Sender<()>>,
}

impl NetworkState {
    pub fn new() -> Self {
        let ip_address = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(_) => String::from("Unable to get IP"),
        };

        Self {
            address: ip_address,
            stop_tx: None,
            broadcast_stop_tx: None,
            record_stop_tx: None,
        }
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn set_address(&mut self, value: String) {
        self.address = value;
    }

    // Getter e setter per broadcast_stop_tx
    pub fn get_broadcast_stop_tx(&self) -> Option<mpsc::Sender<()>> {
        self.broadcast_stop_tx.clone()
    }

    pub fn set_broadcast_stop_tx(&mut self, tx: Option<mpsc::Sender<()>>) {
        self.broadcast_stop_tx = tx;
    }

    // Getter e setter per record_stop_tx
    pub fn get_record_stop_tx(&self) -> Option<mpsc::Sender<()>> {
        self.record_stop_tx.clone()
    }

    pub fn set_record_stop_tx(&mut self, tx: Option<mpsc::Sender<()>>) {
        self.record_stop_tx = tx;
    }

    pub fn get_stop_tx(&self) -> Option<mpsc::Sender<()>> {
        self.stop_tx.clone()
    }

    pub fn set_stop_tx(&mut self, tx: Option<mpsc::Sender<()>>) {
        self.stop_tx = tx;
    }
}

/// Stato della cattura, incluse le informazioni sull'area di cattura.
pub struct CaptureState {
    capture_area: Option<CaptureArea>,
}

impl CaptureState {
    pub fn new() -> Self {
        Self {
            capture_area: Some(CaptureArea::default()), // Inizializza con un'area di cattura vuota
        }
    }

    pub fn get_capture_area(&self) -> Option<&CaptureArea> {
        self.capture_area.as_ref()
    }

    pub fn get_capture_area_mut(&mut self) -> Option<&mut CaptureArea> {
        self.capture_area.as_mut()
    }

    pub fn set_capture_area(&mut self, area: Option<CaptureArea>) {
        self.capture_area = area;
    }
}

/// Stato dell'interfaccia utente, comprese le opzioni visive e di selezione.
pub struct UIState {
    selecting_area: bool,
    show_confirmation_dialog: bool,
    show_monitor_selection: bool,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            selecting_area: false, // Inizialmente la selezione non è attiva
            show_confirmation_dialog: false, // Inizialmente la finestra di conferma non è visibile
            show_monitor_selection: false, // Inizializza con false
        }
    }

    pub fn is_selecting_area(&self) -> bool {
        self.selecting_area
    }

    pub fn set_selecting_area(&mut self, value: bool) {
        self.selecting_area = value;
    }

    pub fn show_confirmation_dialog(&self) -> bool {
        self.show_confirmation_dialog
    }

    pub fn set_show_confirmation_dialog(&mut self, value: bool) {
        self.show_confirmation_dialog = value;
    }

    pub fn is_showing_monitor_selection(&self) -> bool {
        self.show_monitor_selection
    }

    pub fn set_showing_monitor_selection(&mut self, value: bool) {
        self.show_monitor_selection = value;
    }

}

/// Flag per la gestione di stati come broadcasting, recording, ecc.
pub struct AppFlags {
    is_annotation_tools_active: bool,
    is_recording: bool,
    is_broadcasting: bool,
    is_receiving: bool,
}

impl AppFlags {
    pub fn new() -> Self {
        Self {
            is_annotation_tools_active: false,
            is_recording: false,
            is_broadcasting: false,
            is_receiving: false,
        }
    }


    pub fn is_receiving(&self) -> bool {
        self.is_receiving
    }

    pub fn set_receiving(&mut self, value: bool) {
        self.is_receiving = value;
    }
    
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn set_recording(&mut self, value: bool) {
        self.is_recording = value;
    }

    pub fn is_broadcasting(&self) -> bool {
        self.is_broadcasting
    }

    pub fn set_broadcasting(&mut self, value: bool) {
        self.is_broadcasting = value;
    }

    pub fn is_annotation_tools_active(&self) -> bool {
        self.is_annotation_tools_active
    }

    pub fn set_annotation_tools_active(&mut self, value: bool) {
        self.is_annotation_tools_active = value;
    }
}


/// Struttura principale dell'applicazione.
pub struct MyApp {
    pub mode: AppMode,
    pub network: NetworkState,
    pub capture: CaptureState,
    pub ui_state: UIState,
    pub flags: AppFlags,
    pub hotkeys: HotkeySettings,

    pub frame_receiver: Option<mpsc::Receiver<ScreenCapture>>,
    pub texture: Option<egui::TextureHandle>,
    pub receiving_flag: Arc<Mutex<bool>>,
}

/// Enum per gestire la modalità dell'app (Caster o Receiver).
pub enum AppMode {
    Caster,
    Receiver,
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
            frame_receiver: None,
            texture: None,
            receiving_flag: Arc::new(Mutex::new(false)),
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
            }
        }
    }

    pub fn render_received_image(&self, ui: &mut egui::Ui) {
        if let Some(ref texture) = self.texture {
            ui.image(texture);
        }
    }

    // Metodi per gestire la modalità (Caster o Receiver)
    pub fn set_caster(&mut self, value: bool) {
        self.mode = if value { AppMode::Caster } else { AppMode::Receiver };
    }

    pub fn is_caster(&self) -> bool {
        matches!(self.mode, AppMode::Caster)
    }

    // Metodo per avviare la trasmissione
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

    // Metodo per fermare la trasmissione
    pub fn stop_broadcast(&mut self) {
        println!("Stopping broadcast...");
        self.flags.set_broadcasting(false);

        if let Some(tx) = self.network.get_broadcast_stop_tx() {
            if let Err(e) = tx.send(()) {
                println!("Failed to send stop signal: {:?}", e);
            }
        }
    }

    // Metodo per avviare la registrazione
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

    // Metodo per fermare la registrazione
    pub fn stop_recording(&mut self) {
        println!("Stopping recording...");
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
                println!("Lock/Unlock Screen via hotkey (Not implemented yet)");
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
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Gestione della ricezione dei frame
        if self.flags.is_receiving() {
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
                }
            }
        }

        // Render della GUI principale
        egui::CentralPanel::default().show(ctx, |ui| {
            // Esegui le normali operazioni di rendering dell'interfaccia
            if self.ui_state.is_selecting_area() {
                capture_area_panel(ctx, self);
            } else if self.ui_state.is_showing_monitor_selection() {
                monitor_selection_panel(ctx, self);
            } else {
                configure_visuals(ctx);
                central_panel(ctx, self);
            }
        });

        // Render della finestra di ricezione se attiva
        if self.flags.is_receiving() {
            egui::Window::new("Receiving Window")
                .default_width(800.0)  
                .default_height(600.0)  
                .collapsible(false)  
                .resizable(true)  
                .show(ctx, |ui| {
                    egui::TopBottomPanel::top("top_panel")
                        .show_inside(ui, |ui| {
                            // Aggiungi i tuoi pulsanti o controlli qui
                            if ui.button("Stop Receiving").clicked() {
                                self.stop_receiving();
                            }

                            if ui.button("Another Button").clicked() {
                                println!("Another button clicked!");
                            }
                        });

                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    // Render del frame ricevuto (se disponibile)
                    if let Some(ref texture) = self.texture {
                        ui.image(texture);
                    } else {
                        ui.label("No image received yet.");
                    }
                });
            });

        }

        // Gestione degli eventi delle hotkeys
        if let Ok(event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
            if let Some(action) = self.hotkeys.hotkey_map.get(&event.id).cloned() {
                self.handle_hotkey_action(action);
            }
        }
    }
}
