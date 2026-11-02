// A click on the settings page: change it, keep it, apply it.
fn voicechat_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::voicechat as m;
    if mod_id != m::ID {
        return;
    }
    let mut settings = {
        let game = &*game;
        m::settings(|k| game.native_setting(m::ID, k))
    };
    settings.click(button);
    m::set_settings(settings.clone());
    for (key, value) in settings.stored() {
        game.native_set_setting(m::ID, key, &value);
    }
}
