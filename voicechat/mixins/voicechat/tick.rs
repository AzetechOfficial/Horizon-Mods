// Each tick: join or leave voice chat with the server, hand the server's
// voice chat messages to the engine and send its replies, tell the engine
// where the listener and the talking players are, and say news in chat.
fn voicechat_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::voicechat as m;
    use horizon_protocol::frame::voicechat_payloads as payloads;
    if !game.native_window() {
        return;
    }
    let on = game.native_on(m::ID);
    let settings = {
        let game = &*game;
        m::settings(|k| game.native_setting(m::ID, k))
    };
    let server = match (&game.remote, &game.player) {
        (Some(r), Some(_)) if on && r.joined && r.session.address != "local" => Some((r.session.address.clone(), r.profile())),
        _ => None,
    };
    let mut out: Vec<voicechat_engine::Payload> = Vec::new();
    match m::session_change(server.clone()) {
        Some(true) => {
            let address = server.map(|s| s.0).unwrap_or_default();
            out.extend(voicechat_engine::join(move || payloads::resolve(&address)));
        }
        Some(false) => voicechat_engine::leave(),
        None => {}
    }
    for (channel, data) in payloads::take() {
        if on && game.remote.is_some() {
            out.extend(voicechat_engine::payload(&channel, &data));
        }
    }
    m::send(game, out);
    if let Some(p) = game.native_player() {
        let remote = game.remote.as_ref();
        voicechat_engine::update_listener([p.x, p.y, p.z], p.yaw, |uuid| {
            let r = remote?;
            let id = r.player_entity(uuid)?;
            r.tracked(id as i32).map(|(_, at)| at)
        });
    }
    if on && settings.look.announce {
        let status = voicechat_engine::status();
        if let Some(line) = m::news(&status) {
            game.native_chat(&line);
        }
    }
}
