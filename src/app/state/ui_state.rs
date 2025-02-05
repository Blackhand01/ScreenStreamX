pub struct UIState {
    selecting_area: bool,
    show_confirmation_dialog: bool,
    show_monitor_selection: bool,
    show_shortcuts_menu: bool,
    show_caster_preview_window: bool,

}

impl UIState {
    pub fn new() -> Self {
        Self {
            selecting_area: false,
            show_confirmation_dialog: false,
            show_monitor_selection: false,
            show_shortcuts_menu: false,
            show_caster_preview_window: false,
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

    pub fn is_showing_caster_preview_window(&self) -> bool {
        self.show_caster_preview_window
    }

    pub fn set_showing_caster_preview_window(&mut self, value: bool) {
        self.show_caster_preview_window = value;
    }
}
