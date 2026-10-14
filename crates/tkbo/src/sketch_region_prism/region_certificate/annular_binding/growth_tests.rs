// SPDX-License-Identifier: GPL-2.0-only
//! Callback aliases grow native geometry only after initial admission; next capture must refuse first.
use super::*;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face, face},
};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomSurfaceOfLinearExtrusion};
use rustycad_tkmath::gp::{Point2, Point3};
enum GrowthTarget {
    Space(Edge),
    PCurve(Edge, Face),
    Extrusion(Handle<GeomSurfaceOfLinearExtrusion>),
    Hierarchy(
        rustycad_tkbrep::topods::Shape,
        rustycad_tkbrep::topods::Shape,
    ),
}
struct GrowAtBoundary {
    target: GrowthTarget,
    stage: AnnularBindingStage,
    ordinal: usize,
    seen: usize,
}
impl AnnularBindingProgress for GrowAtBoundary {
    fn checkpoint(&mut self, stage: AnnularBindingStage) -> Result<bool, Failure> {
        if stage == self.stage {
            self.seen += 1;
        }
        if stage == self.stage && self.seen == self.ordinal {
            grow(&self.target)?;
        }
        Ok(false)
    }
}
fn space_curve() -> Result<Handle<dyn GeomCurve>, Failure> {
    let poles =
        [[0., 0.], [4., 0.], [2., 2.], [0., 4.], [0., 0.]].map(|p| Point3::new(p[0], p[1], 0.));
    Ok(Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1))?).cast())
}
fn plane_curve() -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let poles = [[0., 0.], [4., 0.], [2., 2.], [0., 4.], [0., 0.]].map(|p| Point2::new(p[0], p[1]));
    Ok(Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1))?).cast())
}
fn grow(target: &GrowthTarget) -> Result<(), Failure> {
    match target {
        GrowthTarget::Space(edge) => {
            BRepBuilder::default().update_edge_curve(edge, &space_curve()?, 1e-7)
        }
        GrowthTarget::PCurve(edge, face) => {
            BRepBuilder::default().update_edge_pcurve_on_face(edge, &plane_curve()?, face, 1e-7)
        }
        GrowthTarget::Extrusion(surface) => surface.get().unwrap().set_basis_curve(&space_curve()?),
        GrowthTarget::Hierarchy(root, child) => {
            root.set_free(true)?;
            rustycad_tkbrep::topods::Builder.add(&mut root.clone(), child)
        }
    }
}
fn exercise_growth(
    stage: AnnularBindingStage,
    ordinal: usize,
    target: fn(&PrismReceipt) -> GrowthTarget,
) {
    tests::with_prepared(|request, receipt| {
        let request = four_pole_request(request);
        let region = tests::fresh_region(&request);
        let mut progress = GrowAtBoundary {
            target: target(receipt),
            stage,
            ordinal,
            seen: 0,
        };
        let Err(failure) = bind_annular_prism_v2(&request, region, receipt, &mut progress) else {
            panic!("expected callback geometry-size refusal");
        };
        assert!(progress.seen >= ordinal);
        assert_eq!(failure.classification(), TwoLoopRefusalClass::Incomplete);
        assert_growth_cause(&failure.cause);
    });
}
fn assert_growth_cause(cause: &AnnularBindingCause<'_>) {
    let AnnularBindingCause::Region(failure) = cause else {
        panic!("expected original polynomial count cause");
    };
    assert!(matches!(
        failure.cause,
        TwoLoopRegionCause::Limit {
            operand: "polynomial poles",
            requested: 4,
            measured: 5
        }
    ));
}
fn copied_space(receipt: &PrismReceipt) -> GrowthTarget {
    let face = face(&receipt.copied.as_ref().unwrap().base).unwrap();
    GrowthTarget::Space(super::super::super::isolation_tests::first_edge(&face))
}
fn copied_pcurve(receipt: &PrismReceipt) -> GrowthTarget {
    let face = face(&receipt.copied.as_ref().unwrap().base).unwrap();
    GrowthTarget::PCurve(
        super::super::super::isolation_tests::first_edge(&face),
        face,
    )
}
fn swept_basis(receipt: &PrismReceipt) -> GrowthTarget {
    let bindings = &receipt.diagnostic.as_ref().unwrap().bindings;
    let side = bindings
        .iter()
        .find(|b| {
            b.native.role == PrismImageRole::Generated
                && b.context.occurrence.shape_type().unwrap() == ShapeEnum::Face
        })
        .unwrap();
    let (surface, _) =
        BRepTool::surface_with_location(&face(&side.context.occurrence).unwrap()).unwrap();
    GrowthTarget::Extrusion(surface.cast())
}
#[test]
fn copied_space_growth_is_guarded_at_each_external_capture_boundary() {
    for (stage, ordinal) in [
        (AnnularBindingStage::Admission, 1),
        (AnnularBindingStage::Copy, 1),
        (AnnularBindingStage::Charts, 1),
        (AnnularBindingStage::Charts, 2),
        (AnnularBindingStage::Commit, 1),
    ] {
        exercise_growth(stage, ordinal, copied_space);
    }
}
#[test]
fn pcurve_growth_after_owned_chart_callback_refuses_before_next_native_capture() {
    exercise_growth(AnnularBindingStage::Charts, 2, copied_pcurve);
}
#[test]
fn extrusion_basis_growth_after_owned_chart_callback_refuses_before_next_native_capture() {
    exercise_growth(AnnularBindingStage::Charts, 2, swept_basis);
}

#[test]
fn swept_hierarchy_growth_after_chart_callback_refuses_before_next_capture() {
    tests::with_actual(|request, region, receipt| {
        let root = receipt.diagnostic.as_ref().unwrap().body.clone();
        let children = rustycad_tkbrep::topods::ShapeIterator::new(&root, true, true);
        let child = children.value().unwrap().clone();
        let mut progress = GrowAtBoundary {
            target: GrowthTarget::Hierarchy(root, child),
            stage: AnnularBindingStage::Charts,
            ordinal: 2,
            seen: 0,
        };
        let Err(failure) = bind_annular_prism_v2(request, region, receipt, &mut progress) else {
            panic!("expected callback hierarchy refusal");
        };
        assert!(progress.seen >= 2);
        assert!(
            matches!(&*failure.cause, AnnularBindingCause::History(cause) if matches!(&**cause, super::super::super::frozen::complete_history::PrismHistoryCause::Bounds(_)))
        );
    });
}

fn four_pole_request<'a, 'b>(
    request: &TwoLoopRegionRequest<'a, 'b>,
) -> TwoLoopRegionRequest<'a, 'b> {
    let mut requested = request.requested;
    requested.maximum_poles = 4;
    TwoLoopRegionRequest {
        requested,
        ..*request
    }
}
