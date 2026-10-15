// Rust translation of LocOpe_DPrism::Curves and BarycCurve of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_DPrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curves of a drafted prism: the lines of its sides through the sample points of
//! the spine, and the line through their barycentre, inclined by the draft.
use super::{DPrism, NECHANT, basis_surface, vector_of};

use crate::errors::{construction_error, null_object};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis1, Direction3, Line3, Point3, Xyz};

impl DPrism {
    /// The `(dy, dz)` of the drafted profile edge and the normal of the spine plane
    /// (Standard_ConstructionError for a non-planar spine).
    pub(super) fn draft_frame(&self) -> Result<(f64, f64, Direction3), Failure> {
        // Retrieves dy and dz with myProfile2
        let (v1, v2) = topexp::vertices(&self.profile2, false);
        let (p1, p2) = (BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
        let (dy, dz) = (p2.y() - p1.y(), p2.z() - p1.z());
        let s = basis_surface(&self.spine)?;
        let pp = s.cast::<GeomPlane>();
        let p = pp
            .get()
            .ok_or_else(|| construction_error("LocOpe_DPrism::Curves", "the spine is not planar"))?
            .pln();
        let mut normale = p.axis().direction();
        if !p.direct()? {
            normale.reverse();
        }
        Ok((dy, dz, normale))
    }

    /// `Curves(SCurves)`: at 8 points of each spine edge, the segment along the drafted
    /// side direction, of length four times the height; `dprism.curves(&mut c)?`.
    pub fn curves(&self, scurves: &mut Sequence<Handle<dyn GeomCurve>>) -> Result<(), Failure> {
        let (dy, dz, normale) = self.draft_frame()?;
        scurves.clear(None);
        let mut the_map = MapOfShape::default();
        for e in Explorer::new(
            &self.spine.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            ShapeEnum::Shape,
        )? {
            let edg = topods::edge(&e)?;
            if !the_map.add(e.clone()) || BRepTool::degenerated(&edg)? {
                continue;
            }
            let located = BRepTool::curve_with_location(&edg)?;
            let base = located
                .curve
                .get()
                .ok_or_else(|| null_object("LocOpe_DPrism::Curves", "edge without 3D curve"))?;
            let c: Handle<dyn GeomCurve> =
                base.transformed(located.location.transformation())?.cast();
            let curve = c.get().ok_or_else(|| {
                null_object(
                    "LocOpe_DPrism::Curves",
                    "transformed geometry is not a curve",
                )
            })?;
            let (f, l) = (located.first, located.last);
            let (u1, u2) = (-2. * self.height.abs(), 2. * self.height.abs());
            for i in 0..=NECHANT {
                let prm = (f64::from(NECHANT - i) * f + f64::from(i) * l) / f64::from(NECHANT);
                let (pt, mut d1) = curve.d1(prm)?;
                if e.orientation() == Orientation::Reversed {
                    d1.reverse();
                }
                let d1 = d1.normalized()?;
                let locy = normale.crossed(Direction3::from_vector(d1)?)?;
                let ldir = vector_of(locy.xyz() * dy + normale.xyz() * dz);
                let lin = Line3::new(pt, Direction3::from_vector(ldir)?);
                let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(lin)).cast();
                scurves
                    .append(Handle::new(GeomTrimmedCurve::new(&line, u1, u2, true, true)?).cast());
            }
        }
        Ok(())
    }

    /// `BarycCurve()`: the line along the drafted height through the barycentre of the
    /// sample points of the first shape (of the spine without one);
    /// `dprism.baryc_curve()?`.
    pub fn baryc_curve(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let (_, dz, normale) = self.draft_frame()?;
        let vec = vector_of(normale.xyz() * dz);
        let mut spt = Sequence::default();
        let sampled = if self.first_shape.is_null() {
            self.spine.as_ref()
        } else {
            &self.first_shape
        };
        super::super::sample_edges(sampled, &mut spt)?;
        let mut bar = Xyz::new(0., 0., 0.);
        for p in spt.iter() {
            bar += p.xyz();
        }
        bar /= spt.len() as f64;
        let axis = Axis1::new(
            Point3::new(bar.x(), bar.y(), bar.z()),
            Direction3::from_vector(vec)?,
        );
        Ok(Handle::new(GeomLine::new(axis)).cast())
    }
}
