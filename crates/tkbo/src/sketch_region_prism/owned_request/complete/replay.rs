// SPDX-License-Identifier: GPL-2.0-only
//! Framing binds original input revisions/policies separately from semantic child names.
use super::super::request_replay::encode_owned_prism_request;
use super::result::OwnedCompletePrism;
impl OwnedCompletePrism {
    /// Read this composition's behavior version; `result.version()` does not alter child versions.
    pub(crate) fn version(&self) -> u32 {
        1
    }
    /// Encode every owned input and proof without changing child bytes; `result.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"Martensite.ScopedOwnedPrism/v1\0".to_vec();
        let request = encode_owned_prism_request(self.request());
        let body = self.body().replay_bytes();
        let volume = self.volume().replay_bytes();
        for child in [
            request.as_slice(),
            self.publication().replay_bytes(),
            self.names().replay_bytes(),
            self.reservations().replay_bytes(),
            &body,
            &volume,
        ] {
            bytes.extend_from_slice(&(child.len() as u64).to_le_bytes());
            bytes.extend_from_slice(child);
        }
        bytes
    }
}
