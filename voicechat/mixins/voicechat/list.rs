// The mod's row in Options > Mods, with its keys.
fn voicechat_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::voicechat as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: [m::MENU, m::PUSH_TO_TALK, m::MUTE, m::DISABLE].iter().map(|k| k.map(String::from)).collect(),
    });
}
