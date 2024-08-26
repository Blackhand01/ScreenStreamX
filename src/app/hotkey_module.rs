use std::collections::HashMap;
use global_hotkey::{GlobalHotKeyManager, hotkey};
use global_hotkey::hotkey::{Code, HotKey};
use std::error::Error;


pub struct HotkeySettings {
    pub hotkey_map: HashMap<u32, HotkeyAction>,
    hotkey_manager: GlobalHotKeyManager,
}

#[derive(Clone)]
pub enum HotkeyAction {
    StartPauseBroadcast,
    StartStopRecording,
    LockUnlockScreen,
    ToggleAnnotation,
    QuickCaptureSelection,
    EndSession,
    SwitchMonitor,
}

impl HotkeySettings {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut settings = HotkeySettings {
            hotkey_map: HashMap::new(),
            hotkey_manager: GlobalHotKeyManager::new()?,
        };

        // Register hotkeys with their corresponding actions
        settings.register_hotkey("Ctrl+Shift+B", HotkeyAction::StartPauseBroadcast)?;
        settings.register_hotkey("Ctrl+Shift+R", HotkeyAction::StartStopRecording)?;
        settings.register_hotkey("Ctrl+Shift+L", HotkeyAction::LockUnlockScreen)?;
        settings.register_hotkey("Ctrl+Shift+A", HotkeyAction::ToggleAnnotation)?;
        settings.register_hotkey("Ctrl+Shift+S", HotkeyAction::QuickCaptureSelection)?;
        settings.register_hotkey("Ctrl+Shift+Q", HotkeyAction::EndSession)?;
        settings.register_hotkey("Ctrl+Shift+M", HotkeyAction::SwitchMonitor)?;

        Ok(settings)
    }

    fn register_hotkey(&mut self, key_combination: &str, action: HotkeyAction) -> Result<(), Box<dyn Error>> {
        let parsed_hotkey = self.parse_key_combination(key_combination)?;
        let hotkey = HotKey::new(parsed_hotkey.0, parsed_hotkey.1);
        self.hotkey_manager.register(hotkey)?;
        self.hotkey_map.insert(hotkey.id(), action);
        Ok(())
    }

    fn parse_key_combination(&self, key_combination: &str) -> Result<(Option<hotkey::Modifiers>, Code), Box<dyn Error>> {
        let mut modifiers = None;
        let mut key = None;

        for part in key_combination.split('+') {
            match part.trim() {
                "Ctrl" => modifiers = Some(modifiers.unwrap_or(hotkey::Modifiers::empty()) | hotkey::Modifiers::CONTROL),
                "Shift" => modifiers = Some(modifiers.unwrap_or(hotkey::Modifiers::empty()) | hotkey::Modifiers::SHIFT),
                "B" => key = Some(Code::KeyB),
                "R" => key = Some(Code::KeyR),
                "L" => key = Some(Code::KeyL),
                "A" => key = Some(Code::KeyA),
                "S" => key = Some(Code::KeyS),
                "Q" => key = Some(Code::KeyQ),
                "M" => key = Some(Code::KeyM),
                _ => return Err("Invalid key combination".into()),
            }
        }

        if let Some(key) = key {
            Ok((modifiers, key))
        } else {
            Err("Invalid key combination".into())
        }
    }

 
}
