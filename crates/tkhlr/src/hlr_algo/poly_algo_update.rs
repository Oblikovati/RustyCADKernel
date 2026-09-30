// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_PolyAlgo.cxx (Update) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_PolyAlgo::Update: the packed boxes of segments, triangles, faces and
//! shells, and the hiding planes of the triangles.
//!
//! A box is packed on a grid over the global box of the projected data: 1020 cells
//! along X and Y and 508 along Z, the X index shifted by 21 bits, the Y index by 10,
//! the Z index in the low bits, with a guard offset of 0x200 on one side.
use super::failures::{array_failure, c_int_truncation, null_handle};
use super::poly_algo::PolyAlgo;
use super::poly_data::PolyData;
use super::poly_records::{PolyMask, TriangleData};
use super::poly_shell_data::PolyShellData;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Xyz;

const UPDATE: &str = "HLRAlgo_PolyAlgo::Update";

/// An axis-aligned box `[min, max]`, empty as `[big, -big]`.
#[derive(Clone, Copy)]
struct Extent {
    min: [f64; 3],
    max: [f64; 3],
}

impl Extent {
    fn empty() -> Self {
        let big = precision::infinite();
        Self {
            min: [big; 3],
            max: [-big; 3],
        }
    }

    /// Widens this box by `other` with the native `if (min > v) min = v` tests.
    fn add(&mut self, other: &Extent) {
        for k in 0..3 {
            if self.min[k] > other.min[k] {
                self.min[k] = other.min[k];
            }
            if self.max[k] < other.max[k] {
                self.max[k] = other.max[k];
            }
        }
    }
}

/// The packing grid of Update: the scale and the shift of each axis.
struct Grid {
    scale: [f64; 3],
    shift: [f64; 3],
}

impl Grid {
    /// The grid of the global box `(x_min, x_max, y_min, y_max, z_min, z_max)`; also
    /// returns the distance tolerance `precad * tol_param`.
    fn new(limits: [f64; 6], tol_param: f64) -> (Self, f64) {
        let [x_min, x_max, y_min, y_max, z_min, z_max] = limits;
        let (dx, dy, dz) = (x_max - x_min, y_max - y_min, z_max - z_min);
        let mut precad = dx;
        if precad < dy {
            precad = dy;
        }
        if precad < dz {
            precad = dz;
        }
        let tolerance = precad * tol_param;
        precad *= 0.01;
        let scale = [
            1020. / (dx + precad),
            1020. / (dy + precad),
            508. / (dz + precad),
        ];
        precad *= 0.5;
        let shift = [-x_min + precad, -y_min + precad, -z_min + precad];
        (Self { scale, shift }, tolerance)
    }

    /// The grid indices `(int)((shift + value) * scale)` of a corner.
    fn cell(&self, corner: &[f64; 3]) -> [i32; 3] {
        [0, 1, 2].map(|k| c_int_truncation((self.shift[k] + corner[k]) * self.scale[k]))
    }

    /// The packed index `((y + (x << 11)) << 10) + z + offset`.
    fn pack(&self, corner: &[f64; 3], offset: i32) -> i32 {
        let [x, y, z] = self.cell(corner);
        let packed = y.wrapping_add(x << 11) << 10;
        packed.wrapping_add(z.wrapping_add(offset))
    }

    /// The packed `(min, max)` of a box whose minimum carries the guard offset
    /// (triangles, faces and shells).
    fn pack_hiding(&self, extent: &Extent) -> (i32, i32) {
        (self.pack(&extent.min, -0x200), self.pack(&extent.max, 0))
    }
}

impl PolyAlgo {
    /// Update: from the global box of the projected segments and hiding triangles of
    /// every shell, packs the box of every segment, computes the plane, the box and
    /// the "greater along X" flags of every hiding triangle of every hiding face, packs
    /// the boxes of the hiding faces and of the shells, and lists the hiding faces of
    /// each shell. It also sets the distance tolerance of the hiding test
    /// (`1e-8 * the largest box extent`); `algo.update()?`.
    pub fn update(&self) -> Result<(), Failure> {
        let shells: Vec<Handle<PolyShellData>> = self.h_shell.read().iter().cloned().collect();
        self.state.write().nbr_shell = shells.len() as i32;
        let mut global = BoundingBox3::default();
        for shell in &shells {
            shell_of(shell)?.update_global_min_max(&mut global)?;
        }
        let limits = global.limits();
        let tol_param = self.state.read().triangle.tol_param;
        let (grid, tolerance) = Grid::new(
            [
                limits.x_min,
                limits.x_max,
                limits.y_min,
                limits.y_max,
                limits.z_min,
                limits.z_max,
            ],
            tol_param,
        );
        self.state.write().triangle.tolerance = tolerance;
        for shell in &shells {
            update_shell(shell_of(shell)?, &grid)?;
        }
        Ok(())
    }
}

fn shell_of(handle: &Handle<PolyShellData>) -> Result<&PolyShellData, Failure> {
    handle
        .get()
        .ok_or_else(|| null_handle(UPDATE, "myHShell(i)"))
}

fn face_of(handle: &Handle<PolyData>) -> Result<&PolyData, Failure> {
    handle.get().ok_or_else(|| null_handle(UPDATE, "a face"))
}

/// The segment boxes, the hiding faces and the packed box of one shell.
fn update_shell(shell: &PolyShellData, grid: &Grid) -> Result<(), Failure> {
    let mut extent = Extent::empty();
    for segment in shell.edges().iter_mut() {
        let points = segment.points();
        let segment_extent = ordered_extent(&points.pnt_p1, &points.pnt_p2);
        let indices = segment.indices_mut();
        indices.min_seg = grid.pack(&segment_extent.min, 0);
        indices.max_seg = grid.pack(&segment_extent.max, 0x200);
        extent.add(&segment_extent);
    }
    let faces = faces_of(shell)?;
    let mut nb_hiding = 0;
    for face in &faces {
        let face = face_of(face)?;
        if face.hiding() {
            nb_hiding += 1;
            extent.add(&update_face(face, grid)?);
        }
    }
    if nb_hiding == 0 {
        shell.update_hiding(0)?;
        *shell.indices() = Default::default();
        return Ok(());
    }
    let (min, max) = grid.pack_hiding(&extent);
    {
        let mut indices = shell.indices();
        (indices.min, indices.max) = (min, max);
    }
    shell.update_hiding(nb_hiding)?;
    let mut hidden_index = 1;
    for face in faces {
        if face_of(&face)?.hiding() {
            let mut hiding = shell.hiding_poly_data();
            hiding
                .set_value(hidden_index, face)
                .map_err(array_failure(UPDATE))?;
            hidden_index += 1;
        }
    }
    Ok(())
}

/// The faces `1..=Upper()` of a shell, as the native loops index them.
fn faces_of(shell: &PolyShellData) -> Result<Vec<Handle<PolyData>>, Failure> {
    let faces = shell.poly_data();
    (1..=faces.upper())
        .map(|j| faces.value(j).cloned().map_err(array_failure(UPDATE)))
        .collect()
}

/// The box of a segment from its projected ends (`p1 < p2` picks the order per axis).
fn ordered_extent(p1: &Xyz, p2: &Xyz) -> Extent {
    let (a, b) = (*p1.coordinates(), *p2.coordinates());
    let mut extent = Extent { min: a, max: b };
    for k in 0..3 {
        (extent.min[k], extent.max[k]) = if a[k] < b[k] {
            (a[k], b[k])
        } else {
            (b[k], a[k])
        };
    }
    extent
}

/// The planes, boxes and flags of the hiding triangles of one face, and its packed
/// box; returns the box of its hiding triangles.
fn update_face(face: &PolyData, grid: &Grid) -> Result<Extent, Failure> {
    let (nodes, triangles, hiding) = (face.nodes()?, face.t_data()?, face.ph_dat()?);
    let nodes = nodes
        .get()
        .ok_or_else(|| null_handle(UPDATE, "myHNodes"))?
        .array1();
    let triangles = triangles
        .get()
        .ok_or_else(|| null_handle(UPDATE, "myHTData"))?;
    let hiding = hiding
        .get()
        .ok_or_else(|| null_handle(UPDATE, "myHPHDat"))?;
    let (mut triangles, mut hiding) = (triangles.change_array1(), hiding.change_array1());
    let mut face_extent = Extent::empty();
    let mut plane = [0.0_f64; 3];
    let mut nb_hide = 0;
    for i in 1..=triangles.upper() {
        let td = triangles.change_value(i).map_err(array_failure(UPDATE))?;
        if td.flags & PolyMask::FMskHiding as i32 == 0 {
            continue;
        }
        let corners = triangle_corners(&nodes, td)?;
        let extent = triangle_extent(&corners);
        let (min, max) = grid.pack_hiding(&extent);
        let d = hiding_plane(&corners, &mut plane);
        nb_hide += 1;
        let data = hiding
            .change_value(nb_hide)
            .map_err(array_failure(UPDATE))?;
        data.set(i, min, max, plane[0], plane[1], plane[2], d);
        set_edge_flags(td, &corners);
        face_extent.add(&extent);
    }
    let (min, max) = grid.pack_hiding(&face_extent);
    let mut indices = face.indices();
    (indices.min, indices.max) = (min, max);
    Ok(face_extent)
}

fn triangle_corners(
    nodes: &Array1<'static, Xyz>,
    td: &TriangleData,
) -> Result<[[f64; 3]; 3], Failure> {
    let corner = |node: i32| -> Result<[f64; 3], Failure> {
        Ok(*nodes
            .value(node)
            .map_err(array_failure(UPDATE))?
            .coordinates())
    };
    Ok([corner(td.node1)?, corner(td.node2)?, corner(td.node3)?])
}

/// The box of a triangle with the native `if/else if` updates from the first corner.
fn triangle_extent(corners: &[[f64; 3]; 3]) -> Extent {
    let mut extent = Extent {
        min: corners[0],
        max: corners[0],
    };
    for corner in &corners[1..] {
        for (k, &value) in corner.iter().enumerate() {
            if extent.min[k] > value {
                extent.min[k] = value;
            } else if extent.max[k] < value {
                extent.max[k] = value;
            }
        }
    }
    extent
}

/// The unit normal of the triangle into `plane` (kept from the previous triangle of
/// the face when this one is degenerate, as the native locals are) and the plane
/// constant `d = n . P1`.
fn hiding_plane(corners: &[[f64; 3]; 3], plane: &mut [f64; 3]) -> f64 {
    let [[x1, y1, z1], [x2, y2, z2], [x3, y3, z3]] = *corners;
    let (dx1, dy1, dz1) = (x2 - x1, y2 - y1, z2 - z1);
    let (dx2, dy2, dz2) = (x3 - x2, y3 - y2, z3 - z2);
    let dnx = dy1 * dz2 - dy2 * dz1;
    let dny = dz1 * dx2 - dz2 * dx1;
    let dnz = dx1 * dy2 - dx2 * dy1;
    let dn = (dnx * dnx + dny * dny + dnz * dnz).sqrt();
    if dn > 0. {
        *plane = [dnx / dn, dny / dn, dnz / dn];
    }
    plane[0] * x1 + plane[1] * y1 + plane[2] * z1
}

/// The "greater along X" flag of each edge: set when `|dx| > |dy|` on the edge.
fn set_edge_flags(td: &mut TriangleData, corners: &[[f64; 3]; 3]) {
    let edges = [
        (0, 1, PolyMask::EMskGrALin1),
        (1, 2, PolyMask::EMskGrALin2),
        (2, 0, PolyMask::EMskGrALin3),
    ];
    for (from, to, mask) in edges {
        let dx = corners[to][0] - corners[from][0];
        let dy = corners[to][1] - corners[from][1];
        let adx = if dx < 0. { -dx } else { dx };
        let ady = if dy < 0. { -dy } else { dy };
        if adx > ady {
            td.flags |= mask as i32;
        } else {
            td.flags &= !(mask as i32);
        }
    }
}
