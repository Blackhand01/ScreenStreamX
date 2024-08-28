use crate::app::capture::CaptureArea;

pub struct CaptureState {
    capture_area: Option<CaptureArea>,
    is_fullscreen: bool,
}

impl CaptureState {
    pub fn new() -> Self {
        Self {
            capture_area: Some(CaptureArea::default()),
            is_fullscreen: true,
        }
    }

    pub fn get_capture_area(&self) -> Option<&CaptureArea> {
        self.capture_area.as_ref()
    }

    pub fn get_capture_area_mut(&mut self) -> Option<&mut CaptureArea> {
        self.capture_area.as_mut()
    }

    pub fn set_capture_area(&mut self, area: Option<CaptureArea>) {
        if let Some(area) = &area {
            let display = scrap::Display::primary().unwrap();
            self.is_fullscreen = area.x == 0 && area.y == 0 && area.width == display.width() && area.height == display.height();
        } else {
            self.is_fullscreen = true;
        }
        self.capture_area = area;
    }

    pub fn is_fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    pub fn set_fullscreen(&mut self, value: bool) {
        self.is_fullscreen = value;
    }
}
