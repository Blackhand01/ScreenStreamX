use eframe::{egui, App, CreationContext};

/// Inizializza l'interfaccia utente dell'applicazione.
/// Restituisce un risultato che indica il successo o l'errore dell'inizializzazione.
pub fn initialize() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ScreenStreamX",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    is_caster: bool,
    address: String,
}

impl MyApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            is_caster: false,
            address: String::new(),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ScreenStreamX");

            ui.horizontal(|ui| {
                if ui.button("Caster").clicked() {
                    self.is_caster = true;
                }
                if ui.button("Receiver").clicked() {
                    self.is_caster = false;
                }
            });

            if self.is_caster {
                ui.label("Caster mode selected");
                // Aggiungi opzioni per il caster qui
            } else {
                ui.label("Receiver mode selected");
                ui.horizontal(|ui| {
                    ui.label("Caster address:");
                    ui.text_edit_singleline(&mut self.address);
                });
                // Aggiungi opzioni per il receiver qui
            }
        });
    }
}
