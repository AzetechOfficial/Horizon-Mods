// Complex Voice Chat: talk to players on servers running Simple Voice Chat
// 2.6.x, with groups, a menu of its own and a lot to customize.
// The protocol, Opus, UDP and audio are in the voicechat_engine crate; this
// is the game side: its keys, settings and the connection's bookkeeping.

pub const ID: &str = "voicechat";
pub const NAME: &str = "Complex Voice Chat";
pub const VERSION: &str = "2.0.0";
pub const DESCRIPTION: &str = "Proximity voice chat with groups, for servers with the Simple Voice Chat plugin (2.6.x). V opens the menu: Players (who's talking, each one's volume and mute), Groups (make one with a name, password and type, join or leave), Audio (microphone, speakers, volumes), Talking (push to talk or voice activation, the level meter, a microphone test) and Customize (theme, opacity, size, animations, mic icon colour, the on-screen status's style, corner and size). Hold Caps Lock to talk, M mutes, J turns voice chat off.";

pub const PUSH_TO_TALK: [&str; 3] = ["push_to_talk", "Push to talk", "key.keyboard.caps.lock"];
pub const MUTE: [&str; 3] = ["mute", "Mute microphone", "key.keyboard.m"];
pub const DISABLE: [&str; 3] = ["disable", "Turn voice chat off/on", "key.keyboard.j"];
pub const MENU: [&str; 3] = ["menu", "Voice chat menu", "key.keyboard.v"];

// The steps the Mods page's buttons cycle through (the menu has sliders).
const VOLUMES: [f32; 9] = [0.0, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
const GAINS: [f32; 7] = [0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0];
const THRESHOLDS: [f32; 7] = [-60.0, -55.0, -50.0, -45.0, -40.0, -35.0, -30.0];

/// The next step after `v` in `table`, wrapping round.
fn next(table: &[f32], v: f32) -> f32 {
    table.iter().copied().find(|t| *t > v + 1e-3).unwrap_or(table[0])
}

/// The next device after `current` (None = default, first) in `list`.
fn next_device(list: &[String], current: &Option<String>) -> Option<String> {
    match current {
        None => list.first().cloned(),
        Some(c) => list.iter().position(|n| n == c).and_then(|i| list.get(i + 1).cloned()),
    }
}

use horizon_client::options_ui::voicechat_menu as menu;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub voice_activation: bool,
    pub volume: f32,
    pub mic_gain: f32,
    pub threshold_db: f32,
    pub microphone: Option<String>,
    pub speakers: Option<String>,
    /// The icons' colour, one of `voicechat_engine::icons::COLOURS`.
    pub colour: String,
    /// The menu's and the on-screen status's look (and chat messages).
    pub look: menu::Prefs,
}

/// The look's settings, by the keys they're kept under.
const LOOK_KEYS: [&str; 9] = ["theme", "opacity", "size", "animations", "show_hud", "hud_style", "hud_corner", "hud_scale", "announce"];

impl Settings {
    fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let flag = |k: &str, d: bool| get(k).map_or(d, |v| v == "true");
        let num = |k: &str, d: f32, lo: f32, hi: f32| {
            get(k).and_then(|v| v.parse::<f32>().ok()).filter(|v| v.is_finite()).map_or(d, |v| v.clamp(lo, hi))
        };
        let name = |k: &str| get(k).filter(|v| !v.is_empty());
        Self {
            voice_activation: flag("voice_activation", false),
            volume: num("volume", 1.0, 0.0, 2.0),
            mic_gain: num("mic_gain", 1.0, 0.0, 4.0),
            threshold_db: num("threshold_db", -45.0, -60.0, 0.0),
            microphone: name("microphone"),
            speakers: name("speakers"),
            colour: get("colour")
                .filter(|c| voicechat_engine::icons::COLOURS.contains(&c.as_str()))
                .unwrap_or_else(|| "blue".into()),
            look: {
                let mut look = menu::Prefs::default();
                for k in LOOK_KEYS {
                    if let Some(v) = get(k) {
                        look.set(k, &v);
                    }
                }
                look
            },
        }
    }
    fn look_value(&self, key: &str) -> String {
        let l = &self.look;
        match key {
            "theme" => l.theme.clone(),
            "opacity" => l.opacity.to_string(),
            "size" => l.size.to_string(),
            "animations" => l.animations.to_string(),
            "show_hud" => l.show_hud.to_string(),
            "hud_style" => l.hud_style.to_string(),
            "hud_corner" => l.hud_corner.to_string(),
            "hud_scale" => l.hud_scale.to_string(),
            "announce" => l.announce.to_string(),
            _ => String::new(),
        }
    }
    pub fn labels(&self) -> Vec<String> {
        let on_off = |b: bool| if b { "ON" } else { "OFF" };
        let pct = |v: f32| (v * 100.0).round() as i32;
        vec![
            format!("Microphone: {}", self.microphone.as_deref().unwrap_or("Default")),
            format!("Speakers: {}", self.speakers.as_deref().unwrap_or("Default")),
            format!("Talk with: {}", if self.voice_activation { "voice activation" } else { "push to talk" }),
            format!("Voice volume: {}%", pct(self.volume)),
            format!("Microphone boost: {}%", pct(self.mic_gain)),
            format!("Voice activation level: {} dB", self.threshold_db.round() as i32),
            format!("On-screen status: {}", on_off(self.look.show_hud)),
            format!("Messages in chat: {}", on_off(self.look.announce)),
            format!("Mic colour: {}", self.colour),
            format!("Menu theme: {}", self.look.theme),
            format!("On-screen status style: {}", menu::HUD_STYLES[self.look.hud_style as usize % menu::HUD_STYLES.len()]),
            format!("On-screen status corner: {}", menu::HUD_CORNERS[self.look.hud_corner as usize % menu::HUD_CORNERS.len()]),
            format!("Menu animations: {}", on_off(self.look.animations)),
            "More in the V menu > Customize".to_owned(),
        ]
    }
    pub fn click(&mut self, i: usize) {
        match i {
            0 => self.microphone = next_device(&voicechat_engine::microphones(), &self.microphone),
            1 => self.speakers = next_device(&voicechat_engine::speakers(), &self.speakers),
            2 => self.voice_activation = !self.voice_activation,
            3 => self.volume = next(&VOLUMES, self.volume),
            4 => self.mic_gain = next(&GAINS, self.mic_gain),
            5 => self.threshold_db = next(&THRESHOLDS, self.threshold_db),
            6 => self.look.show_hud = !self.look.show_hud,
            7 => self.look.announce = !self.look.announce,
            8 => {
                let all = voicechat_engine::icons::COLOURS;
                let i = all.iter().position(|c| *c == self.colour).map_or(0, |i| (i + 1) % all.len());
                self.colour = all[i].into();
            }
            9 => {
                let i = menu::THEMES.iter().position(|t| *t == self.look.theme).map_or(0, |i| (i + 1) % menu::THEMES.len());
                self.look.theme = menu::THEMES[i].into();
            }
            10 => self.look.hud_style = (self.look.hud_style + 1) % menu::HUD_STYLES.len() as u8,
            11 => self.look.hud_corner = (self.look.hud_corner + 1) % menu::HUD_CORNERS.len() as u8,
            12 => self.look.animations = !self.look.animations,
            _ => {}
        }
    }
    pub fn stored(&self) -> Vec<(&'static str, String)> {
        let mut out = vec![
            ("voice_activation", self.voice_activation.to_string()),
            ("volume", self.volume.to_string()),
            ("mic_gain", self.mic_gain.to_string()),
            ("threshold_db", self.threshold_db.to_string()),
            ("microphone", self.microphone.clone().unwrap_or_default()),
            ("speakers", self.speakers.clone().unwrap_or_default()),
            ("colour", self.colour.clone()),
        ];
        out.extend(LOOK_KEYS.iter().map(|k| (*k, self.look_value(k))));
        out
    }
    pub fn engine(&self) -> voicechat_engine::Settings {
        voicechat_engine::Settings {
            voice_activation: self.voice_activation,
            volume: self.volume,
            mic_gain: self.mic_gain,
            threshold_db: self.threshold_db,
            input_device: self.microphone.clone(),
            output_device: self.speakers.clone(),
        }
    }
}

/// The icon for where voice chat is now.
pub fn state_icon(s: &voicechat_engine::Status) -> voicechat_engine::icons::Icon {
    use voicechat_engine::{State, icons::Icon};
    match s.state {
        _ if s.disabled => Icon::Disabled,
        State::Off | State::Waiting | State::Connecting | State::Lost => Icon::Connecting,
        State::Connected if s.muted => Icon::Muted,
        State::Connected if s.transmitting => Icon::Talking,
        State::Connected => Icon::Idle,
    }
}

/// An icon's runs, empty if it's missing.
pub fn icon_runs(colour: &str, icon: voicechat_engine::icons::Icon) -> Vec<(u32, u32, u32, u32)> {
    voicechat_engine::icons::icon(colour, icon).map(voicechat_engine::icons::runs).unwrap_or_default()
}

/// A key's name as written on a keyboard: `key.keyboard.caps.lock` -> CAPS LOCK.
pub fn key_text(key: &str) -> String {
    match key.strip_prefix("key.keyboard.") {
        Some(k) => k.replace('.', " ").to_uppercase(),
        None => key.to_owned(),
    }
}

/// The device lists the menu shows, found again when its pickers open.
pub fn devices(refresh: bool) -> (Vec<String>, Vec<String>) {
    static LISTS: std::sync::Mutex<Option<(Vec<String>, Vec<String>)>> = std::sync::Mutex::new(None);
    let mut guard = LISTS.lock().unwrap_or_else(|e| e.into_inner());
    if refresh || guard.is_none() {
        *guard = Some((voicechat_engine::microphones(), voicechat_engine::speakers()));
    }
    guard.clone().unwrap_or_default()
}

#[derive(Default)]
struct RuntimeState {
    settings: Option<Settings>,
    /// The server and profile voice chat joined, while on a server.
    joined: Option<(String, u128)>,
    /// The last problem said in chat.
    told: Option<String>,
    told_connected: bool,
}

fn with_state<T>(f: impl FnOnce(&mut RuntimeState) -> T) -> T {
    static STATE: std::sync::Mutex<Option<RuntimeState>> = std::sync::Mutex::new(None);
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(RuntimeState::default))
}

/// The settings, read once from the mod's data folder.
pub fn settings(get: impl Fn(&str) -> Option<String>) -> Settings {
    let (s, fresh) = with_state(|st| match &st.settings {
        Some(s) => (s.clone(), false),
        None => {
            let s = Settings::read(get);
            st.settings = Some(s.clone());
            (s, true)
        }
    });
    if fresh {
        voicechat_engine::set_settings(s.engine());
    }
    s
}

pub fn set_settings(s: Settings) {
    voicechat_engine::set_settings(s.engine());
    with_state(|st| st.settings = Some(s));
}

/// What the tick should do about joining: Some(true) join, Some(false) leave.
pub fn session_change(now: Option<(String, u128)>) -> Option<bool> {
    with_state(|st| {
        if st.joined == now {
            return None;
        }
        let was = st.joined.is_some();
        st.joined = now.clone();
        st.told = None;
        st.told_connected = false;
        match (was, now.is_some()) {
            (_, true) => Some(true),
            (true, false) => Some(false),
            (false, false) => None,
        }
    })
}

/// A chat line for a status change worth telling, once.
pub fn news(status: &voicechat_engine::Status) -> Option<String> {
    with_state(|st| {
        if status.state == voicechat_engine::State::Connected && !st.told_connected {
            st.told_connected = true;
            return Some(format!(
                "Complex Voice Chat connected. {}",
                if status.voice_activation { "Voice activation is on; V opens the menu." } else { "Press V for the menu." }
            ));
        }
        match &status.problem {
            Some(p) if st.told.as_deref() != Some(p) => {
                st.told = Some(p.clone());
                Some(format!("Complex Voice Chat: {p}"))
            }
            _ => None,
        }
    })
}

// HUD colours (ARGB).
pub const PANEL: u32 = 0x9010_1018;
pub const GREEN: u32 = 0x55FF55;
pub const RED: u32 = 0xFF5555;
pub const GREY: u32 = 0xAAAAAA;
pub const YELLOW: u32 = 0xFFDD55;
pub const WHITE: u32 = 0xFFFFFF;
pub const PAD: f32 = 3.0;
pub const LINE: f32 = 10.0;

/// Applies the menu's clicks, then gives it the state to show.
pub(super) fn sync_menu(game: &mut crate::game::Game<'_>) {
    use horizon_client::options_ui::voicechat_menu::{Action, GroupRow, PlayerRow, View};
    let mut s = {
        let game = &*game;
        settings(|k| game.native_setting(ID, k))
    };
    let before = s.clone();
    let mut refresh = false;
    let mut out = Vec::new();
    for action in menu::take_actions() {
        match action {
            Action::Mute => {
                voicechat_engine::toggle_mute();
            }
            Action::Disable => out.extend(voicechat_engine::toggle_disabled().1),
            Action::VoiceActivation(on) => s.voice_activation = on,
            Action::Volume(v) => s.volume = v,
            Action::MicGain(v) => s.mic_gain = v,
            Action::Threshold(v) => s.threshold_db = v,
            Action::Microphone(name) => s.microphone = name,
            Action::Speakers(name) => s.speakers = name,
            Action::Test(on) => voicechat_engine::set_testing(on),
            Action::PlayerVolume(uuid, v) => voicechat_engine::set_player_volume(uuid, v),
            Action::RefreshDevices => refresh = true,
            Action::Colour(c) => s.colour = c,
            Action::CreateGroup { name, password, kind } => {
                let kind = voicechat_engine::GroupKind::ALL[(kind as usize).min(2)];
                out.extend(voicechat_engine::create_group(&name, password.as_deref(), kind));
            }
            Action::JoinGroup(id, password) => out.extend(voicechat_engine::join_group(id, password.as_deref())),
            Action::LeaveGroup => out.extend(voicechat_engine::leave_group()),
            Action::Set(key, value) => {
                s.look.set(key, &value);
            }
        }
    }
    for (channel, data) in out {
        game.send(horizon_protocol::packets::play::Serverbound::CustomPayload(
            horizon_protocol::packets::configuration::CustomPayload { channel, data },
        ));
    }
    if s != before {
        set_settings(s.clone());
        for (key, value) in s.stored() {
            game.native_set_setting(ID, key, &value);
        }
    }
    let (microphones, speaker_list) = devices(refresh);
    let st = voicechat_engine::status();
    use voicechat_engine::State;
    let (state, state_good) = match st.state {
        State::Off => ("Not on a voice chat server".to_owned(), false),
        State::Waiting => ("Asking the server...".to_owned(), false),
        State::Connecting => ("Connecting...".to_owned(), false),
        State::Connected => ("Connected".to_owned(), true),
        State::Lost => ("Connection lost".to_owned(), false),
    };
    let group_name = |id: Option<u128>| id.and_then(|id| st.groups.iter().find(|g| g.id == id)).map(|g| g.name.clone());
    let players = st
        .players
        .iter()
        .map(|p| PlayerRow {
            uuid: p.uuid,
            name: p.name.clone(),
            // Talking shows as the speaker icon and the green name: no words.
            status: if p.disabled {
                "voice chat off".into()
            } else if p.disconnected {
                "not connected".into()
            } else {
                String::new()
            },
            talking: p.talking,
            volume: p.volume,
            group: group_name(p.group),
        })
        .collect();
    let talking_ids: Vec<u128> = st.players.iter().filter(|p| p.talking).map(|p| p.uuid).collect();
    let groups = st
        .groups
        .iter()
        .map(|g| GroupRow {
            id: g.id,
            name: g.name.clone(),
            locked: g.has_password,
            kind: match g.kind {
                voicechat_engine::GroupKind::Normal => 0,
                voicechat_engine::GroupKind::Open => 1,
                voicechat_engine::GroupKind::Isolated => 2,
            },
            persistent: g.persistent,
            members: g
                .members
                .iter()
                .map(|(id, n)| {
                    let me = *id == st.own;
                    (if me { format!("{n} (you)") } else { n.clone() }, if me { st.transmitting } else { talking_ids.contains(id) })
                })
                .collect(),
        })
        .collect();
    menu::set_view(View {
        state,
        state_good,
        connected: st.state == State::Connected,
        muted: st.muted,
        disabled: st.disabled,
        transmitting: st.transmitting,
        voice_activation: s.voice_activation,
        talk_key: key_text(&game.native_mod_key(ID, PUSH_TO_TALK[0], PUSH_TO_TALK[2])),
        menu_key: key_text(&game.native_mod_key(ID, MENU[0], MENU[2])),
        mute_key: key_text(&game.native_mod_key(ID, MUTE[0], MUTE[2])),
        off_key: key_text(&game.native_mod_key(ID, DISABLE[0], DISABLE[2])),
        volume: s.volume,
        mic_gain: s.mic_gain,
        threshold_db: s.threshold_db,
        level_db: st.level_db,
        testing: st.testing,
        microphone: s.microphone.clone(),
        speakers: s.speakers.clone(),
        microphones,
        speaker_list,
        players,
        icon: icon_runs(&s.colour, state_icon(&st)),
        speaker_icon: icon_runs(&s.colour, voicechat_engine::icons::Icon::Speaker),
        group_icon: icon_runs(&s.colour, voicechat_engine::icons::Icon::Group),
        swatches: voicechat_engine::icons::COLOURS
            .iter()
            .map(|c| {
                use voicechat_engine::icons::Icon;
                (c.to_string(), icon_runs(c, Icon::Idle), icon_runs(c, Icon::Talking))
            })
            .collect(),
        colour: s.colour.clone(),
        problem: st.problem,
        group: st.group,
        own_group: st.own_group,
        groups,
        groups_enabled: st.groups_enabled,
        group_problem: st.group_problem,
        prefs: s.look.clone(),
    });
}
