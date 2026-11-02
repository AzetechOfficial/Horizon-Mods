// Push to talk (held), mute and voice chat off/on.
fn voicechat_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::voicechat as m;
    if !game.native_on(m::ID) {
        return;
    }
    if key == game.native_mod_key(m::ID, m::PUSH_TO_TALK[0], m::PUSH_TO_TALK[2]) {
        // Released anywhere; pressed only while playing (not typing in chat).
        voicechat_engine::push_to_talk(pressed && game.native_playing());
        return;
    }
    if pressed && game.native_window() && key == game.native_mod_key(m::ID, m::MENU[0], m::MENU[2]) {
        use horizon_client::options_ui::{Page, voicechat_menu as menu};
        if game.native_playing() {
            // Over the world, as Mod Menu's panel: closing goes back to playing.
            voicechat_engine::push_to_talk(false);
            game.capture(false);
            game.open_options_at(true, Page::Credits);
            game.ui.options.in_world = true;
            menu::open();
            m::sync_menu(game);
        } else if game.screen == crate::game::Screen::Options && menu::shows(&game.ui.options) && !menu::typing() {
            let _ = game.options_key(winit::keyboard::KeyCode::Escape, None);
        }
        return;
    }
    if !pressed || !game.native_playing() {
        return;
    }
    let announce = {
        let game = &*game;
        m::settings(|k| game.native_setting(m::ID, k)).look.announce
    };
    if key == game.native_mod_key(m::ID, m::MUTE[0], m::MUTE[2]) {
        let muted = voicechat_engine::toggle_mute();
        if announce {
            game.native_chat(if muted { "Complex Voice Chat: microphone muted" } else { "Complex Voice Chat: microphone on" });
        }
    } else if key == game.native_mod_key(m::ID, m::DISABLE[0], m::DISABLE[2]) {
        let (off, out) = voicechat_engine::toggle_disabled();
        for (channel, data) in out {
            game.send(horizon_protocol::packets::play::Serverbound::CustomPayload(
                horizon_protocol::packets::configuration::CustomPayload { channel, data },
            ));
        }
        if announce {
            game.native_chat(if off {
                "Complex Voice Chat: OFF (you hear nobody and nobody hears you)"
            } else {
                "Complex Voice Chat: ON"
            });
        }
    }
}
