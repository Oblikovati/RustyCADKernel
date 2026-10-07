//! FNV-1a hashes that pin native triangulation orders in the TKMesh targets.
// Each including target uses only some of the hashes.
#![allow(dead_code)]

/// FNV-1a over every node position and UV bit pattern; pins a native node order.
pub fn cylinder_node_hash(triangulation: &rustycad_tkmath::poly::Triangulation) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    hash_cylinder_nodes(&mut hash, triangulation);
    hash
}

/// FNV-1a over every triangle node index; pins a native triangle order.
pub fn cylinder_triangle_hash(triangulation: &rustycad_tkmath::poly::Triangulation) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for index in 1..=triangulation.nb_triangles() {
        let triangle = triangulation.triangle(index).unwrap().get();
        for node in [triangle.0, triangle.1, triangle.2] {
            hash = hash_word(hash, node as u64);
        }
    }
    hash
}

fn hash_cylinder_nodes(hash: &mut u64, triangulation: &rustycad_tkmath::poly::Triangulation) {
    for index in 1..=triangulation.nb_nodes() {
        let point = triangulation.node(index).unwrap();
        let uv = triangulation.uv_node(index).unwrap();
        for value in [point.x(), point.y(), point.z(), uv.x(), uv.y()] {
            *hash = hash_word(*hash, value.to_bits());
        }
    }
}

fn hash_word(mut hash: u64, value: u64) -> u64 {
    hash ^= value;
    hash.wrapping_mul(0x100_0000_01b3)
}
