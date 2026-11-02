// What the player types on the voice chat menu goes into its text boxes
// (the player search, a new group's name and password, a group's password).
fn voicechat_menu_type(options: &mut crate::options_ui::OptionsUi, text: &str) -> core::ops::ControlFlow<()> {
    use crate::options_ui::voicechat_menu as m;
    if !m::shows(options) {
        return core::ops::ControlFlow::Continue(());
    }
    m::type_text(text);
    core::ops::ControlFlow::Break(())
}
