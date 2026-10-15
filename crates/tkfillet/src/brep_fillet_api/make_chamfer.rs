// Rust translation of BRepFilletAPI_MakeChamfer.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFilletAPI_MakeChamfer`: the chamfers of the edges of a shape, as a
//! `BRepBuilderAPI_MakeShape` over `ChFi3d_ChBuilder`.
use crate::chfi_ds::{ChamfMethod, ChamfMode};
use crate::chfi3d::ChFi3dChBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase;

/// `BRepFilletAPI_MakeChamfer`: the chamfers of the edges of a shape: one distance, two
/// distances (on a reference face) or a distance and an angle along each contour of
/// tangent edges.
///
/// ```
/// use rustycad_tkfillet::brep_fillet_api::{BRepFilletAPILocalOperation, BRepFilletAPIMakeChamfer};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let solid = BRepPrimAPIMakeBox::from_sizes(1., 1., 1.)?.shape()?.clone();
/// let api = BRepFilletAPIMakeChamfer::new(&solid)?;
/// assert_eq!(api.nb_contours(), 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepFilletAPIMakeChamfer {
    base: BRepBuilderAPIMakeShapeBase,
    my_builder: ChFi3dChBuilder,
    my_map: MapOfShape,
}

impl BRepFilletAPIMakeChamfer {
    /// `BRepFilletAPI_MakeChamfer(S)`; `BRepFilletAPIMakeChamfer::new(&solid)?`.
    pub fn new(s: &Shape) -> Result<Self, Failure> {
        Ok(Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_builder: ChFi3dChBuilder::new(s, 1.0e-2)?,
            my_map: MapOfShape::default(),
        })
    }

    /// The chamfer builder; `api.builder_of_chamfers().builder_base().is_done()`.
    pub fn builder_of_chamfers(&self) -> &ChFi3dChBuilder {
        &self.my_builder
    }

    /// `Add(Dis, E)`: the contour through `e` with the symmetric distance `dis`;
    /// `api.add_with_dist(2.0, &edge)?`.
    pub fn add_with_dist(&mut self, dis: f64, e: &Edge) -> Result<(), Failure> {
        self.my_builder.add_with_dist(dis, e)
    }

    /// `SetDist(Dis, IC, F)`; `api.set_dist(2.0, 1, &face)?`.
    pub fn set_dist(&mut self, dis: f64, ic: i32, f: &Face) -> Result<(), Failure> {
        self.my_builder.set_dist(dis, ic, f)
    }

    /// `GetDist(IC, Dis)`; `api.get_dist(1)?`.
    pub fn get_dist(&self, ic: i32) -> Result<f64, Failure> {
        self.my_builder.get_dist(ic)
    }

    /// `Add(Dis1, Dis2, E, F)`: the contour through `e` with `dis1` on the face `f` and
    /// `dis2` on the other; `api.add_with_dists(1.0, 3.0, &edge, &face)?`.
    pub fn add_with_dists(
        &mut self,
        dis1: f64,
        dis2: f64,
        e: &Edge,
        f: &Face,
    ) -> Result<(), Failure> {
        self.my_builder.add_with_dists(dis1, dis2, e, f)
    }

    /// `SetDists(Dis1, Dis2, IC, F)`; `api.set_dists(1.0, 3.0, 1, &face)?`.
    pub fn set_dists(&mut self, dis1: f64, dis2: f64, ic: i32, f: &Face) -> Result<(), Failure> {
        self.my_builder.set_dists(dis1, dis2, ic, f)
    }

    /// `Dists(IC, Dis1, Dis2)`; `api.dists(1)?`.
    pub fn dists(&self, ic: i32) -> Result<(f64, f64), Failure> {
        self.my_builder.dists(ic)
    }

    /// `AddDA(Dis, Angle, E, F)`; `api.add_dist_angle(1.0, 0.5, &edge, &face)?`.
    pub fn add_dist_angle(
        &mut self,
        dis: f64,
        angle: f64,
        e: &Edge,
        f: &Face,
    ) -> Result<(), Failure> {
        self.my_builder.add_dist_angle(dis, angle, e, f)
    }

    /// `SetDistAngle(Dis, Angle, IC, F)`; `api.set_dist_angle(1.0, 0.5, 1, &face)?`.
    pub fn set_dist_angle(
        &mut self,
        dis: f64,
        angle: f64,
        ic: i32,
        f: &Face,
    ) -> Result<(), Failure> {
        self.my_builder.set_dist_angle(dis, angle, ic, f)
    }

    /// `GetDistAngle(IC, Dis, Angle)`; `api.get_dist_angle(1)?`.
    pub fn get_dist_angle(&self, ic: i32) -> Result<(f64, f64), Failure> {
        self.my_builder.get_dist_angle(ic)
    }

    /// `SetMode(theMode)`: the mode of the chamfers added next.
    pub fn set_mode(&mut self, mode: ChamfMode) {
        self.my_builder.set_mode(mode);
    }

    /// `IsSymetric(IC)`; `api.is_symetric(1)?`.
    pub fn is_symetric(&self, ic: i32) -> Result<bool, Failure> {
        Ok(self.my_builder.is_chamfer(ic)? == ChamfMethod::Sym)
    }

    /// `IsTwoDistances(IC)`; `api.is_two_distances(1)?`.
    pub fn is_two_distances(&self, ic: i32) -> Result<bool, Failure> {
        Ok(self.my_builder.is_chamfer(ic)? == ChamfMethod::TwoDist)
    }

    /// `IsDistanceAngle(IC)`; `api.is_distance_angle(1)?`.
    pub fn is_distance_angle(&self, ic: i32) -> Result<bool, Failure> {
        Ok(self.my_builder.is_chamfer(ic)? == ChamfMethod::DistAngle)
    }
}

super::local_api::impl_local_api!(BRepFilletAPIMakeChamfer);
