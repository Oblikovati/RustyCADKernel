// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of IntPatch_ImpPrmIntersection::Perform (the set-up, the start
// points and the walking) of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The quadric, the implicit function, the start points and the walking.
use super::super::arc_function::IntPatchArcFunction;
use super::super::{IntPatchTheIWalking, IntPatchTheSurfFunction};
use super::IntPatchImpPrmIntersection;
use super::setup::get_local_step;
use super::tangency::{TangencyInput, compute_tangency};
use crate::int_start::TheTopolTool;
use crate::int_surf::{IntSurfInteriorPoint, IntSurfPathPoint, IntSurfQuadric, IntSurfQuadricTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::Vector;

/// The inputs and the fixed data of one `Perform`.
pub(super) struct PrmInput<'a> {
    pub(super) surf1: &'a Adaptor3dSurfaceHandle,
    pub(super) d1: &'a (dyn TheTopolTool + 'static),
    pub(super) surf2: &'a Adaptor3dSurfaceHandle,
    pub(super) d2: &'a (dyn TheTopolTool + 'static),
    pub(super) tol_arc: f64,
    pub(super) tol_tang: f64,
    pub(super) fleche: f64,
    /// The quadric is the second surface.
    pub(super) reversed: bool,
    pub(super) types: (SurfaceType, SurfaceType),
    pub(super) quad: IntSurfQuadric,
}

impl PrmInput<'_> {
    /// The parametric surface and its domain.
    pub(super) fn psurf(&self) -> (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)) {
        if self.reversed {
            (self.surf1, self.d1)
        } else {
            (self.surf2, self.d2)
        }
    }

    /// The quadric surface.
    pub(super) fn qsurf(&self) -> &Adaptor3dSurfaceHandle {
        if self.reversed {
            self.surf2
        } else {
            self.surf1
        }
    }
}

/// The quadric of a plane, cylinder, sphere or cone (no torus here).
fn quadric_of(s: &Adaptor3dSurfaceHandle) -> Result<Option<IntSurfQuadric>, Failure> {
    let mut quad = IntSurfQuadric::new();
    match s.get_type()? {
        SurfaceType::Plane => quad.set_plane(&s.plane()?),
        SurfaceType::Cylinder => quad.set_cylinder(&s.cylinder()?),
        SurfaceType::Sphere => quad.set_sphere(&s.sphere()?),
        SurfaceType::Cone => quad.set_cone(&s.cone()?),
        _ => return Ok(None),
    }
    Ok(Some(quad))
}

impl IntPatchImpPrmIntersection {
    /// `Perform(Surf1, D1, Surf2, D2, TolArc, TolTang, Fleche, Pas)`: the quadric is the
    /// first surface that is a plane, cylinder, sphere or cone (Standard_ConstructionError
    /// when neither is); the walking starts from the points of the restrictions of the
    /// parametric surface and from its interior points (or the start point), with the
    /// step `Pas` (1e-4 on a difficult patch) and the deflection `Fleche`.
    /// `inter.perform(&s1, &*d1, &s2, &*d2, 1e-7, 1e-6, 0.01, 0.1)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn perform(
        &mut self,
        surf1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        surf2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
        tol_arc: f64,
        tol_tang: f64,
        fleche: f64,
        pas: f64,
    ) -> Result<(), Failure> {
        let types = (surf1.get_type()?, surf2.get_type()?);
        (self.done, self.empt) = (false, true);
        self.slin.clear();
        self.spnt.clear();
        let (quad, reversed) = match quadric_of(surf1)? {
            Some(q) => (q, false),
            None => match quadric_of(surf2)? {
                Some(q) => (q, true),
                None => {
                    return Err(Failure::with_message(
                        "Standard_ConstructionError",
                        &format!(
                            "IntPatch_ImpPrmIntersection::Perform() - surfaces {:?} and {:?}; expected a plane, cylinder, sphere or cone",
                            types.0, types.1
                        ),
                    ));
                }
            },
        };
        let input = PrmInput {
            surf1,
            d1,
            surf2,
            d2,
            tol_arc,
            tol_tang,
            fleche,
            reversed,
            types,
            quad,
        };
        let local_pas: f64 = get_local_step(input.psurf().0, pas)?;
        let mut func = IntPatchTheSurfFunction::new();
        func.set_implicit_surface(&input.quad);
        func.set_tolerance(IntSurfQuadricTool::tolerance(&input.quad));
        func.set_surface(input.psurf().0);
        let mut afunc = IntPatchArcFunction::new();
        afunc.set_quadric(&input.quad);
        afunc.set_surface(input.psurf().0);
        self.solrst
            .perform(&mut afunc, input.psurf().1, tol_arc, tol_tang, false)?;
        if !self.solrst.is_done() {
            return Ok(());
        }
        self.walk(&input, &mut func, local_pas)
    }

    /// The start points and the walking, then the segments, the vertices on the
    /// restrictions of the quadric and the decomposition.
    fn walk(
        &mut self,
        input: &PrmInput<'_>,
        func: &mut IntPatchTheSurfFunction,
        local_pas: f64,
    ) -> Result<(), Failure> {
        let nb_point_rst: i32 = self.solrst.nb_points()?;
        let mut destination: Vec<i32> = vec![0; nb_point_rst as usize + 2];
        let mut seqpdep: Vec<IntSurfPathPoint> = Vec::new();
        if nb_point_rst != 0 {
            let (psurf, domain) = input.psurf();
            let tin = TangencyInput {
                solrst: &self.solrst,
                domain,
                psurf,
            };
            compute_tangency(&tin, &mut seqpdep, func, &mut destination)?;
        }
        let seqpins: Vec<IntSurfInteriorPoint> = self.interior_points(input, func)?;
        if !seqpdep.is_empty() || !seqpins.is_empty() {
            let mut iwalk =
                IntPatchTheIWalking::new(input.tol_tang, input.fleche, local_pas, false)?;
            iwalk.perform(&seqpdep, &seqpins, func, input.psurf().0, input.reversed)?;
            if !iwalk.is_done() {
                return Ok(());
            }
            self.walking_lines(input, &iwalk, (&seqpdep, &destination))?;
            self.tangency_joins(input.tol_arc)?;
        }
        self.segments(input)?;
        self.restrictions(input, func)?;
        self.empt = self.slin.is_empty() && self.spnt.is_empty();
        self.done = true;
        if self.slin.is_empty() {
            return Ok(());
        }
        self.decompose(input, func)
    }

    /// Whether the plane crosses the patch (a sample on the other side of it): with a
    /// segment on the plane, the patch may only touch it, without interior points.
    fn plane_crosses(
        &self,
        input: &PrmInput<'_>,
        func: &mut IntPatchTheSurfFunction,
    ) -> Result<bool, Failure> {
        let t: &(dyn TheTopolTool + 'static) = input.psurf().1;
        let nb_samples: i32 = t.nb_samples()?;
        let mut valf = Vector::with_bounds(1, 1)?;
        let mut uvap = Vector::with_bounds(1, 2)?;
        let (s2d, _): (Point2, Point3) = t.sample_point(1)?;
        (uvap[1], uvap[2]) = (s2d.x(), s2d.y());
        func.value(&uvap, &mut valf)?;
        let rvalf: f64 = 1.0_f64.copysign(valf[1]);
        for i in 2..=nb_samples {
            let (s2d, _): (Point2, Point3) = t.sample_point(i)?;
            (uvap[1], uvap[2]) = (s2d.x(), s2d.y());
            func.value(&uvap, &mut valf)?;
            if rvalf * valf[1] < 0.0 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The interior start points (from the start point when one is set).
    fn interior_points(
        &mut self,
        input: &PrmInput<'_>,
        func: &mut IntPatchTheSurfFunction,
    ) -> Result<Vec<IntSurfInteriorPoint>, Failure> {
        let search_ins: bool =
            if input.quad.type_quadric() == SurfaceType::Plane && self.solrst.nb_segments()? > 0 {
                self.plane_crosses(input, func)?
            } else {
                true
            };
        if !search_ins {
            return Ok(Vec::new());
        }
        let (psurf, domain) = input.psurf();
        if self.my_is_start_pnt {
            self.solins
                .perform_from(func, psurf, self.my_u_start, self.my_v_start)?;
        } else {
            self.solins.perform(func, psurf, domain, input.tol_tang)?;
        }
        let mut out = Vec::new();
        for i in 1..=self.solins.nb_points()? {
            out.push(*self.solins.value(i)?);
        }
        Ok(out)
    }
}
