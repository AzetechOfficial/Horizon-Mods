// Each frame while the voice chat menu is open: apply what was clicked,
// then hand the menu the latest state. When the screen closes, the menu
// closes with it and the pointer goes back to the game.
fn voicechat_frame(game: &mut crate::game::Game<'_>) {
    use crate::game::voicechat as m;
    use horizon_client::options_ui::voicechat_menu as menu;
    if !menu::is_open() {
        return;
    }
    if game.screen != crate::game::Screen::Options || !menu::shows(&game.ui.options) {
        menu::closed();
        voicechat_engine::set_testing(false);
        if game.screen == crate::game::Screen::Playing {
            game.capture(true);
        }
        return;
    }
    if menu::take_done() {
        let _ = game.options_key(winit::keyboard::KeyCode::Escape, None);
        return;
    }
    m::sync_menu(game);
}
