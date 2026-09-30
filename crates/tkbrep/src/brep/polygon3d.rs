// Rust translation of ModelingData/TKBRep/BRep/BRep_Polygon3D.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_Polygon3D`: the 3D polygon of an edge.
use super::curve_representation::{
    BRepCurveRepresentation, CurveRepresentationBase, dump_curve_representation,
};
use super::dump::{dump_base, dump_class_name, dump_handle};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::poly::Polygon3D;
use rustycad_tkmath::toploc::Location;
use std::io;

/// `BRep_Polygon3D`: a located 3D polygon.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepPolygon3D};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let polygon = BRepPolygon3D::new(&Handle::default(), &Location::default());
/// assert!(polygon.is_polygon3d() && polygon.polygon3d().unwrap().is_null());
/// ```
pub struct BRepPolygon3D {
    transient: StandardTransient,
    representation: CurveRepresentationBase,
    polygon: SharedState<Handle<Polygon3D>>,
}
impl_transient!(BRepPolygon3D, "BRep_Polygon3D", dyn BRepCurveRepresentation, transient,
    interfaces: [dyn BRepCurveRepresentation]);

impl BRepPolygon3D {
    /// `BRep_Polygon3D(P, L)`; `BRepPolygon3D::new(&polygon, &location)`.
    pub fn new(polygon: &Handle<Polygon3D>, location: &Location) -> Self {
        Self {
            transient: StandardTransient::default(),
            representation: CurveRepresentationBase::new(location),
            polygon: SharedState::new(polygon.clone()),
        }
    }
}

impl BRepCurveRepresentation for BRepPolygon3D {
    fn curve_representation_base(&self) -> &CurveRepresentationBase {
        &self.representation
    }
    /// Returns true.
    fn is_polygon3d(&self) -> bool {
        true
    }
    fn polygon3d(&self) -> Result<Handle<Polygon3D>, Failure> {
        Ok(self.polygon.read().clone())
    }
    fn set_polygon3d(&self, polygon: &Handle<Polygon3D>) -> Result<(), Failure> {
        *self.polygon.write() = polygon.clone();
        Ok(())
    }
    /// A new representation on the same polygon and location.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure> {
        let copy = Self::new(&self.polygon.read(), &self.location());
        Ok(Handle::new(copy).into_cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_Polygon3D")?;
        dump_base(stream, depth, |nested, nested_depth| {
            dump_curve_representation(&self.representation, nested, nested_depth)
        })?;
        let polygon = self.polygon.read().clone();
        dump_handle(stream, depth, "Polygon3D", &polygon, |p, s, _| {
            p.dump_json(s)
        })
    }
}
