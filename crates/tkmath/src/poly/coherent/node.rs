// Rust translation of FoundationClasses/TKMath/Poly/Poly_CoherentNode.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{CoherentTriPtr, CoherentTriangle};
use crate::gp::Xyz;
use rustycad_tkernel::{ncollection::BaseAllocator, precision, standard::Failure};
use std::io::{self, Write};
/// `Poly_CoherentNode`: a point (it derefs to `gp_XYZ`), UV, normal, original index and the
/// ring of incident triangles; `CoherentNode::new(Xyz::new(0.0, 0.0, 0.0))`.
#[derive(Clone, Debug)]
pub struct CoherentNode {
    point: Xyz,
    uv: [f64; 2],
    triangles: Option<CoherentTriPtr>,
    index: i32,
    normal: [f32; 3],
}
impl Default for CoherentNode {
    // The native default constructor leaves the normal uninitialized; it is zero here,
    // as in the point constructor.
    fn default() -> Self {
        Self::new(Xyz::new(0., 0., 0.))
    }
}
impl std::ops::Deref for CoherentNode {
    type Target = Xyz;
    fn deref(&self) -> &Xyz {
        &self.point
    }
}
impl std::ops::DerefMut for CoherentNode {
    fn deref_mut(&mut self) -> &mut Xyz {
        &mut self.point
    }
}
impl CoherentNode {
    /// A free node at a point, with infinite UV and a null normal; `CoherentNode::new(point)`.
    pub fn new(point: Xyz) -> Self {
        Self {
            point,
            uv: [precision::infinite(); 2],
            triangles: None,
            index: -1,
            normal: [0.0; 3],
        }
    }
    /// Sets the UV coordinates; `node.set_uv(0.5, 0.25)`.
    pub const fn set_uv(&mut self, u: f64, v: f64) {
        self.uv = [u, v];
    }
    /// U coordinate; `node.get_u()`.
    pub const fn get_u(&self) -> f64 {
        self.uv[0]
    }
    /// V coordinate; `node.get_v()`.
    pub const fn get_v(&self) -> f64 {
        self.uv[1]
    }
    /// Stores the normal narrowed to single precision; `node.set_normal(Xyz::new(0.0, 0.0, 1.0))`.
    pub fn set_normal(&mut self, vector: Xyz) {
        self.normal = [vector.x() as f32, vector.y() as f32, vector.z() as f32];
    }
    /// True when the single-precision squared length exceeds the confusion; `node.has_normal()`.
    pub fn has_normal(&self) -> bool {
        let [x, y, z] = self.normal;
        f64::from(x * x + y * y + z * z) > precision::confusion()
    }
    /// The stored normal widened to double; `node.get_normal()`.
    pub fn get_normal(&self) -> Xyz {
        let [x, y, z] = self.normal.map(f64::from);
        Xyz::new(x, y, z)
    }
    /// Sets the original index; `node.set_index(3)`.
    pub const fn set_index(&mut self, index: i32) {
        self.index = index;
    }
    /// The original index, -1 by default; `node.get_index()`.
    pub const fn get_index(&self) -> i32 {
        self.index
    }
    /// True without incident triangles; `node.is_free_node()`.
    pub const fn is_free_node(&self) -> bool {
        self.triangles.is_none()
    }
    /// Drops the triangles and resets point, UV and normal, keeping the index; `node.clear()`.
    pub fn clear(&mut self) {
        self.triangles = None;
        self.reset_geometry();
    }
    /// Clears the incident ring using its native allocator argument; `node.clear_in(&arena)?`.
    pub fn clear_in(&mut self, allocator: &BaseAllocator) -> Result<(), Failure> {
        if let Some(ring) = &mut self.triangles {
            ring.remove_list_in(allocator)?;
        }
        self.triangles = None;
        self.reset_geometry();
        Ok(())
    }
    fn reset_geometry(&mut self) {
        self.uv = [precision::infinite(); 2];
        self.normal = [0.0; 3];
        self.point = Xyz::new(0., 0., 0.);
    }
    /// Adds a triangle last in the ring, as `AddTriangle`; `node.add_triangle(4)`.
    pub fn add_triangle(&mut self, triangle: usize) {
        self.add_triangle_in(triangle, BaseAllocator::common())
            .expect("coherent node triangle ring has a live head");
    }
    /// Adds a triangle member through the supplied allocator; `node.add_triangle_in(4, arena)?`.
    pub fn add_triangle_in(
        &mut self,
        triangle: usize,
        allocator: BaseAllocator,
    ) -> Result<(), Failure> {
        match &mut self.triangles {
            None => self.triangles = Some(CoherentTriPtr::new_in(triangle, allocator)),
            Some(ring) => {
                let head = ring.head_member().ok_or_else(missing_triangle_ring)?;
                ring.prepend_member(head, triangle, allocator)?;
            }
        }
        Ok(())
    }
    /// Removes the head or the first later occurrence of a triangle; a free node reports
    /// false (natively a null dereference); `node.remove_triangle(4)`.
    pub fn remove_triangle(&mut self, triangle: usize) -> bool {
        self.remove_triangle_using(triangle, None).unwrap_or(false)
    }
    /// Removes a triangle member with the supplied allocator; `node.remove_triangle_in(4, &arena)?`.
    pub fn remove_triangle_in(
        &mut self,
        triangle: usize,
        allocator: &BaseAllocator,
    ) -> Result<bool, Failure> {
        self.remove_triangle_using(triangle, Some(allocator))
    }
    fn remove_triangle_using(
        &mut self,
        triangle: usize,
        allocator: Option<&BaseAllocator>,
    ) -> Result<bool, Failure> {
        let Some(ring) = &mut self.triangles else {
            return Ok(false);
        };
        let Some((member, position)) = member_of_triangle(ring, triangle) else {
            return Ok(false);
        };
        remove_ring_member(ring, member, position, allocator)?;
        if ring.is_empty() {
            self.triangles = None;
        }
        Ok(true)
    }
    /// The ring of incident triangles, `None` for a free node, as `TriangleIterator`;
    /// `node.triangle_iterator()`.
    pub const fn triangle_iterator(&self) -> Option<&CoherentTriPtr> {
        self.triangles.as_ref()
    }
    /// Head-first triangle indices, empty for a free node; `node.triangles().collect::<Vec<_>>()`.
    pub fn triangles(&self) -> impl Iterator<Item = usize> + '_ {
        self.triangles.iter().flat_map(CoherentTriPtr::iter)
    }
    /// Prints the point and the nodes of each incident triangle; `node.dump(&mut out, &triangles)?`.
    pub fn dump(&self, output: &mut impl Write, triangles: &[CoherentTriangle]) -> io::Result<()> {
        let (x, y, z) = (self.x(), self.y(), self.z());
        writeln!(
            output,
            "  X ={}; Y ={}; Z ={}",
            fixed4(x),
            fixed4(y),
            fixed4(z)
        )?;
        for triangle in self.triangles().filter_map(|t| triangles.get(t)) {
            let [a, b, c] = [0, 1, 2].map(|i| triangle.node(i));
            writeln!(output, "      {a:>5} {b:>5} {c:>5}")?;
        }
        Ok(())
    }
}
fn member_of_triangle(
    ring: &CoherentTriPtr,
    triangle: usize,
) -> Option<(super::CoherentTriMember, usize)> {
    for (position, current) in ring.iter().enumerate() {
        if current == triangle {
            return Some((ring.iter_from(position).member_token()?, position));
        }
    }
    None
}
fn remove_ring_member(
    ring: &mut CoherentTriPtr,
    member: super::CoherentTriMember,
    position: usize,
    allocator: Option<&BaseAllocator>,
) -> Result<(), Failure> {
    if let Some(allocator) = allocator {
        return ring.remove_member(member, allocator);
    }
    ring.remove(position);
    Ok(())
}
fn missing_triangle_ring() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "CoherentNode has no live triangle-ring member; expected a ring member",
        "",
    )
}
/// `printf("%9.4f")` in the C locale; `fixed4(1.0) == "   1.0000"`.
fn fixed4(value: f64) -> String {
    let text = if value.is_nan() {
        if value.is_sign_negative() {
            "-nan"
        } else {
            "nan"
        }
        .to_owned()
    } else {
        format!("{value:.4}")
    };
    format!("{text:>9}")
}
