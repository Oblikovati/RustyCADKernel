// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_Evolution.hxx and
// TNaming_NameType.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two enumerations of TNaming: how a named shape was produced and how a name
//! re-finds a selected shape.

/// `TNaming_Evolution`: the kind of a `TNaming_NamedShape`, in native order.
///
/// ```
/// use rustycad_tkcaf::tnaming::TNamingEvolution;
/// assert_eq!(TNamingEvolution::Modify as i32, 2);
/// assert_eq!(TNamingEvolution::default(), TNamingEvolution::Primitive);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum TNamingEvolution {
    /// `TNaming_PRIMITIVE`: new shapes without an origin.
    #[default]
    Primitive,
    /// `TNaming_GENERATED`: new shapes generated from old ones.
    Generated,
    /// `TNaming_MODIFY`: old shapes modified into new ones.
    Modify,
    /// `TNaming_DELETE`: old shapes deleted.
    Delete,
    /// `TNaming_REPLACE`: kept for compatibility (loaded as `GENERATED`).
    Replace,
    /// `TNaming_SELECTED`: a selection of a shape in a context.
    Selected,
}

impl TNamingEvolution {
    /// The enumerator of a native integer value (`(TNaming_Evolution)i`), `None` out of range;
    /// `TNamingEvolution::from_index(3) == Some(TNamingEvolution::Delete)`.
    pub fn from_index(index: i32) -> Option<Self> {
        const ALL: [TNamingEvolution; 6] = [
            TNamingEvolution::Primitive,
            TNamingEvolution::Generated,
            TNamingEvolution::Modify,
            TNamingEvolution::Delete,
            TNamingEvolution::Replace,
            TNamingEvolution::Selected,
        ];
        usize::try_from(index)
            .ok()
            .and_then(|i| ALL.get(i).copied())
    }
}

/// `TNaming_NameType`: the algorithm of a `TNaming_Name`, in native order.
///
/// ```
/// use rustycad_tkcaf::tnaming::TNamingNameType;
/// assert_eq!(TNamingNameType::Orientation as i32, 9);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum TNamingNameType {
    /// `TNaming_UNKNOWN`.
    #[default]
    Unknown,
    /// `TNaming_IDENTITY`.
    Identity,
    /// `TNaming_MODIFUNTIL`.
    ModifUntil,
    /// `TNaming_GENERATION`.
    Generation,
    /// `TNaming_INTERSECTION`.
    Intersection,
    /// `TNaming_UNION`.
    Union,
    /// `TNaming_SUBSTRACTION`.
    Substraction,
    /// `TNaming_CONSTSHAPE`.
    ConstShape,
    /// `TNaming_FILTERBYNEIGHBOURGS`.
    FilterByNeighbourgs,
    /// `TNaming_ORIENTATION`.
    Orientation,
    /// `TNaming_WIREIN`.
    WireIn,
    /// `TNaming_SHELLIN`.
    ShellIn,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evolution_indexes_follow_the_native_order() {
        for index in 0..6 {
            let evolution = TNamingEvolution::from_index(index).unwrap();
            assert_eq!(evolution as i32, index);
        }
        assert_eq!(TNamingEvolution::from_index(6), None);
        assert_eq!(TNamingEvolution::from_index(-1), None);
    }

    #[test]
    fn name_types_follow_the_native_order() {
        assert_eq!(TNamingNameType::Unknown as i32, 0);
        assert_eq!(TNamingNameType::ShellIn as i32, 11);
    }
}
