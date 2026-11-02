// On a Horizon server, voice chat's messages come as mod messages: queued
// with the plugin messages for the tick.
fn voicechat_net(channel: &str, data: &[u8]) {
    if channel.starts_with("voicechat:") {
        horizon_protocol::frame::voicechat_payloads::queue(channel, data);
    }
}
