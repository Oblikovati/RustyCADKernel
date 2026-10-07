//! The choice of the fillet solution shared by the constant and evolutive radius
//! functions and their inverses: `Set(Radius, Choix)` and `Set(Choix)` sign the offsets of
//! both surfaces the same way.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::law::LawFunction;

/// The signed radii `(ray1, ray2)` of a choice, as `BlendFunc_ConstRad::Set(Radius, Choix)`
/// assigns them; `signed_radii(2.0, 4) == (2.0, -2.0)`.
pub(crate) fn signed_radii(radius: f64, choix: i32) -> (f64, f64) {
    match choix {
        1 | 2 => (-radius, -radius),
        3 | 4 => (radius, -radius),
        5 | 6 => (radius, radius),
        7 | 8 => (-radius, radius),
        _ => (-radius, -radius),
    }
}

/// The offset signs `(sg1, sg2)` of a choice, as `BlendFunc_EvolRad::Set(Choix)` assigns
/// them; `choice_signs(4) == (1.0, -1.0)`.
pub(crate) fn choice_signs(choix: i32) -> (f64, f64) {
    match choix {
        1 | 2 => (-1., -1.),
        3 | 4 => (1., -1.),
        5 | 6 => (1., 1.),
        7 | 8 => (-1., 1.),
        _ => (-1., -1.),
    }
}

/// The radius law of an evolutive function, or `Standard_NullObject` naming `class`
/// (natively a null dereference); `live_law(&law, "BlendFunc_EvolRad")?`.
pub(crate) fn live_law<'a>(
    law: &'a Handle<dyn LawFunction>,
    class: &str,
) -> Result<&'a dyn LawFunction, Failure> {
    law.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("{class}: null radius law; expected a Law_Function"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_radii_follow_the_choice() {
        assert_eq!(signed_radii(2., 1), (-2., -2.));
        assert_eq!(signed_radii(2., 4), (2., -2.));
        assert_eq!(signed_radii(2., 6), (2., 2.));
        assert_eq!(signed_radii(2., 8), (-2., 2.));
        assert_eq!(signed_radii(2., 0), (-2., -2.));
    }

    #[test]
    fn signs_follow_the_choice() {
        assert_eq!(choice_signs(2), (-1., -1.));
        assert_eq!(choice_signs(3), (1., -1.));
        assert_eq!(choice_signs(5), (1., 1.));
        assert_eq!(choice_signs(7), (-1., 1.));
        assert_eq!(choice_signs(9), (-1., -1.));
    }

    #[test]
    fn a_null_law_is_a_null_object() {
        let law: Handle<dyn LawFunction> = Handle::default();
        let error = live_law(&law, "BlendFunc_EvolRadInv").err().unwrap();
        assert_eq!(error.exception_type(), "Standard_NullObject");
    }
}
