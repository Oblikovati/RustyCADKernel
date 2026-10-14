// SPDX-License-Identifier: GPL-2.0-only
//! Admit the complete caller-name roster before legacy allocation or pairwise collision scans.
use super::*;

pub(super) fn inspect(
    bindings: &PrismPublicationBindings,
    requested: AnnularFrozenOptions,
) -> AnnularFrozenCheck<'static, AnnularNameCensus> {
    if requested.version != 2 {
        return Err(AnnularFrozenCause::Version {
            actual: requested.version,
            expected: 2,
        });
    }
    let measured = measure_names(bindings, requested.maximum_name_units)?;
    Ok(AnnularNameCensus {
        requested,
        effective: requested,
        measured_units: measured,
    })
}
fn measure_names(
    bindings: &PrismPublicationBindings,
    maximum: usize,
) -> AnnularFrozenCheck<'static, usize> {
    let mut measured = 0;
    charge(&mut measured, 2, maximum)?;
    for count in [
        bindings.originals.len(),
        bindings.sources.len(),
        bindings.directed.len(),
    ] {
        charge(&mut measured, count, maximum)?;
    }
    for row in &bindings.directed {
        charge(&mut measured, row.actual.ancestors.len(), maximum)?;
        charge(&mut measured, row.actual.child_slots.len(), maximum)?;
    }
    Ok(measured)
}

fn charge(
    measured: &mut usize,
    additional: usize,
    maximum: usize,
) -> AnnularFrozenCheck<'static, ()> {
    let next = measured.checked_add(additional);
    if next.is_none_or(|value| value > maximum) {
        return Err(AnnularFrozenCause::NameBudget {
            maximum,
            measured: next,
        });
    }
    *measured = next.unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inclusive_structural_limit_and_overflow_never_widen_policy() {
        let mut measured = 7;
        assert!(charge(&mut measured, 3, 10).is_ok());
        assert_eq!(measured, 10);
        assert!(matches!(
            charge(&mut measured, 1, 10),
            Err(AnnularFrozenCause::NameBudget {
                maximum: 10,
                measured: Some(11)
            })
        ));
        measured = usize::MAX;
        assert!(matches!(
            charge(&mut measured, 1, usize::MAX),
            Err(AnnularFrozenCause::NameBudget { measured: None, .. })
        ));
    }
}
