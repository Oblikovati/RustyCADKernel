// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_Bisector.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT_Bisector`: a bisector of the tree the MAT algorithm builds, with the two
//! edges it separates and the bisectors it comes from.
use super::{MatEdge, MatListOfBisector, MatTListItem};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::errors;

// The fields that the native constructor leaves unset are 0 from the cleared
// allocation.
struct BisectorState {
    bisector_number: i32,
    index_number: i32,
    first_edge: Handle<MatEdge>,
    second_edge: Handle<MatEdge>,
    list_of_bisectors: Handle<MatListOfBisector>,
    issue_point: i32,
    end_point: i32,
    first_vector: i32,
    second_vector: i32,
    sense: f64,
    first_parameter: f64,
    second_parameter: f64,
    dist_issue_point: f64,
}

/// `MAT_Bisector`: a transient bisector of the MAT algorithm (the geometric bisector
/// is in the tool, by `BisectorNumber`). Its list holds the bisectors that end at its
/// issue point.
/// ```
/// use rustycad_tktopalgo::mat::MatBisector;
/// let bisector = MatBisector::new();
/// assert_eq!(bisector.bisector_number(), -1);
/// assert!(bisector.list().is_empty());
/// ```
pub struct MatBisector {
    parent: StandardTransient,
    state: SharedState<BisectorState>,
}
impl_transient!(MatBisector, "MAT_Bisector", StandardTransient, parent);

impl Default for MatBisector {
    fn default() -> Self {
        Self::new()
    }
}

// One getter and one setter per native overload pair `Name()`/`Name(value)`.
macro_rules! bisector_fields {
    ($($(#[$doc:meta])* $get:ident, $set:ident: $ty:ty;)*) => {
        $(
            $(#[$doc])*
            pub fn $get(&self) -> $ty {
                self.state.read().$get.clone()
            }
            #[doc = concat!("The setter of [`MatBisector::", stringify!($get), "`]; `bisector.", stringify!($set), "(value)`.")]
            pub fn $set(&self, value: $ty) {
                self.state.write().$get = value;
            }
        )*
    };
}

impl MatBisector {
    /// `MAT_Bisector()`: number -1, infinite parameters and an empty list;
    /// `MatBisector::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(BisectorState {
                bisector_number: -1,
                index_number: 0,
                first_edge: Handle::default(),
                second_edge: Handle::default(),
                list_of_bisectors: Handle::new(MatListOfBisector::new()),
                issue_point: 0,
                end_point: 0,
                first_vector: 0,
                second_vector: 0,
                sense: 0.,
                first_parameter: precision::infinite(),
                second_parameter: precision::infinite(),
                dist_issue_point: 0.,
            }),
        }
    }
    /// `AddBisector(abisector)`: appends a bisector to the list; `b.add_bisector(&other)`.
    pub fn add_bisector(&self, abisector: &Handle<MatBisector>) {
        self.list().back_add(abisector);
    }
    /// `List()`: the list of the bisectors; `b.list()`.
    pub fn list(&self) -> Handle<MatListOfBisector> {
        self.state.read().list_of_bisectors.clone()
    }
    /// `FirstBisector()`: the first bisector of the list; Standard_NullObject when it
    /// is empty (a native null dereference); `b.first_bisector()?`.
    pub fn first_bisector(&self) -> Result<Handle<MatBisector>, Failure> {
        self.list().first_item()
    }
    /// `LastBisector()`: the last bisector of the list; `b.last_bisector()?`.
    pub fn last_bisector(&self) -> Result<Handle<MatBisector>, Failure> {
        self.list().last_item()
    }
    bisector_fields! {
        /// `BisectorNumber()`: the index of the geometric bisector; `b.bisector_number()`.
        bisector_number, set_bisector_number: i32;
        /// `IndexNumber()`: the index of the bisector in the algorithm; `b.index_number()`.
        index_number, set_index_number: i32;
        /// `FirstEdge()`; `b.first_edge()`.
        first_edge, set_first_edge: Handle<MatEdge>;
        /// `SecondEdge()`; `b.second_edge()`.
        second_edge, set_second_edge: Handle<MatEdge>;
        /// `IssuePoint()`: the index of the point it starts at; `b.issue_point()`.
        issue_point, set_issue_point: i32;
        /// `EndPoint()`: the index of the point it ends at; `b.end_point()`.
        end_point, set_end_point: i32;
        /// `DistIssuePoint()`: the distance of the issue point; `b.dist_issue_point()`.
        dist_issue_point, set_dist_issue_point: f64;
        /// `FirstVector()`: the index of the first vector; `b.first_vector()`.
        first_vector, set_first_vector: i32;
        /// `SecondVector()`: the index of the second vector; `b.second_vector()`.
        second_vector, set_second_vector: i32;
        /// `Sense()`; `b.sense()`.
        sense, set_sense: f64;
        /// `FirstParameter()`: the parameter of the intersection with the previous
        /// bisector; `b.first_parameter()`.
        first_parameter, set_first_parameter: f64;
        /// `SecondParameter()`: the parameter of the intersection with the next
        /// bisector; `b.second_parameter()`.
        second_parameter, set_second_parameter: f64;
    }
    /// `Dump(ashift, alevel)`: the numbers of the bisector and of its edges, shifted by
    /// `ashift` double spaces, and at a positive level the list when its current node
    /// is past the end (the native test); a null edge is Standard_NullObject (a native
    /// crash); `b.dump(0, 1, &mut text)?`.
    pub fn dump(&self, ashift: i32, alevel: i32, out: &mut String) -> Result<(), Failure> {
        let shift: String = "  ".repeat(usize::try_from(ashift).unwrap_or(0));
        let (number, first, second) = {
            let state = self.state.read();
            (
                state.bisector_number,
                state.first_edge.clone(),
                state.second_edge.clone(),
            )
        };
        out.push_str(&format!("{shift} BISECTOR : {number}\n"));
        out.push_str(&format!(
            "{shift}   First edge     : {}\n",
            edge_number(&first)?
        ));
        out.push_str(&format!(
            "{shift}   Second edge    : {}\n",
            edge_number(&second)?
        ));
        out.push_str(&shift);
        let list: Handle<MatListOfBisector> = self.list();
        if alevel != 0 && !list.more() {
            out.push_str("   Bisectors List : \n");
            list.dump(ashift + 1, 1, out)?;
        }
        out.push('\n');
        Ok(())
    }
}

fn edge_number(edge: &Handle<MatEdge>) -> Result<i32, Failure> {
    edge.get().map(MatEdge::edge_number).ok_or_else(|| {
        errors::null_object(
            "MAT_Bisector::Dump: the edge is null; expected FirstEdge and SecondEdge",
        )
    })
}

impl MatTListItem for MatBisector {
    const LIST_NAME: &'static str = "MAT_ListOfBisector";
    const NODE_NAME: &'static str = "MAT_TListNodeOfListOfBisector";
    fn dump_item(&self, ashift: i32, alevel: i32, out: &mut String) -> Result<(), Failure> {
        self.dump(ashift, alevel, out)
    }
}
