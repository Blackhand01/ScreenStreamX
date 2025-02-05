// src/utils/multi_monitor.rs
use scrap::Display;

pub fn get_available_monitors() -> Vec<Display> {
    Display::all().expect("Failed to get displays")
}