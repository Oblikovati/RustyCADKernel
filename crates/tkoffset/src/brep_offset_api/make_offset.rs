// Rust translation of ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_MakeOffset.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffsetAPI_MakeOffset`: the parallels of the wires of a planar face (or of
//! planar wires) at a distance, built by one `BRepFill_OffsetWire` per domain.
use super::make_offset_domains::{OffsetDomainOptions, build_domains};
use rustycad_tkbool::brep_algo::convert_wire;
use rustycad_tkbool::brep_fill::BRepFillOffsetWire;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Compound, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::{CurveType, JoinType};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIFaceError, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeShapeBase,
};
use rustycad_tktopalgo::brep_lib::build_curves3d_default;

/// The angular tolerance of the arcs-and-segments approximation of `SetApprox`.
const APPROX_ANGLE_TOLERANCE: f64 = 0.1;

/// `BRepOffsetAPI_MakeOffset`: offsets the wires of a planar face, or planar wires,
/// on the side the sign of the distance selects (negative: inside, the left side
/// of the face boundary).
///
/// ```
/// use rustycad_tkmath::geom_abs::JoinType;
/// use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3};
/// use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakeOffset;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire};
/// let axis = Axis2::new(Point3::new(0.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0)?)?;
/// let edge = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(axis, 1.0)?)?.edge()?;
/// let wire = BRepBuilderAPIMakeWire::from_edge(&edge)?.wire()?;
/// let mut maker = BRepOffsetAPIMakeOffset::new();
/// maker.init(JoinType::Intersection, false);
/// maker.add_wire(&wire);
/// maker.perform(0.5, 0.0)?;
/// assert!(maker.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepOffsetAPIMakeOffset {
    base: BRepBuilderAPIMakeShapeBase,
    my_is_initialized: bool,
    my_last_is_left: bool,
    my_join: JoinType,
    my_is_open_result: bool,
    my_is_to_approx: bool,
    my_face: Face,
    my_wires: Vec<Shape>,
    my_left: Vec<BRepFillOffsetWire>,
    my_right: Vec<BRepFillOffsetWire>,
}

impl Default for BRepOffsetAPIMakeOffset {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepOffsetAPIMakeOffset {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build()`: marks the algorithm done (the work is in `Perform`).
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        self.base.set_done();
        Ok(())
    }

    /// `Generated(S)`: the parallels of `S` (in both orientations) in every domain of
    /// the side of the last `Perform`.
    fn generated(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated.clear();
        let algos = if self.my_last_is_left {
            &mut self.my_left
        } else {
            &mut self.my_right
        };
        for algo in algos.iter_mut() {
            for orientation in [Orientation::Forward, Orientation::Reversed] {
                let parallels = algo.generated_shapes(&shape.oriented(orientation))?;
                for parallel in parallels.iter() {
                    self.base.my_generated.append(parallel.clone());
                }
            }
        }
        Ok(&self.base.my_generated)
    }
}

impl BRepOffsetAPIMakeOffset {
    /// `BRepOffsetAPI_MakeOffset()`: no spine, the arc join and a closed result;
    /// `BRepOffsetAPIMakeOffset::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_is_initialized: false,
            my_last_is_left: false,
            my_join: JoinType::Arc,
            my_is_open_result: false,
            my_is_to_approx: false,
            my_face: Face::default(),
            my_wires: Vec::new(),
            my_left: Vec::new(),
            my_right: Vec::new(),
        }
    }

    /// `BRepOffsetAPI_MakeOffset(Spine, Join, IsOpenResult)` of a face;
    /// `BRepOffsetAPIMakeOffset::from_face(&face, JoinType::Arc, false)`.
    pub fn from_face(spine: &Face, join: JoinType, is_open_result: bool) -> Result<Self, Failure> {
        let mut maker = Self::new();
        maker.init_face(spine, join, is_open_result)?;
        Ok(maker)
    }

    /// `BRepOffsetAPI_MakeOffset(Spine, Join, IsOpenResult)` of a wire;
    /// `BRepOffsetAPIMakeOffset::from_wire(&wire, JoinType::Arc, false)`.
    pub fn from_wire(spine: &Wire, join: JoinType, is_open_result: bool) -> Self {
        let mut maker = Self::new();
        maker.my_wires.push(spine.as_ref().clone());
        maker.my_is_initialized = true;
        maker.my_join = join;
        maker.my_is_open_result = is_open_result;
        maker
    }

    /// `Init(Spine, Join, IsOpenResult)`: the face and its wires (appended to the
    /// wires already given); `maker.init_face(&face, JoinType::Arc, false)?`.
    pub fn init_face(
        &mut self,
        spine: &Face,
        join: JoinType,
        is_open_result: bool,
    ) -> Result<(), Failure> {
        self.my_face = spine.clone();
        self.my_is_initialized = true;
        self.my_join = join;
        self.my_is_open_result = is_open_result;
        self.my_is_to_approx = false;
        let mut explorer = rustycad_tkbrep::topexp::Explorer::new(
            self.my_face.as_ref(),
            rustycad_tkg3d::top_abs::ShapeEnum::Wire,
            rustycad_tkg3d::top_abs::ShapeEnum::Shape,
        )?;
        while explorer.more() {
            self.my_wires.push(explorer.current().clone());
            explorer.next()?;
        }
        Ok(())
    }

    /// `Init(Join, IsOpenResult)`: the join type and the open-result flag only;
    /// `maker.init(JoinType::Intersection, false)`.
    pub fn init(&mut self, join: JoinType, is_open_result: bool) {
        self.my_join = join;
        self.my_is_open_result = is_open_result;
    }

    /// `SetApprox(ToApprox)`: whether `Perform` first converts the contours into 2D
    /// arcs and segments; `maker.set_approx(true)`.
    pub fn set_approx(&mut self, to_approx: bool) {
        self.my_is_to_approx = to_approx;
    }

    /// `AddWire(Spine)`: one more wire to offset; `maker.add_wire(&wire)`.
    pub fn add_wire(&mut self, spine: &Wire) {
        self.my_is_initialized = true;
        self.my_wires.push(spine.as_ref().clone());
    }

    /// `Perform(Offset, Alt)`: the parallels at the distance (inside for a negative
    /// one) and at the altitude `Alt` along the normal; a failure of the domain
    /// building or of an offset leaves the algorithm not done with a null shape, as
    /// the native catch does. StdFail_NotDone without a spine;
    /// `maker.perform(0.5, 0.0)?`.
    pub fn perform(&mut self, offset: f64, alt: f64) -> Result<(), Failure> {
        if !self.my_is_initialized {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "BRepOffsetAPI_MakeOffset : Perform without Init",
            ));
        }
        if self.perform_domains(offset, alt).is_err() {
            self.base.set_not_done();
            self.base.my_shape.nullify();
        }
        Ok(())
    }

    /// The body of the native `try` block of `Perform`.
    fn perform_domains(&mut self, offset: f64, alt: f64) -> Result<(), Failure> {
        if self.my_is_to_approx {
            self.approximate_contours()?;
        }
        let is_left = offset <= 0.;
        self.my_last_is_left = is_left;
        let is_was_reversed = self.build_side_domains(!is_left)?;
        let mut result = Compound::default();
        BRepBuilder::default().make_compound(&mut result);
        let distance = if is_left { offset.abs() } else { offset };
        let count = self.perform_side(distance, alt, is_was_reversed, &mut result)?;
        if count > 1 {
            self.base.my_shape = result.into_shape();
        }
        if self.base.my_shape.is_null() {
            self.base.set_not_done();
        } else {
            self.base.set_done();
        }
        Ok(())
    }

    /// `BuildDomains` of the side when its algorithms do not exist yet; whether the
    /// wires were reversed (false when the domains already existed).
    fn build_side_domains(&mut self, is_positive: bool) -> Result<bool, Failure> {
        let algos = if is_positive {
            &mut self.my_right
        } else {
            &mut self.my_left
        };
        if !algos.is_empty() {
            return Ok(false);
        }
        let options = OffsetDomainOptions {
            join: self.my_join,
            is_open_result: self.my_is_open_result,
            is_positive,
        };
        build_domains(&mut self.my_face, &mut self.my_wires, algos, options)
    }

    /// Performs every algorithm of the side; the first offset shape becomes the
    /// result and all go into the compound; the number of offset shapes.
    fn perform_side(
        &mut self,
        distance: f64,
        alt: f64,
        is_was_reversed: bool,
        result: &mut Compound,
    ) -> Result<usize, Failure> {
        let algos = if self.my_last_is_left {
            &mut self.my_left
        } else {
            &mut self.my_right
        };
        let mut count = 0;
        for algo in algos.iter_mut() {
            algo.perform(distance, alt)?;
            if !algo.is_done() || algo.shape().is_null() {
                continue;
            }
            let shape = oriented_result(algo.shape(), is_was_reversed);
            BRepBuilder::default().add(result, &shape)?;
            if count == 0 {
                self.base.my_shape = shape;
            }
            count += 1;
        }
        Ok(count)
    }

    /// The `myIsToApprox` step of `Perform`: the wires (or the face) converted into
    /// arcs and segments with new 3D curves.
    fn approximate_contours(&mut self) -> Result<(), Failure> {
        if self.my_face.is_null() {
            return self.approximate_wires();
        }
        self.my_face = Self::convert_face(&self.my_face, APPROX_ANGLE_TOLERANCE)?;
        build_curves3d_default(self.my_face.as_ref())?;
        self.my_wires.clear();
        let mut children = ShapeIterator::new(self.my_face.as_ref(), true, true);
        while children.more() {
            self.my_wires.push(children.value()?.clone());
            children.next();
        }
        Ok(())
    }

    /// Converts each wire needing it on the plane face of the first planar wire.
    fn approximate_wires(&mut self) -> Result<(), Failure> {
        let face = first_planar_face(&self.my_wires)?;
        for shape in self.my_wires.iter_mut() {
            let wire = topods::wire(shape)?;
            if !needs_conversion(&wire)? {
                continue;
            }
            let mut new_wire = convert_wire(&wire, APPROX_ANGLE_TOLERANCE, &face)?;
            build_curves3d_default(new_wire.as_ref())?;
            new_wire.set_orientation(wire.orientation());
            *shape = new_wire.into_shape();
        }
        Ok(())
    }

    /// `ConvertFace(theFace, theAngleTolerance)`: the face whose wires (when one has an
    /// edge other than a line or a circle) are converted into 2D arcs and segments
    /// with new 3D curves;
    /// `BRepOffsetAPIMakeOffset::convert_face(&face, 0.1)?`.
    pub fn convert_face(face: &Face, angle_tolerance: f64) -> Result<Face, Failure> {
        let orientation = face.orientation();
        let forward = topods::face(&face.oriented(Orientation::Forward))?;
        let mut new_face = topods::face(&forward.empty_copied()?)?;
        let builder = BRepBuilder::default();
        let mut children = ShapeIterator::new(forward.as_ref(), true, true);
        while children.more() {
            let wire =
                converted_wire(&topods::wire(children.value()?)?, angle_tolerance, &forward)?;
            builder.add(&mut new_face, wire.as_ref())?;
            children.next();
        }
        new_face.set_orientation(orientation);
        Ok(new_face)
    }
}

/// A wire of `ConvertFace`: unchanged unless it needs a conversion.
fn converted_wire(wire: &Wire, angle_tolerance: f64, face: &Face) -> Result<Wire, Failure> {
    if !needs_conversion(wire)? {
        return Ok(wire.clone());
    }
    let orientation = wire.orientation();
    let forward = topods::wire(&wire.oriented(Orientation::Forward))?;
    let mut converted = convert_wire(&forward, angle_tolerance, face)?;
    build_curves3d_default(converted.as_ref())?;
    converted.set_orientation(orientation);
    Ok(converted)
}

/// `NeedsConvertion(theWire)` (static): whether an edge of the wire is neither a line
/// nor a circle.
fn needs_conversion(wire: &Wire) -> Result<bool, Failure> {
    let mut children = ShapeIterator::new(wire.as_ref(), true, true);
    while children.more() {
        let edge = topods::edge(children.value()?)?;
        let curve_type = BRepAdaptorCurve::with_edge(&edge)?.get_type()?;
        if curve_type != CurveType::Line && curve_type != CurveType::Circle {
            return Ok(true);
        }
        children.next();
    }
    Ok(false)
}

/// The plane face of the first wire that makes one (a null face when none does).
fn first_planar_face(wires: &[Shape]) -> Result<Face, Failure> {
    for shape in wires {
        let maker = BRepBuilderAPIMakeFace::from_wire(&topods::wire(shape)?, true)?;
        if maker.error() == BRepBuilderAPIFaceError::FaceDone {
            return maker.face();
        }
    }
    Ok(Face::default())
}

/// The shape of an offset, reversed when the domain wires were.
fn oriented_result(shape: &Shape, is_was_reversed: bool) -> Shape {
    if is_was_reversed {
        shape.reversed()
    } else {
        shape.clone()
    }
}
