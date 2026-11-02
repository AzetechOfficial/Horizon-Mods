// Clicks and drags on the voice chat menu.
fn voicechat_menu_click(
    options: &mut crate::options_ui::OptionsUi,
    x: f32,
    y: f32,
    size: (f32, f32),
    drag: bool,
    right: bool,
) -> core::ops::ControlFlow<Vec<Event>> {
    use crate::options_ui::voicechat_menu as m;
    if !m::shows(options) {
        return core::ops::ControlFlow::Continue(());
    }
    if !right {
        m::click(x, y, size, drag);
    }
    core::ops::ControlFlow::Break(Vec::new())
}
