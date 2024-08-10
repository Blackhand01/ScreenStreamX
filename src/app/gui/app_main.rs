use eframe::{egui, App, CreationContext};
use local_ip_address::local_ip;
use crate::app::capture::CaptureArea;

use super::visuals::configure_visuals;
use super::visuals::central_panel;

pub fn initialize() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ScreenStreamX",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

pub struct MyApp {
    is_caster: bool,
    address: String,
    is_annotation_tools_active: bool,
    is_recording: bool,
    capture_area: Option<CaptureArea>,  // Aggiunto per gestire l'area di cattura
}

impl MyApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let ip_address = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(_) => String::from("Unable to get IP"),
        };

        Self {
            is_caster: false,
            address: ip_address,
            is_annotation_tools_active: false,
            is_recording: false,
            capture_area: None,
        }
    }

    // Getter e setter per i vari stati dell'applicazione
    pub fn is_caster(&self) -> bool {
        self.is_caster
    }

    pub fn set_caster(&mut self, value: bool) {
        self.is_caster = value;
    }

    pub fn is_annotation_tools_active(&self) -> bool {
        self.is_annotation_tools_active
    }

    pub fn set_annotation_tools_active(&mut self, value: bool) {
        self.is_annotation_tools_active = value;
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn set_recording(&mut self, value: bool) {
        self.is_recording = value;
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn set_address(&mut self, value: String) {
        self.address = value;
    }

    // Metodo getter per capture_area
    pub fn get_capture_area(&self) -> Option<&CaptureArea> {
        self.capture_area.as_ref()
    }

    // Metodo setter per capture_area
    pub fn set_capture_area(&mut self, area: Option<CaptureArea>) {
        self.capture_area = area;
    }
}


impl App for MyApp {
    /// Metodo principale di aggiornamento dell'interfaccia utente.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configura gli elementi grafici
        configure_visuals(ctx);

        // Configura il pannello centrale
        central_panel(ctx, self);
    }
}
