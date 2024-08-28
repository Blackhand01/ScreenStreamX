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
    pub frame_receiver: Option<mpsc::Receiver<ScreenCapture>>,
    pub texture: Option<egui::TextureHandle>,
    pub receiving_flag: Arc<Mutex<bool>>,
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
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.flags.is_screen_locked() {
            render_screen_lock_overlay(ctx);
        } else {
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

            if self.flags.is_receiving() {
                egui::Window::new("Receiving Window")
                    .default_width(800.0)
                    .default_height(600.0)
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        egui::TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
                            if ui.button("Stop Receiving").clicked() {
                                self.stop_receiving();
                            }

                            if ui.button("Another Button").clicked() {
                                println!("Another button clicked!");
                            }
                        });

                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            if let Some(ref texture) = self.texture {
                                ui.image(texture);
                            } else {
                                ui.label("No image received yet.");
                            }
                        });
                    });
            }

            if let Ok(event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
                if let Some(action) = self.hotkeys.hotkey_map.get(&event.id).cloned() {
                    self.handle_hotkey_action(action);
                }
            }
        }
    }
}
