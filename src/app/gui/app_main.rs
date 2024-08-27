// src/app/gui/app_main.rs
use eframe::{egui, App, CreationContext};
use local_ip_address::local_ip;
use crate::app::{capture::CaptureArea, hotkey_module::HotkeyAction};
use crate::app::hotkey_module::HotkeySettings;
use crate::app::gui::caster_ui;  // Importiamo correttamente il modulo caster_ui per utilizzare le funzioni di trasmissione
use super::visuals::{configure_visuals, central_panel, capture_area_panel, monitor_selection_panel, render_screen_lock_overlay};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

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
    is_fullscreen: bool, 
}

impl CaptureState {
    pub fn new() -> Self {
        Self {
            capture_area: Some(CaptureArea::default()), 
            is_fullscreen: true, 
        }
    }

    pub fn get_capture_area(&self) -> Option<&CaptureArea> {
        self.capture_area.as_ref()
    }

    pub fn get_capture_area_mut(&mut self) -> Option<&mut CaptureArea> {
        self.capture_area.as_mut()
    }

    pub fn set_capture_area(&mut self, area: Option<CaptureArea>) {
        if let Some(area) = &area {
            let display = scrap::Display::primary().unwrap();
            self.is_fullscreen = area.x == 0 && area.y == 0 && area.width == display.width() && area.height == display.height();
        } else {
            self.is_fullscreen = true;
        }
        self.capture_area = area;
    }

    pub fn is_fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    pub fn set_fullscreen(&mut self, value: bool) {
        self.is_fullscreen = value;
    }
}

/// Stato dell'interfaccia utente, comprese le opzioni visive e di selezione.
pub struct UIState {
    selecting_area: bool,
    show_confirmation_dialog: bool,
    show_monitor_selection: bool,
    show_shortcuts_menu: bool,

}

impl UIState {
    pub fn new() -> Self {
        Self {
            selecting_area: false, // Inizialmente la selezione non è attiva
            show_confirmation_dialog: false, // Inizialmente la finestra di conferma non è visibile
            show_monitor_selection: false, // Inizializza con false
            show_shortcuts_menu: false,
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

    pub fn is_showing_shortcuts_menu(&self) -> bool {
        self.show_shortcuts_menu
    }

    pub fn set_showing_shortcuts_menu(&mut self, value: bool) {
        self.show_shortcuts_menu = value;
    }
}

/// Flag per la gestione di stati come broadcasting, recording, ecc.
pub struct AppFlags {
    is_annotation_tools_active: bool,
    is_recording: bool,
    is_broadcasting: bool,
    is_receiving: bool,
    is_screen_locked: bool, 
}

impl AppFlags {
    pub fn new() -> Self {
        Self {
            is_annotation_tools_active: false,
            is_recording: false,
            is_broadcasting: false,
            is_receiving: false,
            is_screen_locked: false, 
        }
    }

    // Altri metodi...
    pub fn is_screen_locked(&self) -> bool {
        self.is_screen_locked
    }

    pub fn set_screen_locked(&mut self, value: bool) {
        self.is_screen_locked = value;
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
    pub user_settings: UserSettings, // Aggiunto user_settings per la gestione del tema
}

/// Enum per gestire la modalità dell'app (Caster o Receiver).
pub enum AppMode {
    Caster,
    Receiver,
}

pub struct UserSettings {
    theme: Theme,
}

impl UserSettings {
    pub fn new() -> Self {
        Self {
            theme: Theme::Dark,
        }
    }

    // Getter per il tema
    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }

    // Setter per il tema
    pub fn set_theme(&mut self, new_theme: Theme) {
        self.theme = new_theme;
    }
}

pub enum Theme {
    Light,
    Dark,
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
        }
    }

    pub fn toggle_theme(&mut self) {
        self.user_settings.set_theme(match self.user_settings.get_theme() {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        });
    }

    // Aggiunta di un metodo per ottenere lo stato come stringa
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
            // Rimuove l'ultimo carattere "| " alla fine della stringa
            status.truncate(status.len() - 3);
        }
    
        status
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
                // Alterna lo stato di blocco dello schermo
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
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.flags.is_screen_locked() {
            // Usa la funzione modulare per rendere l'overlay di blocco dello schermo
            render_screen_lock_overlay(ctx);
        } else {
            // Logica esistente per la UI
            if self.ui_state.is_selecting_area() {
                capture_area_panel(ctx, self);
            } else if self.ui_state.is_showing_monitor_selection() {
                monitor_selection_panel(ctx, self);
            } else {
                configure_visuals(ctx, self);
                central_panel(ctx, self);
            }
        }

        // Gestione degli eventi delle hotkeys
        if let Ok(event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
            if let Some(action) = self.hotkeys.hotkey_map.get(&event.id).cloned() {
                self.handle_hotkey_action(action);
            }
        }
    }
}
