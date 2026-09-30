// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepLib/BRepLib_PointCloudShape.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_PointCloudShape`: the abstract generator of a point cloud on the faces of a
//! shape, by density (random points of the faces) or by the nodes of their meshes. A
//! derived class receives each point through the pure virtual `addPoint`.
use crate::brep_gprop::BRepGProp;
use crate::brep_lib::BRepLibToolTriangulatedShape;
use crate::brep_top_adaptor::BRepTopAdaptorFClass2d;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, ElementCast};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::stl::{MersenneTwister, UniformRealDistribution};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{self, Direction3, Point2, Point3, Transform3, Vector3};
use rustycad_tkmath::poly::mesh_purpose;

/// The protected data of `BRepLib_PointCloudShape` (docs/topalgo-design.md section 14),
/// public because the derived classes live in other crates. Every implementor of
/// [`BRepLibPointCloudShape`] embeds one.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tktopalgo::brep_lib::BRepLibPointCloudShapeBase;
/// let base = BRepLibPointCloudShapeBase::new(Shape::default(), 1.0e-7);
/// assert_eq!(base.my_dist, 0.0);
/// assert_eq!(base.my_tol, 1.0e-7);
/// ```
#[derive(Clone, Default)]
pub struct BRepLibPointCloudShapeBase {
    /// `myShape`: the shape whose faces get the points.
    pub my_shape: Shape,
    /// `myDist`: the largest random offset of a density point along the normal.
    pub my_dist: f64,
    /// `myTol`: faces smaller than its square are skipped by the default density.
    pub my_tol: f64,
    /// `myFaceArea`: the area of each face met so far.
    pub my_face_area: DataMap<Shape, f64, ShapeMapHasher>,
    /// `myFacePoints`: the number of density points of each face.
    pub my_face_points: DataMap<Shape, i32, ShapeMapHasher>,
    /// `myNbPoints`: set to 0 by the constructor and never used by the class.
    pub my_nb_points: i32,
}

impl BRepLibPointCloudShapeBase {
    /// `BRepLib_PointCloudShape(theShape, theTol)` (defaults: a null shape and
    /// `Precision::Confusion()`); `BRepLibPointCloudShapeBase::new(shape, 1.0e-7)`.
    pub fn new(shape: Shape, tol: f64) -> Self {
        Self {
            my_shape: shape,
            my_tol: tol,
            ..Self::default()
        }
    }

    /// The base `clear()`: empties the area and point-count maps; `base.clear()`.
    pub fn clear(&mut self) {
        self.my_face_area.clear(false);
        self.my_face_points.clear(false);
    }
}

/// `BRepLib_PointCloudShape` (docs/topalgo-design.md section 14): the public and
/// protected members of the abstract class as provided methods over the embedded
/// [`BRepLibPointCloudShapeBase`]; an implementor provides `add_point` (pure virtual
/// natively) and may override `clear` (virtual).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkmath::gp::{Point2, Point3, Vector3};
/// use rustycad_tktopalgo::brep_lib::{BRepLibPointCloudShape, BRepLibPointCloudShapeBase};
/// struct Cloud(BRepLibPointCloudShapeBase, Vec<Point3>);
/// impl BRepLibPointCloudShape for Cloud {
///     fn point_cloud_base(&self) -> &BRepLibPointCloudShapeBase { &self.0 }
///     fn point_cloud_base_mut(&mut self) -> &mut BRepLibPointCloudShapeBase { &mut self.0 }
///     fn add_point(&mut self, point: Point3, _: Vector3, _: Point2, _: &Shape) { self.1.push(point) }
/// }
/// let mut cloud = Cloud(BRepLibPointCloudShapeBase::new(Shape::default(), 1.0e-7), Vec::new());
/// assert!(!cloud.generate_points_by_triangulation()?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait BRepLibPointCloudShape {
    /// The embedded protected data.
    fn point_cloud_base(&self) -> &BRepLibPointCloudShapeBase;

    /// The embedded protected data, to write.
    fn point_cloud_base_mut(&mut self) -> &mut BRepLibPointCloudShapeBase;

    /// `addPoint(thePoint, theNorm, theUV, theFace)` (protected, pure virtual): receives
    /// one point, the normal there, its UV parameters and its face.
    fn add_point(&mut self, point: Point3, normal: Vector3, uv: Point2, face: &Shape);

    /// `clear()` (protected, virtual): the base empties the two maps; `cloud.clear()`.
    fn clear(&mut self) {
        self.point_cloud_base_mut().clear();
    }

    /// `Shape()`: the loaded shape; `cloud.shape()`.
    fn shape(&self) -> &Shape {
        &self.point_cloud_base().my_shape
    }

    /// `SetShape(theShape)`: replaces the shape (the maps are kept);
    /// `cloud.set_shape(&shape)`.
    fn set_shape(&mut self, shape: &Shape) {
        self.point_cloud_base_mut().my_shape = shape.clone();
    }

    /// `Tolerance()`; `cloud.tolerance()`.
    fn tolerance(&self) -> f64 {
        self.point_cloud_base().my_tol
    }

    /// `SetTolerance(theTol)`; `cloud.set_tolerance(1.0e-7)`.
    fn set_tolerance(&mut self, tol: f64) {
        self.point_cloud_base_mut().my_tol = tol;
    }

    /// `GetDistance()`: the largest offset along the normal (0 by default);
    /// `cloud.get_distance()`.
    fn get_distance(&self) -> f64 {
        self.point_cloud_base().my_dist
    }

    /// `SetDistance(theDist)`: stored as given (a negative or tiny value disables the
    /// offset, since it is only used above `Precision::Confusion()`);
    /// `cloud.set_distance(0.5)`.
    fn set_distance(&mut self, dist: f64) {
        self.point_cloud_base_mut().my_dist = dist;
    }

    /// `NbPointsByDensity(theDensity)`: clears the maps and counts the points of each
    /// face, `max(ceil(area / theDensity), 1)`, 0 when the density (given, or computed
    /// when below `Precision::Confusion()`) is below `Precision::Confusion()`. The count
    /// divides by the given density, not the computed one (see the quirk in
    /// docs/tktopalgo-progress.md); `cloud.nb_points_by_density(0.5)?`.
    fn nb_points_by_density(&mut self, density: f64) -> Result<i32, Failure> {
        self.clear();
        let used: f64 = match density < precision::confusion() {
            true => self.compute_density()?,
            false => density,
        };
        if used < precision::confusion() {
            return Ok(0);
        }
        let mut total: i32 = 0;
        for face in Explorer::new(self.shape(), ShapeEnum::Face, ShapeEnum::Shape)? {
            let area: f64 = self.face_area(&face)?;
            let count: i32 = ElementCast::<i32>::cast((area / density).ceil()).max(1);
            self.point_cloud_base_mut().my_face_points.bind(face, count);
            total = total.wrapping_add(count);
        }
        Ok(total)
    }

    /// `NbPointsByTriangulation()`: the node count of the face triangulations;
    /// `cloud.nb_points_by_triangulation()?`.
    fn nb_points_by_triangulation(&self) -> Result<i32, Failure> {
        let mut total: i32 = 0;
        for face in Explorer::new(self.shape(), ShapeEnum::Face, ShapeEnum::Shape)? {
            let (mesh, _) = BRepTool::triangulation(&topods::face(&face)?, mesh_purpose::NONE)?;
            if let Some(mesh) = mesh.get() {
                total = total.wrapping_add(mesh.nb_nodes());
            }
        }
        Ok(total)
    }

    /// `GeneratePointsByDensity(theDensity)`: counts the points first when no face has
    /// a count (false when there are none), then adds the counted points of each face;
    /// true when a face got points. Counts left by an earlier call are reused whatever
    /// the density; `cloud.generate_points_by_density(0.0)?`.
    fn generate_points_by_density(&mut self, density: f64) -> Result<bool, Failure> {
        if self.point_cloud_base().my_face_points.is_empty()
            && self.nb_points_by_density(density)? == 0
        {
            return Ok(false);
        }
        let mut added: i32 = 0;
        for face in Explorer::new(&self.shape().clone(), ShapeEnum::Face, ShapeEnum::Shape)? {
            if self.add_density_points(&face)? {
                added += 1;
            }
        }
        Ok(added > 0)
    }

    /// `GeneratePointsByTriangulation()`: clears the maps and adds the nodes of each
    /// face triangulation; true when a face has one;
    /// `cloud.generate_points_by_triangulation()?`.
    fn generate_points_by_triangulation(&mut self) -> Result<bool, Failure> {
        self.clear();
        let mut added: i32 = 0;
        for face in Explorer::new(&self.shape().clone(), ShapeEnum::Face, ShapeEnum::Shape)? {
            if self.add_triangulation_points(&face)? {
                added += 1;
            }
        }
        Ok(added > 0)
    }

    /// `faceArea(theShape)` (protected): the cached area, else
    /// `BRepGProp::SurfaceProperties` of the face, cached; `cloud.face_area(&face)?`.
    fn face_area(&mut self, face: &Shape) -> Result<f64, Failure> {
        if let Some(area) = self.point_cloud_base().my_face_area.seek(face) {
            return Ok(*area);
        }
        let mut props = GProps::new();
        BRepGProp::surface_properties(face, &mut props, false, false)?;
        let area: f64 = props.mass();
        self.point_cloud_base_mut()
            .my_face_area
            .bind(face.clone(), area);
        Ok(area)
    }

    /// `computeDensity()` (protected): a tenth of the smallest face area not below the
    /// square of the tolerance (`Precision::Infinite()` tenth without such a face);
    /// `cloud.compute_density()?`.
    fn compute_density(&mut self) -> Result<f64, Failure> {
        let mut area_min: f64 = precision::infinite();
        let tol: f64 = self.tolerance();
        for face in Explorer::new(&self.shape().clone(), ShapeEnum::Face, ShapeEnum::Shape)? {
            let area: f64 = self.face_area(&face)?;
            if area < tol * tol {
                continue;
            }
            if area < area_min {
                area_min = area;
            }
        }
        Ok(area_min * 0.1)
    }

    /// `addDensityPoints(theFace)` (protected): the counted number of random points
    /// inside the face (drawn as the private `DensityDraw` does); false when the face has no count or no
    /// surface; `cloud.add_density_points(&face)?`.
    fn add_density_points(&mut self, face_shape: &Shape) -> Result<bool, Failure> {
        let count: i32 = self
            .point_cloud_base()
            .my_face_points
            .seek(face_shape)
            .copied()
            .unwrap_or(0);
        if count == 0 {
            return Ok(false);
        }
        let face: Face = topods::face(face_shape)?;
        let Some(mut draw) = DensityDraw::new(&face, self.get_distance())? else {
            return Ok(false);
        };
        let mut drawn: i32 = 1;
        while drawn <= count {
            let Some((point, normal, uv)) = draw.next_point()? else {
                continue;
            };
            drawn += 1;
            self.add_point(point, normal, uv, face_shape);
        }
        Ok(true)
    }

    /// `addTriangulationPoints(theFace)` (protected): computes the normals of the face
    /// triangulation (stored in it) and adds each node with its normal and UV node,
    /// moved by the triangulation location; false without a triangulation;
    /// `cloud.add_triangulation_points(&face)?`.
    fn add_triangulation_points(&mut self, face_shape: &Shape) -> Result<bool, Failure> {
        let face: Face = topods::face(face_shape)?;
        let (mesh, location) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
        if mesh.is_null() {
            return Ok(false);
        }
        let transform: Transform3 = location.transformation();
        BRepLibToolTriangulatedShape::compute_normals(&face, &mesh)?;
        let Some(nodes) = mesh.get() else {
            return Ok(false);
        };
        let has_uv: bool = nodes.has_uv_nodes();
        for index in 1..=nodes.nb_nodes() {
            let (mut point, mut normal) = (nodes.node(index)?, nodes.normal(index)?);
            if !location.is_identity() {
                point = point.transformed(transform);
                normal = normal.transformed(transform);
            }
            let uv: Point2 = if has_uv {
                nodes.uv_node(index)?
            } else {
                Point2::default()
            };
            self.add_point(point, Vector3::from(normal), uv, face_shape);
        }
        Ok(true)
    }
}

/// The state of one `addDensityPoints` call: the random generator (`std::mt19937`
/// seeded with 0 for each face), the UV ranges, the classifier, the surface and the
/// face location.
struct DensityDraw {
    generator: MersenneTwister,
    u_range: UniformRealDistribution,
    v_range: UniformRealDistribution,
    classifier: BRepTopAdaptorFClass2d,
    surface: rustycad_tkernel::standard::Handle<dyn GeomSurface>,
    transform: Transform3,
    reversed: bool,
    distance: f64,
}

impl DensityDraw {
    /// The draw on `face` (its `BRepTools::UVBounds`, a classifier of tolerance
    /// `Precision::Confusion()`, the transformation of the face's own location), or
    /// `None` when the face has no surface; `DensityDraw::new(&face, 0.0)?`.
    fn new(face: &Face, distance: f64) -> Result<Option<Self>, Failure> {
        let (u_min, u_max, v_min, v_max) = brep_tools::uv_bounds(face)?;
        let classifier = BRepTopAdaptorFClass2d::new(face, precision::confusion())?;
        let transform: Transform3 = face.location().transformation();
        let (surface, _) = BRepTool::surface_with_location(face)?;
        if surface.is_null() {
            return Ok(None);
        }
        Ok(Some(Self {
            generator: MersenneTwister::new(0),
            u_range: UniformRealDistribution::new(u_min, u_max),
            v_range: UniformRealDistribution::new(v_min, v_max),
            classifier,
            surface,
            transform,
            reversed: face.orientation() == Orientation::Reversed,
            distance,
        }))
    }

    /// One draw: a (u, v) pair (u first), `None` when it classifies OUT; else the
    /// surface point (moved along the normal by a random distance in `[0, myDist)` when
    /// `myDist` exceeds `Precision::Confusion()`) and the unit normal (reversed on a
    /// REVERSED face), both moved by the face location, and the UV pair;
    /// `draw.next_point()?`.
    fn next_point(&mut self) -> Result<Option<(Point3, Vector3, Point2)>, Failure> {
        let u: f64 = self.u_range.sample(&mut self.generator);
        let v: f64 = self.v_range.sample(&mut self.generator);
        let uv = Point2::new(u, v);
        if self.classifier.perform(uv, true)? == State::Out {
            return Ok(None);
        }
        let surface = self.surface.get().ok_or_else(null_surface)?;
        let d1 = surface.d1(u, v)?;
        let mut normal: Vector3 = d1.d1u.crossed(d1.d1v);
        if self.reversed {
            normal.reverse();
        }
        let magnitude: f64 = normal.magnitude();
        if magnitude > gp::resolution() {
            normal /= magnitude;
        }
        let mut point: Point3 = d1.point;
        if self.distance > precision::confusion() {
            let offset = UniformRealDistribution::new(0.0, self.distance);
            let shift: f64 = offset.sample(&mut self.generator);
            point.set_xyz(point.xyz() + *normal.xyz() * shift);
        }
        point = point.transformed(self.transform);
        if magnitude > gp::resolution() {
            normal = Vector3::from(Direction3::from_vector(normal)?.transformed(self.transform));
        }
        Ok(Some((point, normal, uv)))
    }
}

/// The surface handle checked in `DensityDraw::new` is never null afterwards.
fn null_surface() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepLib_PointCloudShape::addDensityPoints: null surface; expected the face surface",
        "",
    )
}
