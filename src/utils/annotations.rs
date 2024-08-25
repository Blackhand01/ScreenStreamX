use crate::app::gui::app_main::MyApp;

pub fn toggle_annotation_tools(app: &mut MyApp) {
    app.flags.set_annotation_tools_active(!app.flags.is_annotation_tools_active());
}
