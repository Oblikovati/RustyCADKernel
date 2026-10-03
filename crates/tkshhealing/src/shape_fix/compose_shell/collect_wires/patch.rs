use crate::shape_fix::{ShapeFixWireSegment, WirePatchIndex};

pub(super) fn extend_same_patch(
    wire: &ShapeFixWireSegment,
    patch: &mut WirePatchIndex,
    nu: i32,
    nv: i32,
    extend: bool,
) -> bool {
    let Ok(candidate) = wire.patch_index(1) else {
        return false;
    };
    let (du, dv) = period_shifts(candidate, *patch, nu, nv);
    let candidate = shift_patch(candidate, du * nu, dv * nv);
    let joined = joined_patch(candidate, *patch);
    if !adjacent_or_equal(joined) {
        return false;
    }
    if extend {
        *patch = joined;
    }
    true
}

fn period_shifts(
    candidate: WirePatchIndex,
    current: WirePatchIndex,
    nu: i32,
    nv: i32,
) -> (i32, i32) {
    (
        period_shift(candidate.u_min, current.u_min, nu),
        period_shift(candidate.v_min, current.v_min, nv),
    )
}

fn period_shift(candidate_min: i32, current_min: i32, count: i32) -> i32 {
    if candidate_min - current_min > count {
        -(candidate_min - current_min) / count
    } else if current_min - candidate_min > count {
        (current_min - candidate_min) / count
    } else {
        0
    }
}

fn shift_patch(mut patch: WirePatchIndex, du: i32, dv: i32) -> WirePatchIndex {
    patch.u_min += du;
    patch.u_max += du;
    patch.v_min += dv;
    patch.v_max += dv;
    patch
}

fn joined_patch(first: WirePatchIndex, second: WirePatchIndex) -> WirePatchIndex {
    WirePatchIndex {
        u_min: first.u_min.min(second.u_min),
        u_max: first.u_max.max(second.u_max),
        v_min: first.v_min.min(second.v_min),
        v_max: first.v_max.max(second.v_max),
    }
}

fn adjacent_or_equal(patch: WirePatchIndex) -> bool {
    let du = patch.u_max - patch.u_min;
    let dv = patch.v_max - patch.v_min;
    matches!(du, 0 | 1) && matches!(dv, 0 | 1)
}

#[cfg(test)]
mod tests {
    use super::{period_shifts, shift_patch};
    use crate::shape_fix::WirePatchIndex;

    #[test]
    fn periodic_translation_scales_patch_index_shift_by_grid_size() {
        let candidate = WirePatchIndex {
            u_min: 5,
            u_max: 6,
            v_min: 1,
            v_max: 1,
        };
        let current = WirePatchIndex {
            u_min: 1,
            u_max: 2,
            v_min: 1,
            v_max: 1,
        };
        let (du, dv) = period_shifts(candidate, current, 2, 3);
        assert_eq!(shift_patch(candidate, du * 2, dv * 3).u_min, 1);
    }
}
