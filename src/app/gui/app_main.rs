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



pub struct MyApp {
    pub mode: AppMode,
    pub network: NetworkState,
    pub capture: CaptureState,
    pub ui_state: UIState,
    pub flags: AppFlags,
    pub hotkeys: HotkeySettings,
    pub user_settings: UserSettings,
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
            render_screen_lock_overlay(ctx);
        } else {
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