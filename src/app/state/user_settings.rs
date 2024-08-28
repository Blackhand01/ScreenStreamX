use crate::app::gui::app_main::Theme;

pub struct UserSettings {
    theme: Theme,
}

impl UserSettings {
    pub fn new() -> Self {
        Self {
            theme: Theme::Dark,
        }
    }

    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, new_theme: Theme) {
        self.theme = new_theme;
    }
}
