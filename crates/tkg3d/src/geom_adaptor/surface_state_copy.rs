//! Value-copy semantics for an optional native `GeomAdaptor_Surface` member.
use super::{
    BSplineData, BezierData, ExtrusionData, OffsetData, RevolutionData, SurfaceData, SurfaceState,
    with_data,
};

/// Copies C++ value fields and retains identities held by nested OCCT handles.
pub(crate) fn copy_value(state: &SurfaceState) -> SurfaceState {
    let data = copied_data(&state.data);
    with_data(state, data)
}

fn copied_data(data: &SurfaceData) -> SurfaceData {
    match data {
        SurfaceData::Extrusion(value) => SurfaceData::Extrusion(copy_extrusion(value)),
        SurfaceData::Revolution(value) => SurfaceData::Revolution(copy_revolution(value)),
        SurfaceData::Offset(value) => SurfaceData::Offset(copy_offset(value)),
        SurfaceData::Bezier(value) => SurfaceData::Bezier(copy_bezier(value)),
        SurfaceData::BSpline(value) => SurfaceData::BSpline(copy_bspline(value)),
        other => super::elementary_copy(other),
    }
}

fn copy_extrusion(value: &ExtrusionData) -> ExtrusionData {
    ExtrusionData {
        basis_curve: value.basis_curve.clone(),
        direction: value.direction,
        eval_rep: value.eval_rep.clone(),
    }
}

fn copy_revolution(value: &RevolutionData) -> RevolutionData {
    RevolutionData {
        basis_curve: value.basis_curve.clone(),
        axis: value.axis,
        eval_rep: value.eval_rep.clone(),
    }
}

fn copy_offset(value: &OffsetData) -> OffsetData {
    OffsetData {
        basis_adaptor: value.basis_adaptor.clone(),
        equivalent_adaptor: value.equivalent_adaptor.clone(),
        offset_surface: value.offset_surface.clone(),
        offset: value.offset,
        eval_rep: value.eval_rep.clone(),
    }
}

fn copy_bezier(value: &BezierData) -> BezierData {
    BezierData {
        surface: value.surface.clone(),
        cache: value.cache.clone(),
        eval_rep: value.eval_rep.clone(),
    }
}

fn copy_bspline(value: &BSplineData) -> BSplineData {
    BSplineData {
        surface: value.surface.clone(),
        cache: value.cache.clone(),
        eval_rep: value.eval_rep.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::{ncollection::Array1, standard::Handle};
    use rustycad_tkmath::bsplslib::{Cache, SharedBSplineSurfaceCache};

    #[test]
    fn value_copy_shares_the_native_spline_cache_handle() {
        let cache = shared_cache();
        let state = bezier_state(cache.clone());
        let copied = copy_value(&state);
        assert_bezier_cache_shared(cache, copied);
    }

    fn shared_cache() -> SharedBSplineSurfaceCache {
        let mut knots = Array1::with_bounds(1, 4).unwrap();
        knots.as_mut_slice().copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
        let cache = Cache::new(1, false, &knots, 1, false, &knots, None).unwrap();
        SharedBSplineSurfaceCache::new(cache)
    }

    fn bezier_state(cache: SharedBSplineSurfaceCache) -> SurfaceState {
        SurfaceState {
            data: SurfaceData::Bezier(BezierData {
                surface: Handle::default(),
                cache,
                eval_rep: Handle::default(),
            }),
            ..SurfaceState::default()
        }
    }

    fn assert_bezier_cache_shared(source: SharedBSplineSurfaceCache, copied: SurfaceState) {
        let SurfaceData::Bezier(copied) = copied.data else {
            panic!("Bezier surface data changed kind during value copy");
        };
        assert!(source.is_same(&copied.cache));
    }
}
