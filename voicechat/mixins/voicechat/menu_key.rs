// Keys on the voice chat menu: typing into its boxes, Escape back a page
// (it closes the menu from a sidebar page), arrows through the pages.
// Other keys do nothing to the page underneath.
fn voicechat_menu_key(
    options: &mut crate::options_ui::OptionsUi,
    key: winit::keyboard::KeyCode,
) -> core::ops::ControlFlow<Vec<Event>> {
    use crate::options_ui::voicechat_menu as m;
    if !m::shows(options) {
        return core::ops::ControlFlow::Continue(());
    }
    if !m::key(key) && key == winit::keyboard::KeyCode::Escape {
        return core::ops::ControlFlow::Continue(());
    }
    core::ops::ControlFlow::Break(Vec::new())
}
