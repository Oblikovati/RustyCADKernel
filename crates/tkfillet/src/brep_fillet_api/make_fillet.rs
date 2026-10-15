// Rust translation of BRepFilletAPI_MakeFillet.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFilletAPI_MakeFillet`: the fillets of the edges of a shape, as a
//! `BRepBuilderAPI_MakeShape` over `ChFi3d_FilBuilder`.
use crate::chfi_ds::ErrorStatus;
use crate::chfi3d::{ChFi3dFilBuilder, FilletShape};
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase;

/// `BRepFilletAPI_MakeFillet`: the fillets of the edges of a shape: constant, two
/// radii, a law or `(parameter, radius)` pairs along each contour of tangent edges.
///
/// ```
/// use rustycad_tkfillet::brep_fillet_api::{BRepFilletAPILocalOperation, BRepFilletAPIMakeFillet};
/// use rustycad_tkfillet::chfi3d::FilletShape;
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let solid = BRepPrimAPIMakeBox::from_sizes(1., 1., 1.)?.shape()?.clone();
/// let api = BRepFilletAPIMakeFillet::new(&solid, FilletShape::Rational)?;
/// assert_eq!(api.nb_contours(), 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepFilletAPIMakeFillet {
    base: BRepBuilderAPIMakeShapeBase,
    my_builder: ChFi3dFilBuilder,
    my_map: MapOfShape,
}

impl BRepFilletAPIMakeFillet {
    /// `BRepFilletAPI_MakeFillet(S, FShape)`; `BRepFilletAPIMakeFillet::new(&solid,
    /// FilletShape::Rational)?`.
    pub fn new(s: &Shape, fshape: FilletShape) -> Result<Self, Failure> {
        Ok(Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_builder: ChFi3dFilBuilder::new(s, fshape, 1.0e-2)?,
            my_map: MapOfShape::default(),
        })
    }

    /// The fillet builder; `api.builder_of_fillets().builder_base().is_done()`.
    pub fn builder_of_fillets(&self) -> &ChFi3dFilBuilder {
        &self.my_builder
    }

    /// `SetParams(Tang, Tesp, T2d, TApp3d, TolApp2d, Fleche)`.
    pub fn set_params(
        &mut self,
        tang: f64,
        tesp: f64,
        t2d: f64,
        tapp3d: f64,
        tolapp2d: f64,
        fleche: f64,
    ) {
        self.my_builder
            .builder_base_mut()
            .set_params(tang, tesp, t2d, tapp3d, tolapp2d, fleche);
    }

    /// `SetContinuity(InternalContinuity, AngularTolerance)`.
    pub fn set_continuity(&mut self, internal_continuity: Continuity, angle_tol: f64) {
        self.my_builder
            .builder_base_mut()
            .set_continuity(internal_continuity, angle_tol);
    }

    /// `Add(Radius, E)`: the contour through `e` with the constant radius `radius`;
    /// `api.add_with_radius(2.0, &edge)?`.
    pub fn add_with_radius(&mut self, radius: f64, e: &Edge) -> Result<(), Failure> {
        let (ic, iinc) = self.add_and_find(e)?;
        if ic != 0 {
            self.set_radius_on_contour(radius, ic, iinc)?;
        }
        Ok(())
    }

    /// `Add(R1, R2, E)`: the contour through `e` with the radius evolving linearly from
    /// `r1` to `r2` on `e`; `api.add_with_radii(1.0, 3.0, &edge)?`.
    pub fn add_with_radii(&mut self, r1: f64, r2: f64, e: &Edge) -> Result<(), Failure> {
        let (ic, iinc) = self.add_and_find(e)?;
        if ic != 0 {
            self.set_radii(r1, r2, ic, iinc)?;
        }
        Ok(())
    }

    /// `Add(L, E)`: the contour through `e` with the law `l` of the radius;
    /// `api.add_with_law(&law, &edge)?`.
    pub fn add_with_law(&mut self, l: &Handle<dyn LawFunction>, e: &Edge) -> Result<(), Failure> {
        let (ic, iinc) = self.add_and_find(e)?;
        if ic != 0 {
            self.set_radius_law(l, ic, iinc)?;
        }
        Ok(())
    }

    /// `Add(UandR, E)`: the contour through `e` with the radii at the parameters;
    /// `api.add_with_pairs(&[Point2::new(0., 1.), Point2::new(1., 2.)], &edge)?`.
    pub fn add_with_pairs(&mut self, u_and_r: &[Point2], e: &Edge) -> Result<(), Failure> {
        let (ic, iinc) = self.add_and_find(e)?;
        if ic != 0 {
            self.set_radius_pairs(u_and_r, ic, iinc)?;
        }
        Ok(())
    }

    /// `myBuilder.Add(E)` and `myBuilder.Contains(E, IinC)`.
    fn add_and_find(&mut self, e: &Edge) -> Result<(i32, i32), Failure> {
        self.my_builder.add(e)?;
        self.my_builder.builder_base().contains_with_index(e)
    }

    /// `SetRadius(Radius, IC, IinC)`: the constant radius on the edge `iinc` of the
    /// contour `ic`; `api.set_radius_on_contour(2.0, 1, 1)?`.
    pub fn set_radius_on_contour(
        &mut self,
        radius: f64,
        ic: i32,
        iinc: i32,
    ) -> Result<(), Failure> {
        self.my_builder
            .set_radius_at(Xy::new(0., radius), ic, iinc)?;
        self.my_builder.set_radius_at(Xy::new(1., radius), ic, iinc)
    }

    /// `SetRadius(R1, R2, IC, IinC)`: the radius from `r1` to `r2` on the edge `iinc` of
    /// the contour `ic` (their mean when they are equal within the confusion);
    /// `api.set_radii(1.0, 3.0, 1, 1)?`.
    pub fn set_radii(&mut self, r1: f64, r2: f64, ic: i32, iinc: i32) -> Result<(), Failure> {
        let (r1, r2) = if (r1 - r2).abs() < precision::confusion() {
            let m = (r1 + r2) * 0.5;
            (m, m)
        } else {
            (r1, r2)
        };
        self.my_builder.set_radius_at(Xy::new(0., r1), ic, iinc)?;
        self.my_builder.set_radius_at(Xy::new(1., r2), ic, iinc)
    }

    /// `SetRadius(L, IC, IinC)`: the law of the radius of the contour `ic`;
    /// `api.set_radius_law(&law, 1, 1)?`.
    pub fn set_radius_law(
        &mut self,
        l: &Handle<dyn LawFunction>,
        ic: i32,
        iinc: i32,
    ) -> Result<(), Failure> {
        self.my_builder.set_radius_law(l, ic, iinc)
    }

    /// `SetRadius(UandR, IC, IinC)`: one radius, two radii, or the radii at the
    /// parameters rescaled to [0, 1]; `api.set_radius_pairs(&pairs, 1, 1)?`.
    pub fn set_radius_pairs(
        &mut self,
        u_and_r: &[Point2],
        ic: i32,
        iinc: i32,
    ) -> Result<(), Failure> {
        match u_and_r {
            [one] => self.set_radius_on_contour(one.y(), ic, iinc),
            [first, last] => self.set_radii(first.y(), last.y(), ic, iinc),
            _ => {
                let (Some(f), Some(l)) = (u_and_r.first(), u_and_r.last()) else {
                    return Ok(());
                };
                let (uf, ul) = (f.x(), l.x());
                for p in u_and_r {
                    let ucur = (p.x() - uf) / (ul - uf);
                    self.my_builder
                        .set_radius_at(Xy::new(ucur, p.y()), ic, iinc)?;
                }
                Ok(())
            }
        }
    }

    /// `IsConstant(IC)`; `api.is_constant(1)?`.
    pub fn is_constant(&self, ic: i32) -> Result<bool, Failure> {
        self.my_builder.is_constant(ic)
    }

    /// `Radius(IC)`; `api.radius(1)?`.
    pub fn radius(&self, ic: i32) -> Result<f64, Failure> {
        self.my_builder.radius(ic)
    }

    /// `IsConstant(IC, E)`; `api.is_constant_on_edge(1, &edge)?`.
    pub fn is_constant_on_edge(&self, ic: i32, e: &Edge) -> Result<bool, Failure> {
        self.my_builder.is_constant_on_edge(ic, e)
    }

    /// `Radius(IC, E)`; `api.radius_on_edge(1, &edge)?`.
    pub fn radius_on_edge(&self, ic: i32, e: &Edge) -> Result<f64, Failure> {
        self.my_builder.radius_on_edge(ic, e)
    }

    /// `SetRadius(Radius, IC, E)`; `api.set_radius_on_edge(2.0, 1, &edge)?`.
    pub fn set_radius_on_edge(&mut self, radius: f64, ic: i32, e: &Edge) -> Result<(), Failure> {
        self.my_builder.set_radius_on_edge(radius, ic, e)
    }

    /// `SetRadius(Radius, IC, V)`; `api.set_radius_on_vertex(2.0, 1, &v)?`.
    pub fn set_radius_on_vertex(
        &mut self,
        radius: f64,
        ic: i32,
        v: &Vertex,
    ) -> Result<(), Failure> {
        self.my_builder.set_radius_on_vertex(radius, ic, v)
    }

    /// `GetBounds(IC, E, F, L)`: the bounds of the law on the edge, None without one;
    /// `api.get_bounds(1, &edge)?`.
    pub fn get_bounds(&self, ic: i32, e: &Edge) -> Result<Option<(f64, f64)>, Failure> {
        self.my_builder.get_bounds(ic, e)
    }

    /// `GetLaw(IC, E)`; `api.get_law(1, &edge)?`.
    pub fn get_law(&self, ic: i32, e: &Edge) -> Result<Handle<dyn LawFunction>, Failure> {
        self.my_builder.get_law(ic, e)
    }

    /// `SetLaw(IC, E, L)`; `api.set_law(1, &edge, &law)?`.
    pub fn set_law(
        &mut self,
        ic: i32,
        e: &Edge,
        l: &Handle<dyn LawFunction>,
    ) -> Result<(), Failure> {
        self.my_builder.set_law(ic, e, l)
    }

    /// `SetFilletShape(FShape)`.
    pub fn set_fillet_shape(&mut self, fshape: FilletShape) {
        self.my_builder.set_fillet_shape(fshape);
    }

    /// `GetFilletShape()`.
    pub fn get_fillet_shape(&self) -> FilletShape {
        self.my_builder.get_fillet_shape()
    }

    /// `NbSurfaces()`: the number of surfaces of the data structure.
    pub fn nb_surfaces(&self) -> i32 {
        self.builder().data_structure().nb_surfaces()
    }

    /// `NewFaces(I)`: the faces built on the surface `i`; `api.new_faces(1)?`.
    pub fn new_faces(&self, i: i32) -> Result<List<Shape>, Failure> {
        self.builder().new_faces(i)
    }

    /// `NbFaultyContours()`.
    pub fn nb_faulty_contours(&self) -> i32 {
        self.my_builder.builder_base().nb_faulty_contours()
    }

    /// `FaultyContour(I)`.
    pub fn faulty_contour(&self, i: i32) -> i32 {
        self.my_builder.builder_base().faulty_contour(i)
    }

    /// `NbComputedSurfaces(IC)`.
    pub fn nb_computed_surfaces(&self, ic: i32) -> i32 {
        self.my_builder.builder_base().nb_computed_surfaces(ic)
    }

    /// `ComputedSurface(IC, IS)`.
    pub fn computed_surface(&self, ic: i32, is: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
        self.my_builder.builder_base().computed_surface(ic, is)
    }

    /// `NbFaultyVertices()`.
    pub fn nb_faulty_vertices(&self) -> i32 {
        self.my_builder.builder_base().nb_faulty_vertices()
    }

    /// `FaultyVertex(IV)`.
    pub fn faulty_vertex(&self, iv: i32) -> Result<Vertex, Failure> {
        self.my_builder.builder_base().faulty_vertex(iv)
    }

    /// `HasResult()`.
    pub fn has_result(&self) -> bool {
        self.my_builder.builder_base().has_result()
    }

    /// `BadShape()`.
    pub fn bad_shape(&self) -> Result<Shape, Failure> {
        self.my_builder.builder_base().bad_shape()
    }

    /// `StripeStatus(IC)`.
    pub fn stripe_status(&self, ic: i32) -> Result<ErrorStatus, Failure> {
        self.my_builder.builder_base().stripe_status(ic)
    }
}

super::local_api::impl_local_api!(BRepFilletAPIMakeFillet);
