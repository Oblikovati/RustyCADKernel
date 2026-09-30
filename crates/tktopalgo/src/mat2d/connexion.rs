// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Connexion.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT2d_Connexion`: the shortest link between two lines of a figure, from a point
//! of an item of the first line to a point of an item of the second.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::storage::format_general;
use rustycad_tkmath::gp::{Point2, Vector2};

#[derive(Clone, Copy, Default)]
struct ConnexionState {
    line_a: i32,
    line_b: i32,
    item_a: i32,
    item_b: i32,
    distance: f64,
    parameter_on_a: f64,
    parameter_on_b: f64,
    point_a: Point2,
    point_b: Point2,
}

/// `MAT2d_Connexion`: a transient link between two lines (their indices, the items,
/// the parameters and the points at both ends and the distance).
/// ```
/// use rustycad_tktopalgo::mat2d::Mat2dConnexion;
/// use rustycad_tkmath::gp::Point2;
/// let c = Mat2dConnexion::with_values(1, 2, 3, 4, 1.0, 0.5, 0.25, Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
/// let r = c.reverse();
/// assert_eq!((r.index_first_line(), r.index_item_on_first()), (2, 4));
/// ```
pub struct Mat2dConnexion {
    parent: StandardTransient,
    state: SharedState<ConnexionState>,
}
impl_transient!(Mat2dConnexion, "MAT2d_Connexion", StandardTransient, parent);

impl Default for Mat2dConnexion {
    fn default() -> Self {
        Self::new()
    }
}

// One getter and one setter per native overload pair `Name()`/`Name(value)`.
macro_rules! connexion_fields {
    ($($(#[$doc:meta])* $get:ident, $set:ident, $field:ident: $ty:ty;)*) => {
        $(
            $(#[$doc])*
            pub fn $get(&self) -> $ty {
                self.state.read().$field
            }
            #[doc = concat!("The setter of [`Mat2dConnexion::", stringify!($get), "`]; `c.", stringify!($set), "(value)`.")]
            pub fn $set(&self, value: $ty) {
                self.state.write().$field = value;
            }
        )*
    };
}

impl Mat2dConnexion {
    /// `MAT2d_Connexion()`: every index and value 0; `Mat2dConnexion::new()`.
    pub fn new() -> Self {
        Self::from_state(ConnexionState::default())
    }
    fn from_state(state: ConnexionState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `MAT2d_Connexion(LineA, LineB, ItemA, ItemB, Distance, ParameterOnA,
    /// ParameterOnB, PointA, PointB)`; `Mat2dConnexion::with_values(1, 2, 1, 1, d, u, v, p, q)`.
    #[allow(clippy::too_many_arguments)] // The native constructor.
    pub fn with_values(
        line_a: i32,
        line_b: i32,
        item_a: i32,
        item_b: i32,
        distance: f64,
        parameter_on_a: f64,
        parameter_on_b: f64,
        point_a: Point2,
        point_b: Point2,
    ) -> Self {
        Self::from_state(ConnexionState {
            line_a,
            line_b,
            item_a,
            item_b,
            distance,
            parameter_on_a,
            parameter_on_b,
            point_a,
            point_b,
        })
    }
    connexion_fields! {
        /// `IndexFirstLine()`; `c.index_first_line()`.
        index_first_line, set_index_first_line, line_a: i32;
        /// `IndexSecondLine()`; `c.index_second_line()`.
        index_second_line, set_index_second_line, line_b: i32;
        /// `IndexItemOnFirst()`; `c.index_item_on_first()`.
        index_item_on_first, set_index_item_on_first, item_a: i32;
        /// `IndexItemOnSecond()`; `c.index_item_on_second()`.
        index_item_on_second, set_index_item_on_second, item_b: i32;
        /// `ParameterOnFirst()`; `c.parameter_on_first()`.
        parameter_on_first, set_parameter_on_first, parameter_on_a: f64;
        /// `ParameterOnSecond()`; `c.parameter_on_second()`.
        parameter_on_second, set_parameter_on_second, parameter_on_b: f64;
        /// `PointOnFirst()`; `c.point_on_first()`.
        point_on_first, set_point_on_first, point_a: Point2;
        /// `PointOnSecond()`; `c.point_on_second()`.
        point_on_second, set_point_on_second, point_b: Point2;
        /// `Distance()`; `c.distance()`.
        distance, set_distance, distance: f64;
    }
    /// `Reverse()`: a new connexion from the second line to the first;
    /// `c.reverse()`.
    pub fn reverse(&self) -> Handle<Mat2dConnexion> {
        let s: ConnexionState = *self.state.read();
        Handle::new(Self::with_values(
            s.line_b,
            s.line_a,
            s.item_b,
            s.item_a,
            s.distance,
            s.parameter_on_b,
            s.parameter_on_a,
            s.point_b,
            s.point_a,
        ))
    }
    /// `IsAfter(aConnexion, aSense)`: true when this connexion starts on the same line
    /// as `c2`, after it (a later item, a larger parameter, or at the same point on the
    /// side `sense` of it); `c.is_after(&other, 1.0)`.
    pub fn is_after(&self, c2: &Mat2dConnexion, sense: f64) -> bool {
        let s: ConnexionState = *self.state.read();
        let o: ConnexionState = *c2.state.read();
        if s.line_a != o.line_a {
            return false;
        }
        if s.item_a > o.item_a {
            return true;
        }
        if s.item_a != o.item_a {
            return false;
        }
        if s.parameter_on_a > o.parameter_on_a {
            return true;
        }
        if s.parameter_on_a == o.parameter_on_a {
            let vect1 = Vector2::between(o.point_a, o.point_b);
            let vect2 = Vector2::between(s.point_a, s.point_b);
            return vect1.crossed(vect2) * sense > 0.;
        }
        false
    }
    /// `Dump(Deep, Offset)`: the fields as `std::cout` gets them; `c.dump(0, 1, &mut text)`.
    pub fn dump(&self, _deep: i32, offset: i32, out: &mut String) {
        let s: ConnexionState = *self.state.read();
        let g = |value: f64| format_general(value, 6);
        let indent = |n: i32| " ".repeat(usize::try_from(n).unwrap_or(0));
        let (o, m) = (indent(offset), indent(offset + 1));
        out.push_str(&format!("{o}MAT2d_Connexion :\n"));
        out.push_str(&format!(
            "{m}IndexFirstLine    :{}\n{m}IndexSecondLine   :{}\n",
            s.line_a, s.line_b
        ));
        out.push_str(&format!(
            "{m}IndexItemOnFirst  :{}\n{m}IndexItemOnSecond :{}\n",
            s.item_a, s.item_b
        ));
        out.push_str(&format!("{m}ParameterOnFirst  :{}\n", g(s.parameter_on_a)));
        out.push_str(&format!("{m}ParameterOnSecond :{}\n", g(s.parameter_on_b)));
        out.push_str(&format!(
            "{m}PointOnFirst      :\n  X = {}\n  Y = {}\n",
            g(s.point_a.x()),
            g(s.point_a.y())
        ));
        out.push_str(&format!(
            "{m}PointOnSecond     :\n  X = {}\n  Y = {}\n",
            g(s.point_b.x()),
            g(s.point_b.y())
        ));
        out.push_str(&format!("{m}Distance          :{}\n", g(s.distance)));
    }
}
