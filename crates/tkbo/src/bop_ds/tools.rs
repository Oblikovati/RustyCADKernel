//! `BOPDS_Tools`: the shape-type codes of the data structure and the interference kinds.
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The rank of a shape type, from compound (0) to shape (8) (`TypeToInteger(T)`);
/// `type_to_integer(ShapeEnum::Vertex) == 7`.
pub fn type_to_integer(kind: ShapeEnum) -> i32 {
    match kind {
        ShapeEnum::Compound => 0,
        ShapeEnum::CompSolid => 1,
        ShapeEnum::Solid => 2,
        ShapeEnum::Shell => 3,
        ShapeEnum::Face => 4,
        ShapeEnum::Wire => 5,
        ShapeEnum::Edge => 6,
        ShapeEnum::Vertex => 7,
        ShapeEnum::Shape => 8,
    }
}

/// The interference kind of two shape types in either order: 0 VV, 1 VE, 2 EE, 3 VF, 4 EF,
/// 5 FF, 6 VZ, 7 EZ, 8 FZ, 9 ZZ, or -1 for a pair that never interferes
/// (`TypeToInteger(T1, T2)`); `type_pair_to_integer(ShapeEnum::Edge, ShapeEnum::Face) == 4`.
pub fn type_pair_to_integer(first: ShapeEnum, second: ShapeEnum) -> i32 {
    match type_to_integer(second) * 10 + type_to_integer(first) {
        77 => 0,
        76 | 67 => 1,
        66 => 2,
        74 | 47 => 3,
        64 | 46 => 4,
        44 => 5,
        72 | 27 => 6,
        62 | 26 => 7,
        42 | 24 => 8,
        22 => 9,
        _ => -1,
    }
}

/// Whether shapes of the type carry geometry (vertex, edge, face) (`HasBRep`);
/// `has_brep(ShapeEnum::Face)`.
pub fn has_brep(kind: ShapeEnum) -> bool {
    matches!(kind, ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face)
}

/// Whether shapes of the type take part in interferences: the geometric types and solids
/// (`IsInterfering`); `is_interfering(ShapeEnum::Solid)`.
pub fn is_interfering(kind: ShapeEnum) -> bool {
    has_brep(kind) || kind == ShapeEnum::Solid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_codes_are_symmetric_and_cover_the_ten_kinds() {
        let kinds = [
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            ShapeEnum::Solid,
        ];
        let mut codes = Vec::new();
        for (row, first) in kinds.iter().enumerate() {
            for second in &kinds[row..] {
                assert_eq!(
                    type_pair_to_integer(*first, *second),
                    type_pair_to_integer(*second, *first)
                );
                codes.push(type_pair_to_integer(*first, *second));
            }
        }
        codes.sort_unstable();
        assert_eq!(codes, (0..10).collect::<Vec<i32>>());
        assert_eq!(type_pair_to_integer(ShapeEnum::Wire, ShapeEnum::Edge), -1);
    }

    #[test]
    fn interfering_types_are_the_geometric_types_and_solids() {
        assert!(has_brep(ShapeEnum::Edge) && !has_brep(ShapeEnum::Solid));
        assert!(is_interfering(ShapeEnum::Solid) && !is_interfering(ShapeEnum::Shell));
        assert_eq!(type_to_integer(ShapeEnum::Compound), 0);
    }
}
