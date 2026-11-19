//! Peer authorization.
//!
//! Every method except `info` requires the peer to be allowed. In "ask" mode (the default)
//! an unknown peer triggers an approval prompt in the UI; "open" mode trusts everyone.
//!
//! Peers are identified by the device id they announce. There is no cryptographic proof of
//! that id yet, so on a hostile LAN a peer could impersonate a trusted device. This is
//! documented in docs/SECURITY.md.
use crate::protocol::Peer;
use crate::state::AppState;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

pub const APPROVAL_TIMEOUT: Duration = Duration::from_secs(45);
const DENY_COOLDOWN: Duration = Duration::from_secs(30);

pub async fn authorize(state: &AppState, peer: Option<&Peer>, ip: &str) -> bool {
    if state.trust_mode() == "open" {
        return true;
    }
    let Some(peer) = peer else { return false };
    if peer.id.is_empty() {
        return false;
    }
    if state.db.is_trusted(&peer.id) || state.session_allowed.lock().contains(&peer.id) {
        return true;
    }
    if let Some(t) = state.denied.lock().get(&peer.id) {
        if t.elapsed() < DENY_COOLDOWN {
            return false;
        }
    }
    let (tx, rx) = oneshot::channel();
    let first = {
        let mut p = state.pending.lock();
        let list = p.entry(peer.id.clone()).or_default();
        list.push(tx);
        list.len() == 1
    };
    if first {
        (state.emit)("approval-request", json!({ "id": peer.id, "name": peer.name, "ip": ip }));
    }
    match tokio::time::timeout(APPROVAL_TIMEOUT, rx).await {
        Ok(Ok(allowed)) => allowed,
        _ => {
            // Timed out (or sender dropped): treat as deny, and close the prompt.
            if first {
                resolve(state, &peer.id, false, false);
                (state.emit)("approval-expired", json!({ "id": peer.id }));
            }
            false
        }
    }
}

/// Called by the UI. `remember` stores the peer as permanently trusted.
pub fn resolve(state: &AppState, id: &str, allow: bool, remember: bool) {
    if allow {
        if remember {
            let name = state.devices.lock().get(id).map(|d| d.name.clone()).unwrap_or_else(|| id.to_string());
            state.db.trust(id, &name);
        } else {
            state.session_allowed.lock().insert(id.to_string());
        }
    } else {
        state.denied.lock().insert(id.to_string(), Instant::now());
    }
    let waiters = state.pending.lock().remove(id).unwrap_or_default();
    for w in waiters {
        let _ = w.send(allow);
    }
}
