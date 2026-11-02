// The voice chat menu, drawn over the page it borrows while open.
fn voicechat_menu_draw(
    options: &mut crate::options_ui::OptionsUi,
    batch: &mut crate::hud::HudBatch,
    atlas: &crate::hud::HudAtlas,
    cursor: (f32, f32),
) -> core::ops::ControlFlow<()> {
    use crate::options_ui::voicechat_menu as m;
    if !m::shows(options) {
        return core::ops::ControlFlow::Continue(());
    }
    m::draw(batch, atlas, cursor);
    core::ops::ControlFlow::Break(())
}
