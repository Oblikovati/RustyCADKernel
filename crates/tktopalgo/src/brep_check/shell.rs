// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Shell.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Shell`: the checker of a shell (connected faces; in a solid, closed and
//! consistently oriented). `Closed` and `NbConnectedSet` are in `shell_closed.rs`,
//! `Orientation` in `shell_orientation.rs`.
use super::explore::{holds_same, sub_shapes};
use super::result::{BRepCheckResult, ResultBase, SharedStatusList, add_status, close_status_list};
use super::status::BRepCheckStatus;
use rustycad_tkbrep::topods::{Shape, Shell};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The own fields of `BRepCheck_Shell`.
pub(super) struct ShellFields {
    /// `myNbori`: the number of oriented faces found by `Closed`.
    pub(super) nbori: i32,
    /// `myCdone`, `myCstat`: `Closed` has run, and its result.
    pub(super) cdone: bool,
    pub(super) cstat: BRepCheckStatus,
    /// `myOdone`, `myOstat`: `Orientation` has run, and its result.
    pub(super) odone: bool,
    pub(super) ostat: BRepCheckStatus,
    /// `myMapEF`: each edge and the faces through it.
    pub(super) map_ef: IndexedDataMapOfShapeListOfShape,
}

/// `BRepCheck_Shell`: checks that the faces of a shell are connected and, in the
/// context of a solid, that the shell is closed and consistently oriented.
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Shell;
/// use rustycad_tktopalgo::brep_check::{BRepCheckResult, BRepCheckShell, BRepCheckStatus};
/// let mut s = Shell::default();
/// BRepBuilder::default().make_shell(&mut s);
/// let checker = BRepCheckShell::new(&s).unwrap();
/// assert_eq!(checker.status().unwrap().first(), Some(&BRepCheckStatus::EmptyShell));
/// ```
pub struct BRepCheckShell {
    transient: StandardTransient,
    base: ResultBase,
    pub(super) fields: SharedState<ShellFields>,
}
impl_transient!(BRepCheckShell, "BRepCheck_Shell", dyn BRepCheckResult, transient,
    interfaces: [dyn BRepCheckResult]);

/// `Propagate(mapEF, theFace, theMapF)`: the faces connected to `face` through the
/// edges of `map_ef`, breadth first (new faces go to the end of the indexed map).
pub(super) fn propagate_faces(
    map_ef: &IndexedDataMapOfShapeListOfShape,
    face: &Shape,
    map_f: &mut IndexedMapOfShape,
) -> Result<(), Failure> {
    map_f.add(face.clone());
    let mut index: usize = 1;
    while index <= map_f.extent() {
        let current = map_f.find_key(index)?.clone();
        for edge in sub_shapes(&current, ShapeEnum::Edge)? {
            let Some(faces) = map_ef.seek(&edge) else {
                continue;
            };
            for other in faces {
                map_f.add(other.clone());
            }
        }
        index += 1;
    }
    Ok(())
}

impl BRepCheckShell {
    /// `BRepCheck_Shell(S)`: `Init(S)`; `BRepCheckShell::new(&shell)?`.
    pub fn new(shell: &Shell) -> Result<Self, Failure> {
        let checker = Self {
            transient: StandardTransient::default(),
            base: ResultBase::new(),
            fields: SharedState::new(ShellFields {
                nbori: 0,
                cdone: false,
                cstat: BRepCheckStatus::NoError,
                odone: false,
                ostat: BRepCheckStatus::NoError,
                map_ef: IndexedDataMapOfShapeListOfShape::default(),
            }),
        };
        checker.init(shell)?;
        Ok(checker)
    }

    /// `SetUnorientable()`: adds `UnorientableShape` to the statuses of the shell;
    /// `checker.set_unorientable()?`.
    pub fn set_unorientable(&self) -> Result<(), Failure> {
        add_status(&self.base.own_list()?, BRepCheckStatus::UnorientableShape);
        Ok(())
    }

    /// `IsUnorientable()`: the result of `Orientation` when it has run, else whether
    /// `UnorientableShape` was recorded; `checker.is_unorientable()?`.
    pub fn is_unorientable(&self) -> Result<bool, Failure> {
        {
            let fields = self.fields.read();
            if fields.odone {
                return Ok(fields.ostat != BRepCheckStatus::NoError);
            }
        }
        let list = self.base.own_list()?;
        let found = list
            .read()
            .iter()
            .any(|s| *s == BRepCheckStatus::UnorientableShape);
        Ok(found)
    }

    /// The shell being checked.
    pub(super) fn shell(&self) -> Shape {
        self.base.shape()
    }

    /// `myMap(myShape)`.
    pub(super) fn own_list(&self) -> Result<SharedStatusList, Failure> {
        self.base.own_list()
    }

    /// The solid context: the closure defect, else the orientation of an orientable
    /// shell.
    fn check_in_solid(&self, list: &SharedStatusList) -> Result<(), Failure> {
        let status = self.closed(false)?;
        // `(fst == NotClosed && S.Closed()) || fst != NoError` reduces to the second.
        if status != BRepCheckStatus::NoError {
            add_status(list, status);
        } else if !self.is_unorientable()? {
            add_status(list, self.orientation(false)?);
        }
        Ok(())
    }
}

impl BRepCheckResult for BRepCheckShell {
    fn result_base(&self) -> &ResultBase {
        &self.base
    }

    /// `InContext(S)`: `SubshapeNotInShape` when the shell is not in `context`; in a
    /// solid, the closure and then the orientation.
    fn in_context(&self, context: &Shape) -> Result<(), Failure> {
        let Some(list) = self.base.bind_context_list(context) else {
            return Ok(());
        };
        if !holds_same(context, ShapeEnum::Shell, &self.base.shape())? {
            add_status(&list, BRepCheckStatus::SubshapeNotInShape);
            return Ok(());
        }
        if context.shape_type()? == ShapeEnum::Solid {
            self.check_in_solid(&list)?;
        }
        close_status_list(&list);
        Ok(())
    }

    /// `Minimum()`: `EmptyShell` without faces, `NotConnected` when the face
    /// occurrences are not all reached through the edges from the first face; resets
    /// `Closed` and `Orientation`.
    fn minimum(&self) -> Result<(), Failure> {
        {
            let mut fields = self.fields.write();
            fields.cdone = false;
            fields.odone = false;
        }
        if self.base.is_minimum() {
            return Ok(());
        }
        let list = self.base.bind_own_list();
        let faces = sub_shapes(&self.base.shape(), ShapeEnum::Face)?;
        let mut map_ef = IndexedDataMapOfShapeListOfShape::default();
        for face in &faces {
            for edge in sub_shapes(face, ShapeEnum::Edge)? {
                super::wire::append_under_key(&mut map_ef, &edge, face)?;
            }
        }
        if faces.is_empty() {
            add_status(&list, BRepCheckStatus::EmptyShell);
        } else if faces.len() >= 2 {
            let mut map_f = IndexedMapOfShape::default();
            propagate_faces(&map_ef, &faces[0], &mut map_f)?;
            // The distinct faces reached are compared with the face occurrences.
            if map_f.extent() != faces.len() {
                add_status(&list, BRepCheckStatus::NotConnected);
            }
        }
        close_status_list(&list);
        self.fields.write().map_ef.clear(true);
        self.base.set_minimum(true);
        Ok(())
    }

    /// `Blind()`: nothing more than the minimum.
    fn blind(&self) {
        self.base.set_blind(true);
    }
}
