//! Lightweight RFC 6455 WebSocket framing and broadcaster for the Swarm Dashboard.

use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

/// Minimal standard SHA-1 implementation for RFC 6455 WebSocket Handshake
pub fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let mut msg = data.to_vec();
    let orig_len_bits = (data.len() as u64) * 8;
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&orig_len_bits.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

/// Computes the Sec-WebSocket-Accept header value
pub fn compute_accept_key(sec_key: &str) -> String {
    const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let concatenated = format!("{}{}", sec_key.trim(), WS_GUID);
    let digest = sha1_digest(concatenated.as_bytes());
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &digest)
}

/// Encodes a UTF-8 text message into an unmasked RFC 6455 WebSocket frame
pub fn encode_text_frame(text: &str) -> Vec<u8> {
    let payload = text.as_bytes();
    let len = payload.len();
    let mut frame = Vec::with_capacity(len + 10);

    // Byte 0: FIN = 1, opcode = 0x1 (Text)
    frame.push(0x81);

    // Byte 1+: Payload length
    if len <= 125 {
        frame.push(len as u8);
    } else if len <= 65535 {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }

    frame.extend_from_slice(payload);
    frame
}

/// Central WebSocket broadcaster to connected dashboard browsers
#[derive(Clone)]
pub struct WebSocketBroadcaster {
    clients: Arc<Mutex<Vec<OwnedWriteHalf>>>,
}

impl Default for WebSocketBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketBroadcaster {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_client(&self, write_half: OwnedWriteHalf) {
        let mut guard = self.clients.lock().await;
        guard.push(write_half);
    }

    pub async fn broadcast_json(&self, val: &serde_json::Value) {
        let text = serde_json::to_string(val).unwrap_or_default();
        let frame = encode_text_frame(&text);

        let mut guard = self.clients.lock().await;
        let mut active = Vec::new();

        for mut client in guard.drain(..) {
            if client.write_all(&frame).await.is_ok() && client.flush().await.is_ok() {
                active.push(client);
            }
        }

        *guard = active;
    }
}
