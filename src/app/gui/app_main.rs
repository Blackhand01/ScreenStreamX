// src/app/gui/app_main.rs
use eframe::{egui, App, CreationContext};
use local_ip_address::local_ip;
use crate::app::capture::CaptureArea;

use super::visuals::{configure_visuals, central_panel, capture_area_panel};
use std::sync::mpsc;

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

    pub fn show_monitor_selection(&self) -> bool {
        self.show_monitor_selection
    }

    pub fn set_show_monitor_selection(&mut self, value: bool) {
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
}

/// Enum per gestire la modalità dell'app (Caster o Receiver).
pub enum AppMode {
    Caster,
    Receiver,
}

impl MyApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            mode: AppMode::Receiver,
            network: NetworkState::new(),
            capture: CaptureState::new(),
            ui_state: UIState::new(),
            flags: AppFlags::new(),
        }
    }

    // Metodi per gestire la modalità (Caster o Receiver)
    pub fn set_caster(&mut self, value: bool) {
        self.mode = if value { AppMode::Caster } else { AppMode::Receiver };
    }

    pub fn is_caster(&self) -> bool {
        matches!(self.mode, AppMode::Caster)
    }
}

impl App for MyApp {
    /// Metodo principale di aggiornamento dell'interfaccia utente (ciclo di eventi).
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.ui_state.is_selecting_area() {
            capture_area_panel(ctx, self);
        } else {
            // UI normale
            configure_visuals(ctx);
            central_panel(ctx, self);
        }
    }
}
