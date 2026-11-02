// Voice chat's plugin messages, taken out of the packet stream before the
// game decodes them (it has no use for them), for the client's tick to read.
// Also finds the game server's IP the way the connection did (SRV, then DNS).

const PREFIX: &[u8] = b"voicechat:";
/// The first bytes of a compressed frame inflated to see whether it's ours.
const PEEK: u64 = 64;

static QUEUE: std::sync::Mutex<Vec<(String, Vec<u8>)>> = std::sync::Mutex::new(Vec::new());

/// The voice chat messages received since the last call.
pub fn take() -> Vec<(String, Vec<u8>)> {
    QUEUE.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default()
}

/// The game server's IP for an address as typed (`host[:port]`).
pub fn resolve(address: &str) -> Option<std::net::IpAddr> {
    let (host, port) = crate::servers::parse_address(address)?;
    if let Ok(ip) = host.trim_matches(['[', ']']).parse() {
        return Some(ip);
    }
    let resolved = crate::resolver::resolve(&host, port).ok()?;
    resolved.addresses.first().map(|a| a.ip())
}

/// The channel and data of a custom payload body (id + payload) on a
/// `voicechat:` channel.
fn ours(body: &[u8]) -> Option<(String, Vec<u8>)> {
    let mut r = crate::buf::Reader::new(body);
    let id = r.varint().ok()?;
    if id != crate::ids::play::clientbound::CUSTOM_PAYLOAD
        && id != crate::ids::configuration::clientbound::CUSTOM_PAYLOAD
    {
        return None;
    }
    let len = usize::try_from(r.varint().ok()?).ok()?;
    if len > 256 {
        return None;
    }
    let channel = r.bytes(len).ok()?;
    if !channel.starts_with(PREFIX) {
        return None;
    }
    let channel = std::str::from_utf8(channel).ok()?.to_owned();
    Some((channel, r.rest().to_vec()))
}

/// Whether `frame` was a voice chat message (now queued).
pub(crate) fn intercept(frame: &[u8], threshold: Option<usize>) -> bool {
    use std::io::Read;
    let found = match threshold {
        None => ours(frame),
        Some(_) => {
            let mut r = crate::buf::Reader::new(frame);
            let Ok(size) = r.varint() else { return false };
            let rest = &frame[r.position()..];
            if size == 0 {
                ours(rest)
            } else {
                let Ok(size) = usize::try_from(size) else { return false };
                if size > 1 << 21 {
                    return false;
                }
                let mut head = Vec::new();
                if flate2::read::ZlibDecoder::new(rest).take(PEEK).read_to_end(&mut head).is_err() {
                    return false;
                }
                if !maybe_ours(&head) {
                    return false;
                }
                let mut body = Vec::with_capacity(size);
                if flate2::read::ZlibDecoder::new(rest).take(size as u64 + 1).read_to_end(&mut body).is_err()
                    || body.len() != size
                {
                    return false;
                }
                ours(&body)
            }
        }
    };
    let Some(message) = found else { return false };
    if let Ok(mut q) = QUEUE.lock()
        && q.len() < 4096
    {
        q.push(message);
    }
    true
}

/// The start of a body: a custom payload id and a channel starting `voicechat:`.
fn maybe_ours(head: &[u8]) -> bool {
    let mut r = crate::buf::Reader::new(head);
    let Ok(id) = r.varint() else { return false };
    if id != crate::ids::play::clientbound::CUSTOM_PAYLOAD
        && id != crate::ids::configuration::clientbound::CUSTOM_PAYLOAD
    {
        return false;
    }
    if r.varint().is_err() {
        return false;
    }
    let rest = r.rest();
    rest.len() >= PREFIX.len() && rest.starts_with(PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(channel: &str, data: &[u8]) -> Vec<u8> {
        let mut w = crate::buf::Writer::new();
        w.varint(crate::ids::play::clientbound::CUSTOM_PAYLOAD);
        w.string(channel);
        w.bytes(data);
        w.data
    }

    #[test]
    fn voice_messages_are_taken_and_others_left() {
        take();
        let body = payload("voicechat:secret", &[1, 2, 3]);
        assert!(intercept(&body, None));
        assert!(!intercept(&payload("minecraft:brand", b"x"), None));
        // Compressed threshold, sent uncompressed (size 0).
        assert!(intercept(&[&[0u8][..], &body].concat(), Some(256)));
        // Large enough to be compressed.
        let big = payload("voicechat:states", &vec![7u8; 5000]);
        let frame = crate::frame::contents(&big, Some(256)).unwrap();
        assert!(intercept(&frame, Some(256)));
        let other = crate::frame::contents(&payload("minecraft:brand", &vec![7u8; 5000]), Some(256)).unwrap();
        assert!(!intercept(&other, Some(256)));
        let got = take();
        assert_eq!(got.len(), 3);
        assert_eq!(got[0], ("voicechat:secret".to_owned(), vec![1, 2, 3]));
        assert_eq!(got[2].1.len(), 5000);
        assert!(take().is_empty());
    }

    #[test]
    fn literal_ips_resolve_without_dns() {
        assert_eq!(resolve("127.0.0.1:25565"), Some("127.0.0.1".parse().unwrap()));
    }
}
