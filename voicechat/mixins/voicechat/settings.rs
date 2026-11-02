// The mod's settings page.
fn voicechat_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::voicechat as m;
    if mod_id != m::ID {
        return;
    }
    let game = &*game;
    buttons.extend(m::settings(|k| game.native_setting(m::ID, k)).labels());
}
