// SPDX-License-Identifier: GPL-2.0-only
//! Complete joined bytes carry exact coefficient bits; canonical named conclusions add no pointer identity.
use super::*;
impl JoinedPrismOrientationCertificate {
    /// Serialize complete binding and sorted conclusions; `certificate.replay_bytes()` is process independent.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"martensite.joined-prism-orientation/v1".to_vec();
        bytes.extend((self.binding.len() as u64).to_le_bytes());
        bytes.extend(&*self.binding);
        bytes.extend(
            format!(
                "{:?}/{:?}/{:?}/{}/{:?}",
                self.requested,
                self.embedding_requested,
                self.winding,
                self.jacobian_sign,
                self.faces
            )
            .as_bytes(),
        );
        bytes
    }
}
