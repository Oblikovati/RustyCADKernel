// Rust translation of the intermediary curves connecting two contours or sharp edges that
// do not meet in ChFi3d_Builder::PerformMoreThreeCorner (ChFi3d_Builder_CnCrn.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The connection between two neighbours that do not meet, on the face between them: a
//! straight line, the projection of a Hermite curve, or a batten, given to the plate and
//! stored with its points and its pcurve on the face.
use super::batten::{BattenEnds, calcul_batten, calcul_droite};
use super::plate_bounds::{add_constraint, build_curve3d, curve_on};
use super::statics::{
    calcul_c2d_on_face, orientation_arete_vive_consecutive, orientation_ic_non_vive,
    orientation_icplus_non_vive,
};
use super::{Corner, indices, slot};
use crate::chfi3d::builder::ChFi3dBuilder;
use crate::chfi3d::{fil_curve_in_ds, fil_point_in_ds};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSPoint};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_plate::GeomPlateBuildPlateSurface;
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

/// The ends of a connection: the pcurves it leaves on the face and its end points.
struct Joint {
    curv2d: [Handle<dyn Geom2dCurve>; 2],
    p2d: [Point2; 2],
}

/// The connection itself: its pcurve, its 3d curve when it is a projection, and whether
/// it is a batten with imposed tangents at its ends.
struct Link {
    pcurve: Handle<dyn Geom2dCurve>,
    curveint: Handle<dyn GeomCurve>,
    contraint: [bool; 2],
}

/// The point of the pcurve at `u` (Standard_ConstructionError on a null pcurve).
fn pcurve_point(c2d: &Handle<dyn Geom2dCurve>, u: f64) -> Result<Point2, Failure> {
    c2d.get()
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_ConstructionError",
                "Failed to get p-curve of edge",
            )
        })?
        .value(u)
}

/// The abscissae on a seam moved by a period to the side of the other end.
fn reframe(c: &Corner, (ic, icplus): (usize, usize), p2d: &mut [Point2; 2]) {
    let pi1 = 0.0 <= p2d[0].x() && p2d[0].x() <= PI;
    let pi2 = 0.0 <= p2d[1].x() && p2d[1].x() <= PI;
    let shift = if pi2 && !pi1 {
        -2.0 * PI
    } else if !pi2 && pi1 {
        2.0 * PI
    } else {
        0.0
    };
    if c.evive[ic].is_same(&c.edgecouture) {
        p2d[0] = Point2::new(p2d[0].x() + shift, p2d[0].y());
    }
    if c.evive[icplus].is_same(&c.edgecouture) {
        p2d[1] = Point2::new(p2d[1].x() - shift, p2d[1].y());
    }
}

impl ChFi3dBuilder {
    /// The pcurves and points of the ends of the connection between `ic` and `icplus`
    /// on their common face.
    fn joint(&self, c: &Corner, (ic, icplus): (usize, usize)) -> Result<Joint, Failure> {
        let face = &c.fvive[ic][icplus];
        let curv2d1 = if c.sharp[ic] {
            BRepTool::curve_on_face(&c.evive[ic], face)?.pcurve
        } else {
            calcul_c2d_on_face(c.cd(ic)?, c.jf[ic], c.i[ic][icplus])?
        };
        let p2d1 = pcurve_point(&curv2d1, c.p[ic][icplus])?;
        let curv2d2 = if c.sharp[icplus] {
            BRepTool::curve_on_face(&c.evive[icplus], face)?.pcurve
        } else {
            calcul_c2d_on_face(c.cd(icplus)?, 3 - c.jf[icplus], c.i[icplus][ic])?
        };
        let p2d2 = pcurve_point(&curv2d2, c.p[icplus][ic])?;
        let mut p2d = [p2d1, p2d2];
        if c.couture {
            reframe(c, (ic, icplus), &mut p2d);
        }
        Ok(Joint {
            curv2d: [curv2d1, curv2d2],
            p2d,
        })
    }

    /// The connections between the neighbours that do not meet.
    pub(super) fn connections(
        &mut self,
        c: &mut Corner,
        psurf: &mut GeomPlateBuildPlateSurface,
    ) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if !c.oksea[ic] && !c.moresurf[ic] && !c.libre[ic] {
                self.connection(c, ic, psurf)?;
            }
        }
        Ok(())
    }

    /// The connection from `ic` to its next neighbour; none when its ends coincide.
    fn connection(
        &mut self,
        c: &mut Corner,
        ic: usize,
        psurf: &mut GeomPlateBuildPlateSurface,
    ) -> Result<(), Failure> {
        let (icplus, icmoins) = indices(c.nedge, ic);
        let raccordbatten = !c.regul[ic] && !c.regul[icplus];
        c.n3d += 1;
        let joint = self.joint(c, (ic, icplus))?;
        let asurf = Handle::new(GeomAdaptorSurface::with_surface(&BRepTool::surface(
            &c.fvive[ic][icplus],
        )?)?);
        let (tolu, tolv) = (asurf.u_resolution(1.0e-3)?, asurf.v_resolution(1.0e-3)?);
        let ratio = if tolu > tolv {
            tolu / tolv
        } else {
            tolv / tolu
        };
        let (xdir, ydir) = (
            joint.p2d[1].x() - joint.p2d[0].x(),
            joint.p2d[1].y() - joint.p2d[0].y(),
        );
        let l0 = (xdir * xdir + ydir * ydir).sqrt();
        if l0 < 1.0e-7 || c.ponctuel[ic] {
            c.n3d -= 1;
            c.ponctuel[ic] = true;
            if !c.deuxconges {
                if c.sharp[icplus] && c.indpoint[icplus][0] == 0 {
                    c.indpoint[icplus] = [c.indpoint[ic][1]; 2];
                }
                if c.sharp[ic] && c.indpoint[ic][0] == 0 {
                    c.indpoint[ic] = [c.indpoint[icmoins][1]; 2];
                }
            }
            return Ok(());
        }
        let link = self.link(
            c,
            (ic, icplus),
            (&joint, &asurf, ratio, raccordbatten),
            (xdir, ydir),
        )?;
        self.store_connection(c, (ic, icplus), link, (&asurf, psurf))
    }

    /// The connection curve: a segment when the resolutions of the face differ much, else
    /// the projected Hermite curve across regular edges, else a batten.
    fn link(
        &self,
        c: &mut Corner,
        (ic, icplus): (usize, usize),
        (joint, asurf, ratio, mut raccordbatten): (&Joint, &Handle<GeomAdaptorSurface>, f64, bool),
        (xdir, ydir): (f64, f64),
    ) -> Result<Link, Failure> {
        let mut link = Link {
            pcurve: Handle::default(),
            curveint: Handle::default(),
            contraint: [true, true],
        };
        if ratio > 10.0 && c.nconges == 1 {
            raccordbatten = true;
        }
        if ratio > 10.0 && raccordbatten {
            link.pcurve = calcul_droite(joint.p2d[0], xdir, ydir)?;
            raccordbatten = false;
        } else if !raccordbatten {
            let (pc, cc) = if c.regul[ic] {
                (&c.proj2d2[ic], &c.cproj2[ic])
            } else {
                (&c.proj2d1[ic + 1], &c.cproj1[ic + 1])
            };
            if cc.is_null() {
                raccordbatten = true;
            } else {
                (link.pcurve, link.curveint) = (pc.clone(), cc.clone());
                c.maxapp1 = 1.0e-6;
            }
        }
        if raccordbatten {
            link.contraint = [!c.is_libre_edge(ic), !c.is_libre_edge(icplus)];
            let ends = BattenEnds {
                p2d: joint.p2d,
                contraint: link.contraint,
                curv2d: [&joint.curv2d[0], &joint.curv2d[1]],
                par: [c.p[ic][icplus], c.p[icplus][ic]],
                inverse: [inverse(c, ic), inverse(c, icplus)],
            };
            link.pcurve = calcul_batten(asurf, &c.fvive[ic][icplus], (xdir, ydir), &ends)?;
        }
        Ok(link)
    }
}

/// The edge `ic` runs from the vertex (its first vertex for a sharp edge, the sense of
/// the contour otherwise).
fn inverse(c: &Corner, ic: usize) -> bool {
    if c.sharp[ic] {
        topexp::first_vertex(&c.evive[ic], false).is_same(&c.v1)
    } else {
        c.sens[ic] == 1
    }
}

/// The order of the plate constraint on the connection: G1 next to a contour without
/// imposed tangent, between two contours, at a regular edge tangent to another, or along
/// G1 edges.
fn connection_order(c: &Corner, (ic, icplus): (usize, usize), contraint: [bool; 2]) -> i32 {
    let g1 = (!c.sharp[ic] && !c.sharp[icplus])
        || (!contraint[0] && !c.sharp[icplus])
        || (!contraint[1] && !c.sharp[ic])
        || c.tangentregul[ic]
        || c.tangentregul[icplus]
        || c.is_g1[ic];
    i32::from(g1)
}

impl ChFi3dBuilder {
    /// The connection given to the plate and stored: its 3d curve, its points (a new one
    /// on a sharp `icplus` without), their tolerances and its pcurve on the face.
    fn store_connection(
        &mut self,
        c: &mut Corner,
        (ic, icplus): (usize, usize),
        mut link: Link,
        (asurf, psurf): (&Handle<GeomAdaptorSurface>, &mut GeomPlateBuildPlateSurface),
    ) -> Result<(), Failure> {
        let hcons = curve_on(&link.pcurve, asurf)?;
        *slot(&mut c.order, c.n3d)? = connection_order(c, (ic, icplus), link.contraint);
        add_constraint(
            psurf,
            &hcons,
            (c.order[c.n3d as usize], 10),
            (self.tolapp3d, self.angular),
        )?;
        if link.curveint.is_null() {
            let (maxapp1, pardeb, parfin) = build_curve3d(&hcons, &mut c.curv3d)?;
            c.maxapp1 = maxapp1;
            link.curveint = Handle::new(GeomTrimmedCurve::new(
                &c.curv3d, pardeb, parfin, true, true,
            )?)
            .cast();
        }
        let n3d = c.n3d;
        *slot(&mut c.indcurve3d, n3d)? = self.ds.add_curve(&mut TopOpeBRepDSCurve::new(
            &link.curveint,
            c.maxapp1,
            false,
        ));
        let curve = link
            .curveint
            .get()
            .ok_or_else(|| Failure::new("PerformMoreThreeCorner"))?;
        let (pardeb, parfin) = (curve.first_parameter(), curve.last_parameter());
        if c.sharp[icplus] && c.indpoint[icplus][0] == 0 {
            let ip = self
                .ds
                .add_point(&TopOpeBRepDSPoint::new(curve.value(parfin)?, c.maxapp));
            c.indpoint[icplus] = [ip, ip];
        }
        let isvt = if c.deuxconges {
            [c.sharp[ic], c.sharp[icplus]]
        } else {
            [false, false]
        };
        let icurve = c.indcurve3d[n3d as usize];
        let ends = [
            (Orientation::Forward, c.indpoint[ic][1], pardeb),
            (Orientation::Reversed, c.indpoint[icplus][0], parfin),
        ];
        for (k, (or, ip, par)) in ends.into_iter().enumerate() {
            let interf = fil_point_in_ds(or, icurve, ip, par, isvt[k]);
            self.ds.change_curve_interferences(icurve).append(interf);
        }
        for (k, (_, ip, _)) in ends.into_iter().enumerate() {
            if !isvt[k] {
                let tpt = self.ds.change_point(ip)?;
                tpt.set_tolerance(tpt.tolerance() + c.maxapp1);
            }
        }
        let orinterf = connection_orientation(c, (ic, icplus))?;
        let interfc = fil_curve_in_ds(icurve, c.numfa[ic][icplus], &link.pcurve, orinterf);
        self.ds
            .change_shape_interferences_index(c.numfa[ic][icplus])?
            .append(interfc);
        Ok(())
    }
}

/// The orientation of the connection on the face, from the contour `ic`, else from the
/// contour `icplus`, else from the two sharp edges.
fn connection_orientation(
    c: &Corner,
    (ic, icplus): (usize, usize),
) -> Result<Orientation, Failure> {
    if !c.sharp[ic] {
        orientation_ic_non_vive(c.cd(ic)?, c.jf[ic], c.i[ic][icplus], c.sens[ic])
    } else if !c.sharp[icplus] {
        orientation_icplus_non_vive(c.cd(icplus)?, c.jf[icplus], c.i[icplus][ic], c.sens[icplus])
    } else {
        orientation_arete_vive_consecutive(&c.fvive[ic][icplus], &c.evive[ic], &c.v1)
    }
}
