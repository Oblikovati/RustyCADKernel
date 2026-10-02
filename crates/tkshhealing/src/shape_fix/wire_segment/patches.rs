//! Keep edge insertion before patch mutation, including native partial failures.
use super::*;
fn patch_position(index: i32) -> Result<usize, Failure> {
    usize::try_from(index)
        .ok()
        .filter(|position| *position > 0)
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!("ShapeFix_WireSegment patch index {index}; expected index >= 1"),
            )
        })
}
fn patch_values(patch: WirePatchIndex) -> [i32; 4] {
    [patch.u_min, patch.u_max, patch.v_min, patch.v_max]
}
impl ShapeFixWireSegment {
    /// Insert edge with unconstrained patch intervals; index zero appends; `segment.add_edge(0,&edge)?`.
    pub fn add_edge(&self, index: i32, edge: &Edge) -> Result<(), Failure> {
        self.add_edge_with_patch(index, edge, WirePatchIndex::default())
    }
    /// Insert geometry before updating patch sequences; `segment.add_edge_with_patch(1,&edge,patch)?`.
    pub fn add_edge_with_patch(
        &self,
        index: i32,
        edge: &Edge,
        patch: WirePatchIndex,
    ) -> Result<(), Failure> {
        self.wire.add_edge(edge, index)?;
        for (sequence, value) in self.patches.iter().zip(patch_values(patch)) {
            if index == 0 {
                sequence.append(value);
            } else {
                sequence.change_sequence(|values| {
                    values
                        .insert_before(patch_position(index)?, value)
                        .map(|_| ())
                })?;
            }
        }
        Ok(())
    }
    /// Replace all four limits in native mutation order; `segment.set_patch_index(1,patch)?`.
    pub fn set_patch_index(&self, index: i32, patch: WirePatchIndex) -> Result<(), Failure> {
        let position = patch_position(index)?;
        for (sequence, value) in self.patches.iter().zip(patch_values(patch)) {
            sequence.change_sequence(|values| values.set_value(position, value))?;
        }
        Ok(())
    }
    /// Read U-min, U-max, V-min, V-max; `segment.patch_index(1)?`.
    pub fn patch_index(&self, index: i32) -> Result<WirePatchIndex, Failure> {
        let position = patch_position(index)?;
        let read =
            |slot: usize| self.patches[slot].sequence(|values| values.value(position).copied());
        Ok(WirePatchIndex {
            u_min: read(0)?,
            u_max: read(1)?,
            v_min: read(2)?,
            v_max: read(3)?,
        })
    }
    /// Raise the lower U limit without repairing inverted intervals; `segment.define_u_min(1,3)?`.
    pub fn define_u_min(&self, index: i32, bound: i32) -> Result<(), Failure> {
        self.constrain_patch(index, 0, bound, true)
    }
    /// Lower the upper U limit without repairing inverted intervals; `segment.define_u_max(1,3)?`.
    pub fn define_u_max(&self, index: i32, bound: i32) -> Result<(), Failure> {
        self.constrain_patch(index, 1, bound, false)
    }
    /// Raise the lower V limit without repairing inverted intervals; `segment.define_v_min(1,3)?`.
    pub fn define_v_min(&self, index: i32, bound: i32) -> Result<(), Failure> {
        self.constrain_patch(index, 2, bound, true)
    }
    /// Lower the upper V limit without repairing inverted intervals; `segment.define_v_max(1,3)?`.
    pub fn define_v_max(&self, index: i32, bound: i32) -> Result<(), Failure> {
        self.constrain_patch(index, 3, bound, false)
    }
    fn constrain_patch(
        &self,
        index: i32,
        slot: usize,
        bound: i32,
        lower: bool,
    ) -> Result<(), Failure> {
        let position = patch_position(index)?;
        self.patches[slot].change_sequence(|values| {
            let old = *values.value(position)?;
            if (lower && old < bound) || (!lower && old > bound) {
                values.set_value(position, bound)?;
            }
            Ok(())
        })
    }
    /// Both patch spans must be zero or one; `segment.check_patch_index(1)?`.
    pub fn check_patch_index(&self, index: i32) -> Result<bool, Failure> {
        let patch = self.patch_index(index)?;
        let du = patch.u_max.wrapping_sub(patch.u_min);
        let dv = patch.v_max.wrapping_sub(patch.v_min);
        Ok(matches!(du, 0 | 1) && matches!(dv, 0 | 1))
    }
}
