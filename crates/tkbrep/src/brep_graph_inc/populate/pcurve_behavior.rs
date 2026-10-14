// SPDX-License-Identifier: GPL-2.0-only
//! Explicit import replay selection; source representation lookup is operation local.
use crate::topods::{Edge, Face, Wire};
use rustycad_tkernel::standard::Failure;
use std::borrow::Cow;

/// Select the stored pcurve lookup behavior; `PCurveImportBehavior::LegacyLocalV1` replays V1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PCurveImportBehavior {
    /// Original local-edge lookup, including missing pcurves on some located faces.
    #[default]
    LegacyLocalV1,
    /// Resolve a stored pcurve using the complete face/wire/edge occurrence location.
    CumulativeOccurrenceV2,
}
impl PCurveImportBehavior {
    pub(super) fn lookup_edge<'e>(
        self,
        edge: &'e Edge,
        face: &Face,
        wire: &Wire,
    ) -> Result<Cow<'e, Edge>, Failure> {
        match self {
            Self::LegacyLocalV1 => Ok(Cow::Borrowed(edge)),
            Self::CumulativeOccurrenceV2 => {
                // ADR 0026: lookup and stored surface must share the source occurrence frame.
                // Definition placement belongs to the importer and must not enter this key.
                let parent = (face.location() * wire.location())?;
                Ok(Cow::Owned(Edge::unchecked(edge.moved(&parent, false)?)))
            }
        }
    }
}
