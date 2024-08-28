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
