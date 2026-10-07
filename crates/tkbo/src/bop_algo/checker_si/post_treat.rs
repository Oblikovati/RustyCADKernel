//! `BOPAlgo_CheckerSI::PostTreat`: the interfering pairs of every kind, in native order.
use super::BOPAlgoCheckerSI;
use crate::bop_ds::{DataStructure, InterfFF, Pair};
use crate::bop_tools::are_faces_same_domain;
use crate::int_tools::Context;
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BOPAlgoCheckerSI {
    /// Adds to the interference table the pairs of source shapes of every kind (0 V/V .. 9
    /// S/S): a vertex, edge or face pair with a new shape is skipped, an E/F interference only
    /// counts with a common part, and an F/F one only for one face, same-domain tangent faces
    /// or a section curve with pave blocks (`PostTreat`).
    pub(super) fn post_treat(&mut self) -> Result<(), Failure> {
        let context = self.filler.context_handle();
        let context = context.get().ok_or_else(super::missing_structure)?;
        let fuzzy = self.filler.options().fuzzy_value();
        let structure = self.structure_mut()?;
        let mut pairs = source_pairs(structure);
        pairs.extend(face_face_pairs(structure, context, fuzzy)?);
        let [vertex_solid, edge_solid, face_solid, solid_solid] = solid_pairs(structure);
        pairs.extend(without_new_shapes(structure, vertex_solid));
        pairs.extend(edge_solid.into_iter().chain(face_solid).chain(solid_solid));
        let table = structure.interference_table_mut();
        for (first, second) in pairs {
            table.add(Pair::new(first, second));
        }
        Ok(())
    }
}

/// The V/V, V/E, E/E, V/F and E/F pairs between source shapes (kinds 0 to 4).
fn source_pairs(structure: &DataStructure) -> Vec<(i32, i32)> {
    let mut pairs: Vec<(i32, i32)> = structure
        .interf_vv_records()
        .iter()
        .map(|r| r.indices())
        .collect();
    pairs.extend(structure.interf_ve_records().iter().map(|r| r.indices()));
    pairs.extend(structure.interf_ee_records().iter().map(|r| r.indices()));
    pairs.extend(structure.interf_vf_records().iter().map(|r| r.indices()));
    let mut pairs = without_new_shapes(structure, pairs);
    let edge_face = structure
        .interf_ef_records()
        .iter()
        .filter(|record| record.common_part().shape_type() != ShapeEnum::Shape)
        .map(|record| record.indices())
        .collect();
    pairs.extend(without_new_shapes(structure, edge_face));
    pairs
}

/// The pairs whose two shapes are source shapes.
fn without_new_shapes(structure: &DataStructure, pairs: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    pairs
        .into_iter()
        .filter(|&(first, second)| {
            !structure.is_new_shape(first) && !structure.is_new_shape(second)
        })
        .collect()
}

/// The F/F pairs (kind 5) that make a real interference.
fn face_face_pairs(
    structure: &DataStructure,
    context: &Context,
    fuzzy: f64,
) -> Result<Vec<(i32, i32)>, Failure> {
    let mut pairs = Vec::new();
    for record in structure.interf_ff_records().iter() {
        if record.points().is_empty() && record.curves().is_empty() && !record.tangent_faces() {
            continue;
        }
        if is_face_face_interference(structure, record, context, fuzzy)? {
            pairs.push(record.indices());
        }
    }
    Ok(pairs)
}

/// A face with itself, tangent faces of the same domain, or a section curve with a block.
fn is_face_face_interference(
    structure: &DataStructure,
    record: &InterfFF,
    context: &Context,
    fuzzy: f64,
) -> Result<bool, Failure> {
    let (first, second) = record.indices();
    if first == second {
        // Self-intersection inside one face.
        return Ok(true);
    }
    if record.tangent_faces() {
        let first = topods::face(structure.shape(first)?)?;
        let second = topods::face(structure.shape(second)?)?;
        return are_faces_same_domain(&first, &second, context, fuzzy);
    }
    Ok(record
        .curves()
        .iter()
        .any(|curve| !curve.pave_blocks().is_empty()))
}

/// The V/S, E/S, F/S and S/S pairs (kinds 6 to 9), one list per kind.
fn solid_pairs(structure: &mut DataStructure) -> [Vec<(i32, i32)>; 4] {
    [
        structure.interf_vz().iter().map(|r| r.indices()).collect(),
        structure.interf_ez().iter().map(|r| r.indices()).collect(),
        structure.interf_fz().iter().map(|r| r.indices()).collect(),
        structure.interf_zz().iter().map(|r| r.indices()).collect(),
    ]
}
