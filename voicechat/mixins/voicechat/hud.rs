// The on-screen status, in the corner and at the size picked in the V menu's
// Customize page. Three styles:
//   Icon only      the mic icon in the chosen colour for the state now (and
//                  the group icon while in a group); the default
//   Icon + status  the icon on a small panel, with a word or two and the group
//   Full           that, plus who's talking
// Nothing shows until a server's voice chat answers (most servers have none).
fn voicechat_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::voicechat as m;
    use horizon_client::extras::Corner;
    use voicechat_engine::State;
    if !game.native_on(m::ID) || game.remote.is_none() {
        return;
    }
    let settings = {
        let game = &*game;
        m::settings(|k| game.native_setting(m::ID, k))
    };
    let look = &settings.look;
    let corner = match look.hud_corner {
        1 => Corner::BottomRight,
        2 => Corner::TopLeft,
        3 => Corner::TopRight,
        _ => Corner::BottomLeft,
    };
    if !look.show_hud || !game.native_corner_shown(corner) {
        return;
    }
    let s = voicechat_engine::status();
    let key = m::key_text(&game.native_mod_key(m::ID, m::PUSH_TO_TALK[0], m::PUSH_TO_TALK[2]));
    let head: (String, u32) = match s.state {
        State::Off | State::Waiting => return,
        State::Connecting => ("Connecting...".into(), m::YELLOW),
        State::Lost => ("Connection lost".into(), m::RED),
        State::Connected if s.disabled => ("Voice chat off".into(), m::GREY),
        State::Connected if s.muted => ("Muted".into(), m::RED),
        State::Connected if s.transmitting => ("Talking".into(), m::GREEN),
        State::Connected if s.voice_activation => ("Ready".into(), m::WHITE),
        State::Connected => (format!("Hold {key}"), m::WHITE),
    };
    let scale = look.hud_scale.clamp(1.0, 3.0);
    const ICON: f32 = 16.0;
    let mic = m::icon_runs(&settings.colour, m::state_icon(&s));
    let group_icon = m::icon_runs(&settings.colour, voicechat_engine::icons::Icon::Group);
    let speaker = m::icon_runs(&settings.colour, voicechat_engine::icons::Icon::Speaker);
    let draw = |game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch, runs: &[(u32, u32, u32, u32)], at: [f32; 2], px: f32| {
        for &(rx, ry, len, argb) in runs {
            game.native_fill(batch, [at[0] + rx as f32 * px, at[1] + ry as f32 * px, len as f32 * px, px], argb);
        }
    };
    let start = batch.vertices.len();
    let (x, y) = if look.hud_style == 0 {
        // The icon alone, and the group icon beside it while in a group.
        let w = if s.group.is_some() { ICON * 2.0 + 2.0 } else { ICON };
        let (x, y) = corners.place(corner, w * scale, ICON * scale);
        draw(game, batch, &mic, [x, y], 1.0);
        if s.group.is_some() {
            draw(game, batch, &group_icon, [x + ICON + 2.0, y], 1.0);
        }
        (x, y)
    } else {
        let mut lines: Vec<(String, u32, bool)> = vec![(head.0, head.1, false)];
        // The group's line and the talkers' have a small icon before them.
        if let Some(g) = &s.group {
            lines.push((g.clone(), m::YELLOW, true));
        }
        if look.hud_style >= 2 {
            for name in s.talking.iter().take(6) {
                lines.push((name.clone(), m::GREEN, true));
            }
        }
        let text_x = ICON + 4.0;
        let w = lines
            .iter()
            .map(|(t, _, talker)| game.native_text_width(t) + if *talker { 10.0 } else { 0.0 })
            .fold(0.0, f32::max)
            + text_x
            + 2.0 * m::PAD;
        let h = (lines.len() as f32 * m::LINE - 1.0).max(ICON) + 2.0 * m::PAD;
        let (x, y) = corners.place(corner, w * scale, h * scale);
        game.native_fill(batch, [x, y, w, h], m::PANEL);
        draw(game, batch, &mic, [x + m::PAD, y + m::PAD], 1.0);
        let mut ty = y + m::PAD;
        for (k, (text, rgb, talker)) in lines.iter().enumerate() {
            let mut tx = x + m::PAD + text_x;
            if *talker {
                let icon = if k == 1 && s.group.is_some() { &group_icon } else { &speaker };
                draw(game, batch, icon, [tx, ty], 0.5);
                tx += 10.0;
            }
            game.native_text(batch, text, [tx, ty], *rgb, true);
            ty += m::LINE;
        }
        (x, y)
    };
    if scale != 1.0 {
        batch.scale_since(start, x, y, scale, scale);
    }
}
