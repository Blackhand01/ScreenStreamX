mod app;
mod ai;
mod utils;

use app::gui::app_main::initialize;
use ai::transcription::Transcriber;
use utils::hotkeys::HotkeyManager;

/// Punto di ingresso principale dell'applicazione ScreenCaster.
/// Questa funzione inizializza l'applicazione, configura i gestori di hotkey
/// e il trascrittore, quindi avvia il ciclo degli eventi.
fn main() {
    println!("Main function started.");

    // Inizializza l'applicazione e ottieni l'evento loop, il display e la GUI.
    if let Err(e) = initialize() {
        eprintln!("Errore durante l'inizializzazione dell'interfaccia: {}", e);
        return;
    }
    println!("Attempting to begin capture...");

    // Configura e avvia il gestore di hotkey.
    let hotkey_manager = HotkeyManager::new();
    println!("Hotkey manager initialized.");
    hotkey_manager.register_hotkeys();
    println!("Hotkeys registered.");

    // Inizializza il trascrittore.
    let _transcriber = Transcriber::new();
    println!("Transcriber initialized.");
}
