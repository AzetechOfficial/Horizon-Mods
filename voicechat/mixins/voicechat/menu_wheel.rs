// The wheel scrolls the voice chat menu's list.
fn voicechat_menu_wheel(options: &mut crate::options_ui::OptionsUi, amount: f32) -> core::ops::ControlFlow<Vec<Event>> {
    use crate::options_ui::voicechat_menu as m;
    if !m::shows(options) {
        return core::ops::ControlFlow::Continue(());
    }
    m::wheel(amount);
    core::ops::ControlFlow::Break(Vec::new())
}
