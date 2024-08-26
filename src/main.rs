mod app;
mod ai;
mod utils;

use app::gui::app_main::initialize;
use ai::transcription::Transcriber;
use app::hotkey_module::HotkeySettings;

/// Punto di ingresso principale dell'applicazione ScreenCaster.
/// Questa funzione inizializza l'applicazione, configura i gestori di hotkey
/// e il trascrittore, quindi avvia il ciclo degli eventi.
fn main() {
    println!("Main function started.");

    // Inizializza l'applicazione e gestisci eventuali errori di inizializzazione.
    if let Err(e) = initialize() {
        eprintln!("Errore durante l'inizializzazione dell'interfaccia: {}", e);
        return;
    }
    println!("Applicazione inizializzata con successo.");

    // Configura e avvia il gestore di hotkey.
    let hotkey_manager = match HotkeySettings::new() {
        Ok(manager) => {
            println!("Hotkey manager initialized.");
            manager
        },
        Err(e) => {
            eprintln!("Failed to initialize hotkeys: {}", e);
            return;
        }
    };

    // Se hai una funzione per registrare tutte le hotkey, chiama qui.
    // hotkey_manager.register_hotkeys(); // Decommenta questa riga se hai un metodo simile.
    println!("Hotkeys registered.");

    // Inizializza il trascrittore.
    let _transcriber = Transcriber::new();
    println!("Transcriber initialized.");

    // A questo punto l'applicazione è pronta per eseguire altre operazioni, se necessario.
    // Se hai un loop o un gestore di eventi da avviare, questo è il posto giusto.
}
