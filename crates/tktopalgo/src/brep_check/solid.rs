// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Solid.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Solid`: the checker of a solid (shells that overlap, detached parts,
//! shells outside each other or enclosing each other), with the file-local helper
//! `BRepCheck_ToolSolid` (its classifier holder `BRepCheck_HSC` is the boxed
//! classifier).
use super::errors::null_object;
use super::explore::sub_shapes;
use super::result::{BRepCheckResult, ResultBase, SharedStatusList, add_status};
use super::status::BRepCheckStatus;
use crate::brep_class3d::BRepClass3dSolidClassifier;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Shape, ShapeIterator, Solid};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_small;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point3;

/// `aPAR_T`: the fraction of the edge range where the inner point is taken
/// (10 e^-PI, natively).
const PAR_T: f64 = 0.43213918;

/// `BRepCheck_ToolSolid`: one closed shell of the solid as a solid, its classifier,
/// whether it is a hole, and a point on its first non-degenerated edge.
struct ToolSolid {
    solid: Solid,
    is_hole: bool,
    point: Point3,
    point_tol: f64,
    // `myHSC`: null until `Init`.
    classifier: Option<Box<BRepClass3dSolidClassifier>>,
}

impl ToolSolid {
    fn new(solid: Solid) -> Self {
        Self {
            solid,
            is_hole: false,
            point: Point3::new(-1.0, -1.0, -1.0),
            point_tol: precision::confusion(),
            classifier: None,
        }
    }

    /// `Init()`: the classifier on the solid, the hole flag (the infinite point is IN)
    /// and the inner point.
    fn init(&mut self) -> Result<(), Failure> {
        let mut classifier = Box::new(BRepClass3dSolidClassifier::new());
        classifier.load(&self.solid)?;
        classifier.perform_infinite_point(real_small())?;
        self.is_hole = classifier.state() == State::In;
        self.classifier = Some(classifier);
        for shape in sub_shapes(&self.solid, ShapeEnum::Edge)? {
            let edge = Edge::unchecked(shape);
            if BRepTool::degenerated(&edge)? {
                continue;
            }
            let (curve, t1, t2) = BRepTool::curve(&edge)?;
            let t: f64 = (1.0 - PAR_T) * t1 + PAR_T * t2;
            let c = curve
                .get()
                .ok_or_else(|| null_object("BRepCheck_ToolSolid::Init: edge without 3d curve"))?;
            self.point = c.value(t)?;
            self.point_tol = BRepTool::edge_tolerance(&edge)?;
            break;
        }
        Ok(())
    }

    /// `IsOut(aOther)`: the inner point of `other` is OUT of this solid; a tool that
    /// was not initialized has no classifier (a null dereference natively).
    fn is_out(&mut self, other: (Point3, f64)) -> Result<bool, Failure> {
        let classifier = self
            .classifier
            .as_mut()
            .ok_or_else(|| null_object("BRepCheck_ToolSolid::IsOut: not initialized"))?;
        classifier.perform(&other.0, other.1)?;
        Ok(classifier.state() == State::Out)
    }
}

/// `BRepCheck_Solid`: checks a solid per se. The scan: shells that overlap
/// (`InvalidImbricationOfShells`), detached parts that are not INTERNAL and solids of
/// INTERNAL shells only (`BadOrientationOfSubshape`), several closed shells that are not
/// holes (`EnclosedRegion`) and closed shells out of one another
/// (`SubshapeNotInShape`).
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Solid;
/// use rustycad_tktopalgo::brep_check::{BRepCheckResult, BRepCheckSolid, BRepCheckStatus};
/// let mut s = Solid::default();
/// BRepBuilder::default().make_solid(&mut s);
/// let checker = BRepCheckSolid::new(&s).unwrap();
/// assert_eq!(checker.status().unwrap().first(), Some(&BRepCheckStatus::NoError));
/// ```
pub struct BRepCheckSolid {
    transient: StandardTransient,
    base: ResultBase,
}
impl_transient!(BRepCheckSolid, "BRepCheck_Solid", dyn BRepCheckResult, transient,
    interfaces: [dyn BRepCheckResult]);

impl BRepCheckSolid {
    /// `BRepCheck_Solid(theS)`: `Init(theS)`, so the solid is checked;
    /// `BRepCheckSolid::new(&solid)?`.
    pub fn new(solid: &Solid) -> Result<Self, Failure> {
        let checker = Self {
            transient: StandardTransient::default(),
            base: ResultBase::new(),
        };
        checker.init(solid)?;
        Ok(checker)
    }

    /// Step 1: a face met twice makes the shells overlap.
    fn check_imbrication(&self, list: &SharedStatusList) -> Result<(), Failure> {
        let mut faces = MapOfShape::default();
        for face in sub_shapes(&self.base.shape(), ShapeEnum::Face)? {
            if !faces.add(face) {
                add_status(list, BRepCheckStatus::InvalidImbricationOfShells);
                break;
            }
        }
        Ok(())
    }

    /// Step 2: the children; returns the closed shells that are not INTERNAL as solids.
    fn closed_shells(&self, list: &SharedStatusList) -> Result<Vec<ToolSolid>, Failure> {
        let (mut shells, mut internal_shells) = (0, 0);
        let mut tools = Vec::new();
        let mut iterator = ShapeIterator::new(&self.base.shape(), true, true);
        while let Ok(child) = iterator.value() {
            let child = child.clone();
            iterator.next();
            if child.shape_type()? != ShapeEnum::Shell {
                if child.orientation() != Orientation::Internal {
                    add_status(list, BRepCheckStatus::BadOrientationOfSubshape);
                }
                continue;
            }
            // Skip the internal shells.
            if has_internal_face(&child) {
                internal_shells += 1;
                continue;
            }
            shells += 1;
            // Skip the shells that are not closed.
            if !BRepTool::is_closed(&child)? {
                continue;
            }
            tools.push(ToolSolid::new(solid_of(&child)?));
        }
        if shells == 0 && internal_shells != 0 {
            // All shells in the solid are internal.
            add_status(list, BRepCheckStatus::BadOrientationOfSubshape);
        }
        Ok(tools)
    }
}

/// Whether the shell has a face INTERNAL among its children.
fn has_internal_face(shell: &Shape) -> bool {
    let mut iterator = ShapeIterator::new(shell, true, true);
    while let Ok(face) = iterator.value() {
        if face.orientation() == Orientation::Internal {
            return true;
        }
        iterator.next();
    }
    false
}

/// A new solid holding `shell`.
fn solid_of(shell: &Shape) -> Result<Solid, Failure> {
    let builder = BRepBuilder::default();
    let mut solid = Solid::default();
    builder.make_solid(&mut solid);
    builder.add(&mut solid, shell)?;
    Ok(solid)
}

/// Steps 3.2 and 3.1: more than one closed shell that is not a hole encloses a region
/// (the later tools are then not initialized), and a shell whose point is out of another
/// one is outside the solid.
fn check_tools(tools: &mut [ToolSolid], list: &SharedStatusList) -> Result<(), Failure> {
    let mut growths = 0;
    for tool in tools.iter_mut() {
        tool.init()?;
        if !tool.is_hole {
            growths += 1;
            if growths > 1 {
                // Too many growths.
                add_status(list, BRepCheckStatus::EnclosedRegion);
                break;
            }
        }
    }
    let count = tools.len();
    for i in 0..count.saturating_sub(1) {
        for j in i + 1..count {
            let other = (tools[j].point, tools[j].point_tol);
            if tools[i].is_out(other)? {
                // Something of the solid is out of the solid.
                add_status(list, BRepCheckStatus::SubshapeNotInShape);
                return Ok(());
            }
        }
    }
    Ok(())
}

impl BRepCheckResult for BRepCheckSolid {
    fn result_base(&self) -> &ResultBase {
        &self.base
    }

    /// `InContext(theContextShape)`: nothing to check.
    fn in_context(&self, _context: &Shape) -> Result<(), Failure> {
        Ok(())
    }

    /// `Minimum()`: the scan of [`BRepCheckSolid`].
    fn minimum(&self) -> Result<(), Failure> {
        if self.base.is_minimum() {
            return Ok(());
        }
        self.base.set_minimum(true);
        let list = self.base.bind_own_list();
        list.write().append(BRepCheckStatus::NoError);
        self.check_imbrication(&list)?;
        let mut tools = self.closed_shells(&list)?;
        if tools.len() < 2 {
            return Ok(());
        }
        check_tools(&mut tools, &list)
    }

    /// `Blind()`: nothing more than the minimum.
    fn blind(&self) {
        self.base.set_blind(true);
    }
}
