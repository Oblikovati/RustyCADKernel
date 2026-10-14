// SPDX-License-Identifier: GPL-2.0-only
//! Consistent native span observations are not canonical algebraic partition coverage.
use crate::sketch_region::SourcePartitionObservation;
use rustycad_tkmath::predicates::roots::DyadicBound;

#[derive(Debug, PartialEq)]
pub(crate) enum NativeSpanAudit {
    Unreferenced,
    Contiguous { blocks: usize },
}
#[derive(Debug, PartialEq)]
pub(crate) enum NativeSpanCause {
    Nonfinite {
        role: &'static str,
        value: f64,
    },
    Reversed {
        range: [f64; 2],
    },
    MissingBlocks,
    UnexpectedBlocks {
        count: usize,
    },
    Boundary {
        position: usize,
        expected: DyadicBound,
        observed: DyadicBound,
    },
}
pub(super) fn inspect_native_spans(
    source: &SourcePartitionObservation,
) -> Result<NativeSpanAudit, NativeSpanCause> {
    let range = admitted_range(source.range)?;
    if !source.has_reference {
        return unreferenced_spans(source);
    }
    if source.blocks.is_empty() {
        return Err(NativeSpanCause::MissingBlocks);
    }
    let mut boundary = range[0].clone();
    for (position, block) in source.blocks.iter().enumerate() {
        boundary = append_native_span(position, &boundary, block.own_range)?;
    }
    compare_boundary(source.blocks.len(), &range[1], &boundary)?;
    Ok(NativeSpanAudit::Contiguous {
        blocks: source.blocks.len(),
    })
}
fn admitted_range(range: [f64; 2]) -> Result<[DyadicBound; 2], NativeSpanCause> {
    let first = native_parameter("range start", range[0])?;
    let last = native_parameter("range end", range[1])?;
    if first > last {
        return Err(NativeSpanCause::Reversed { range });
    }
    Ok([first, last])
}
fn native_parameter(role: &'static str, value: f64) -> Result<DyadicBound, NativeSpanCause> {
    DyadicBound::from_f64(value).map_err(|_| NativeSpanCause::Nonfinite { role, value })
}
fn unreferenced_spans(
    source: &SourcePartitionObservation,
) -> Result<NativeSpanAudit, NativeSpanCause> {
    if !source.blocks.is_empty() {
        return Err(NativeSpanCause::UnexpectedBlocks {
            count: source.blocks.len(),
        });
    }
    Ok(NativeSpanAudit::Unreferenced)
}
fn append_native_span(
    position: usize,
    boundary: &DyadicBound,
    range: [f64; 2],
) -> Result<DyadicBound, NativeSpanCause> {
    let span = admitted_range(range)?;
    compare_boundary(position, boundary, &span[0])?;
    Ok(span[1].clone())
}
fn compare_boundary(
    position: usize,
    expected: &DyadicBound,
    observed: &DyadicBound,
) -> Result<(), NativeSpanCause> {
    if expected != observed {
        return Err(NativeSpanCause::Boundary {
            position,
            expected: expected.clone(),
            observed: observed.clone(),
        });
    }
    Ok(())
}

impl std::fmt::Display for NativeSpanCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Keep offending native observations distinct from the expected structural contract.
        formatter.write_str(&self.message())
    }
}
impl NativeSpanCause {
    fn message(&self) -> String {
        match self {
            Self::Nonfinite { role, value } => {
                format!("{role}={value}; expected finite native parameter")
            }
            Self::Reversed { range } => {
                format!("range={range:?}; expected ordered closed native range")
            }
            Self::MissingBlocks => {
                "has_reference=true, blocks=0; expected observed referenced pave blocks".to_owned()
            }
            Self::UnexpectedBlocks { count } => unexpected_span_message(*count),
            Self::Boundary {
                position,
                expected,
                observed,
            } => native_boundary_message(*position, expected, observed),
        }
    }
}
fn unexpected_span_message(count: usize) -> String {
    format!("has_reference=false, blocks={count}; expected unreferenced empty pave list")
}
fn native_boundary_message(
    position: usize,
    expected: &DyadicBound,
    observed: &DyadicBound,
) -> String {
    format!("boundary[{position}]={observed}; expected exactly {expected}")
}
