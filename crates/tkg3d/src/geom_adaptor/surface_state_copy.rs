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
    use crate::geom::{GeomOffsetSurface, GeomPlane, GeomSurface};
    use crate::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
    use crate::geom_eval::RepSurfaceDescFull;
    use rustycad_tkernel::{ncollection::Array1, standard::Handle};
    use rustycad_tkmath::bsplslib::{Cache, SharedBSplineSurfaceCache};
    use rustycad_tkmath::gp::{Axis1, Plane3, Xyz};

    #[test]
    fn extrusion_value_copy_retains_curve_and_representation_handles() {
        let source = ExtrusionData {
            basis_curve: Handle::new(GeomAdaptorCurve::default()).cast(),
            direction: Xyz::new(1.0, 2.0, 3.0),
            eval_rep: Handle::new(RepSurfaceDescFull::new(plane_surface())).cast(),
        };
        let copied = copy_extrusion(&source);
        assert!(source.basis_curve == copied.basis_curve);
        assert!(source.eval_rep == copied.eval_rep);
        assert_eq!(source.direction, copied.direction);
    }

    #[test]
    fn revolution_value_copy_retains_curve_and_representation_handles() {
        let source = RevolutionData {
            basis_curve: Handle::new(GeomAdaptorCurve::default()).cast(),
            axis: Axis1::default(),
            eval_rep: Handle::new(RepSurfaceDescFull::new(plane_surface())).cast(),
        };
        let copied = copy_revolution(&source);
        assert!(source.basis_curve == copied.basis_curve);
        assert!(source.eval_rep == copied.eval_rep);
        assert_eq!(source.axis.location(), copied.axis.location());
        assert_eq!(source.axis.direction(), copied.axis.direction());
    }

    #[test]
    fn offset_value_copy_retains_both_nested_adaptors_and_surface() {
        let source = offset_state();
        let copied = copy_offset(&source);
        assert!(source.basis_adaptor == copied.basis_adaptor);
        assert!(source.equivalent_adaptor == copied.equivalent_adaptor);
        assert!(source.offset_surface == copied.offset_surface);
        assert!(source.eval_rep == copied.eval_rep);
        assert_eq!(source.offset, copied.offset);
    }

    fn plane_surface() -> Handle<dyn GeomSurface> {
        Handle::new(GeomPlane::from_pln(Plane3::default())).cast()
    }

    fn offset_state() -> OffsetData {
        let plane = plane_surface();
        OffsetData {
            basis_adaptor: Handle::new(GeomAdaptorSurface::with_surface(&plane).unwrap()),
            equivalent_adaptor: Handle::new(GeomAdaptorSurface::with_surface(&plane).unwrap()),
            offset_surface: Handle::new(GeomOffsetSurface::new(&plane, 2.5, false).unwrap()),
            offset: 2.5,
            eval_rep: Handle::new(RepSurfaceDescFull::new(plane)).cast(),
        }
    }

    #[test]
    fn bspline_value_copy_retains_cache_and_representation_handles() {
        let source = BSplineData {
            surface: Handle::default(),
            cache: shared_cache(),
            eval_rep: Handle::new(RepSurfaceDescFull::new(plane_surface())).cast(),
        };
        let copied = copy_bspline(&source);
        assert!(source.cache.is_same(&copied.cache));
        assert!(source.eval_rep == copied.eval_rep);
        assert!(copied.surface.is_null());
    }

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
