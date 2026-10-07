//! HLRBRep_PolyAlgo.cxx: StoreShell, Normal, AverageNormal and AddNormalOnTriangle.
use super::orient::{Arrays, NodeH, add_flag, data, node_h, pin, seg, tri};
use super::{Frame, NMSK_NORM, NMSK_OUT_L, PolyState, null_algo};
use crate::hlr_algo::{PolyData, PolyInternalData, PolyShellData, TriangleData};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomRectangularTrimmedSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::cslib::{self, DerivativeStatus};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Xyz;
use rustycad_tkmath::poly::{Triangulation, mesh_purpose};

/// The arrays StoreShell fills across the shells (`theIShell`, `thePD`, `thePID`,
/// `theShapeMap1`, `theShapeMap2`; the shell index of each edge is never read).
pub(super) struct ShellStore {
    pub(super) i_shell: i32,
    pub(super) pd: Vec<Handle<PolyData>>,
    pub(super) pid: Vec<Handle<PolyInternalData>>,
    pub(super) shape_map1: MapOfShape,
    pub(super) shape_map2: MapOfShape,
}

fn triangulation(
    face: &Face,
) -> Result<(Handle<Triangulation>, rustycad_tkmath::toploc::Location), Failure> {
    BRepTool::triangulation(face, mesh_purpose::NONE)
}

fn tri_of(t: &Handle<Triangulation>) -> Result<&Triangulation, Failure> {
    t.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "HLRBRep_PolyAlgo: a null triangulation",
        )
    })
}

// An open shell is closed when each edge is used as many times FORWARD as REVERSED.
fn edges_balanced(shape: &Shape) -> Result<bool, Failure> {
    let mut em = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, ShapeEnum::Edge, &mut em)?;
    let nb_edge = em.extent();
    let mut flags = vec![0i32; nb_edge.max(1) + 1];
    for e in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&e)?;
        let index = em.find_index(&e);
        if BRepTool::degenerated(&edge)? {
            continue;
        }
        match edge.orientation() {
            Orientation::Forward => flags[index] += 1,
            Orientation::Reversed => flags[index] -= 1,
            _ => {}
        }
    }
    Ok((1..=nb_edge).all(|k| flags[k] == 0))
}

fn planar(face: &Face) -> Result<bool, Failure> {
    let mut surf = BRepTool::surface(face)?;
    if surf.is_null() {
        return Ok(false);
    }
    let trimmed = surf.cast::<GeomRectangularTrimmedSurface>();
    if let Some(t) = trimmed.get() {
        surf = t.basis_surface();
    }
    Ok(GeomAdaptorSurface::with_surface(&surf)?.get_type()? == SurfaceType::Plane)
}

impl PolyState {
    /// StoreShell(S, iShell, Shell, IsoledF, IsoledE, ES, PD, PID, Map1, Map2): the
    /// polyhedral shell of the triangulated faces of `shape` (or its isolated edges):
    /// the faces as internal triangulations with oriented triangles, the edges as
    /// segments, the outlines inserted, then the polyhedral data of each face.
    pub(super) fn store_shell(
        &mut self,
        shape: &Shape,
        st: &mut ShellStore,
        isoled_f: bool,
        isoled_e: bool,
    ) -> Result<(), Failure> {
        let mut nb_face_shell = 0;
        let mut is_closed = false;
        self.t = Frame::of(&self.proj.transformation())?;
        self.ti = Frame::of(&self.proj.inverted_transformation())?;
        let avoid = if isoled_f {
            ShapeEnum::Shell
        } else {
            ShapeEnum::Shape
        };
        if !isoled_e {
            if !isoled_f {
                is_closed = shape.closed()? || edges_balanced(shape)?;
            }
            for face in Explorer::new(shape, ShapeEnum::Face, avoid)? {
                let (tr, _) = triangulation(&topods::face(&face)?)?;
                if !tr.is_null() && st.shape_map1.add(face) {
                    nb_face_shell += 1;
                }
            }
        }
        let algo = self.algo.clone();
        let algo = algo.get().ok_or_else(null_algo)?;
        if nb_face_shell > 0 || isoled_e {
            st.i_shell += 1;
            let psd = Handle::new(PolyShellData::new(nb_face_shell)?);
            algo.change_poly_shell()
                .set_value(st.i_shell, psd)
                .map_err(|e| e.into_failure("HLRBRep_PolyAlgo::StoreShell"))?;
        }
        let psd = algo
            .poly_shell()
            .value(st.i_shell)
            .cloned()
            .unwrap_or_default();
        if nb_face_shell > 0 {
            let psd = psd
                .get()
                .ok_or_else(|| Failure::new("HLRBRep_PolyAlgo::StoreShell: no shell"))?;
            let mut i_face = 0;
            for face in Explorer::new(shape, ShapeEnum::Face, avoid)? {
                let f = topods::face(&face)?;
                let (tr, _) = triangulation(&f)?;
                if !tr.is_null() && st.shape_map2.add(face) {
                    i_face += 1;
                    self.store_face(&f, i_face, psd, st)?;
                }
            }
            self.store_edges(shape, psd, st, is_closed)?;
        } else if isoled_e {
            let psd = psd
                .get()
                .ok_or_else(|| Failure::new("HLRBRep_PolyAlgo::StoreShell: no shell"))?;
            let mut list = psd.edges();
            for e in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Face)? {
                if st.shape_map1.add(e.clone()) {
                    let edge_index = self.e_map.find_index(&e) as i32;
                    let mut edge = topods::edge(&e)?;
                    self.init_bipoints_with_connexity(
                        edge_index,
                        &mut edge,
                        &mut list,
                        &st.pid,
                        &List::default(),
                        false,
                    )?;
                }
            }
        }
        Ok(())
    }

    // The segments of the edges of the faces, the outlines, then the polyhedral data.
    fn store_edges(
        &mut self,
        shape: &Shape,
        psd: &PolyShellData,
        st: &mut ShellStore,
        is_closed: bool,
    ) -> Result<(), Failure> {
        let nb_faces = self.f_map.extent();
        let mut list = psd.edges();
        let mut ef = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut ef)?;
        for face_iter in 1..=nb_faces {
            if st.pid[face_iter].is_null() {
                continue;
            }
            let face = self.f_map.find_key(face_iter)?.clone();
            for e in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !st.shape_map1.add(e.clone()) {
                    continue;
                }
                let edge_index = self.e_map.find_index(&e) as i32;
                let mut edge = topods::edge(&e)?;
                let index_e = ef.find_index(&e);
                if index_e > 0 {
                    let ls = ef.find_from_index(index_e)?.clone();
                    self.init_bipoints_with_connexity(
                        edge_index, &mut edge, &mut list, &st.pid, &ls, true,
                    )?;
                } else {
                    self.init_bipoints_with_connexity(
                        edge_index,
                        &mut edge,
                        &mut list,
                        &st.pid,
                        &List::default(),
                        false,
                    )?;
                }
            }
        }
        self.insert_on_outline(&st.pid)?;
        self.check_fr_back_triangles(&mut list, &st.pid)?;
        self.update_outlines(&mut list, &st.pid)?;
        self.update_edges_bipoints(&mut list, &st.pid, is_closed)?;
        self.update_poly_data(&st.pd, &st.pid, is_closed)?;
        for pid in st.pid.iter_mut().skip(1) {
            *pid = Handle::default();
        }
        Ok(())
    }

    // One triangulated face: its polyhedral data in the shell, its internal
    // triangulation (nodes in the view frame, links, normals and scalars), and the
    // orientation of its triangles.
    fn store_face(
        &mut self,
        face: &Face,
        i_face: i32,
        psd: &PolyShellData,
        st: &mut ShellStore,
    ) -> Result<(), Failure> {
        let (tr, loc) = triangulation(face)?;
        let tri_ = tri_of(&tr)?;
        let face_index = self.f_map.find_index(&Shape::from(face.clone()));
        let is_reversed = face.orientation() == Orientation::Reversed;
        let mut tt = loc.transformation();
        tt.pre_multiply(self.proj.transformation());
        self.tt = Frame::of(&tt)?;
        let (nb_nodes, nb_tris) = (tri_.nb_nodes(), tri_.nb_triangles());
        st.pd[face_index] = Handle::new(PolyData::new());
        psd.poly_data()
            .set_value(i_face, st.pd[face_index].clone())
            .map_err(|e| e.into_failure("HLRBRep_PolyAlgo::StoreShell"))?;
        st.pid[face_index] = Handle::new(PolyInternalData::new(nb_nodes, nb_tris)?);
        let pid = st.pid[face_index].clone();
        let pid = pid.get().ok_or_else(|| Failure::new("no internal data"))?;
        pid.set_planar(planar(face)?);
        let a = Arrays::of(pid);
        self.fill_triangles(tri_, &a, nb_tris, is_reversed)?;
        for i in 1..=nb_nodes {
            let n = node_h(&a.n, i)?;
            let mut p = Xyz::from(tri_.node(i)?);
            self.tt.apply_xyz(&mut p, false);
            {
                let mut ind = pin(&n)?.indices();
                ind.nd_sg = 0;
                ind.flag = 0;
            }
            pin(&n)?.data().point = p;
        }
        {
            let t = a.t.get().ok_or_else(|| Failure::new("no triangles"))?;
            let s = a.s.get().ok_or_else(|| Failure::new("no segments"))?;
            let nn = a.n.get().ok_or_else(|| Failure::new("no nodes"))?;
            pid.update_links(&mut t.change_array1(), &mut s.change_array1(), &nn.array1())?;
        }
        if tri_.has_uv_nodes() {
            self.store_normals(face, tri_, &a, nb_nodes)?;
        }
        for i in 1..=nb_tris {
            let mut td = tri(&a.t, i)?;
            let nodes = [
                node_h(&a.n, td.node1)?,
                node_h(&a.n, td.node2)?,
                node_h(&a.n, td.node3)?,
            ];
            self.orient_triangle(&mut td, &nodes)?;
            super::orient::set_tri(&a.t, i, td)?;
        }
        Ok(())
    }

    fn fill_triangles(
        &self,
        tri_: &Triangulation,
        a: &Arrays,
        nb_tris: i32,
        is_reversed: bool,
    ) -> Result<(), Failure> {
        for i in 1..=nb_tris {
            let (n1, n2, n3) = tri_.triangle(i)?.get();
            let td = if is_reversed {
                TriangleData {
                    node1: n3,
                    node2: n2,
                    node3: n1,
                    flags: 0,
                }
            } else {
                TriangleData {
                    node1: n1,
                    node2: n2,
                    node3: n3,
                    flags: 0,
                }
            };
            super::orient::set_tri(&a.t, i, td)?;
        }
        Ok(())
    }

    // The surface parameters, normals and scalars of the nodes.
    fn store_normals(
        &mut self,
        face: &Face,
        tri_: &Triangulation,
        a: &Arrays,
        nb_nodes: i32,
    ) -> Result<(), Failure> {
        let has_surf = BRepTool::is_geometric_face(face)?;
        self.b_surf.initialize(face, false)?;
        for i in 1..=nb_nodes {
            let n = node_h(&a.n, i)?;
            pin(&n)?.data().uv = tri_.uv_node(i)?.xy();
            if tri_.has_normals() {
                pin(&n)?.data().normal = Xyz::from(tri_.normal(i)?);
            }
            if (tri_.has_normals() || has_surf) && self.normal(i, &n, a, false)? {
                add_flag(&n, NMSK_NORM)?;
            } else {
                pin(&n)?.indices().flag &= !NMSK_NORM;
                pin(&n)?.data().scal = 0.;
            }
        }
        Ok(())
    }

    /// Normal(iNode, NodIndices, Nod1RValues, TData, PISeg, PINod, orient): the
    /// normal of the node in the view frame (from the surface when it has none),
    /// turned like the triangles around it, and its scalar with the sight (0 on the
    /// outline); false when the surface normal is undefined.
    pub(super) fn normal(
        &self,
        i_node: i32,
        n: &NodeH,
        a: &Arrays,
        to_orient: bool,
    ) -> Result<bool, Failure> {
        let d = data(n)?;
        if d.normal.square_modulus() < precision::confusion() {
            let d1 = self.b_surf.d1(d.uv.x(), d.uv.y())?;
            let r = cslib::normal(d1.d1u, d1.d1v, precision::angular())?;
            let norm = match (r.status, r.normal) {
                (DerivativeStatus::Done, Some(norm)) => norm,
                _ => {
                    let d2 = self.b_surf.d2(d.uv.x(), d.uv.y())?;
                    let r2 = cslib::normal_from_second_derivatives(
                        d2.d1u,
                        d2.d1v,
                        d2.d2u,
                        d2.d2v,
                        d2.d2uv,
                        precision::angular(),
                    );
                    match (r2.is_done(), r2.normal) {
                        (true, Some(norm)) => norm,
                        _ => return Ok(false),
                    }
                }
            };
            pin(n)?.data().normal = Xyz::from(norm);
        }
        let mut normal = data(n)?.normal;
        self.t.apply_xyz(&mut normal, self.proj.perspective());
        pin(n)?.data().normal = normal;
        let (ok, aver) = self.average_normal(i_node, n, a)?;
        let mut scal;
        if ok {
            if normal * aver < 0. {
                normal.reverse();
                pin(n)?.data().normal = normal;
            }
            let mut eye = Xyz::new(0., 0., -1.);
            if self.proj.perspective() {
                let p = data(n)?.point;
                eye = Xyz::new(p.x(), p.y(), p.z() - self.proj.focus()?);
                let eye_mod = eye.modulus();
                if eye_mod > 0. {
                    eye /= eye_mod;
                }
            }
            scal = normal * eye;
        } else {
            scal = 0.;
            pin(n)?.data().normal = Xyz::new(1., 0., 0.);
        }
        if (scal > 0. && scal < self.tol_angular) || (scal <= 0. && -scal < self.tol_angular) {
            scal = 0.;
            add_flag(n, NMSK_OUT_L)?;
        }
        pin(n)?.data().scal = scal;
        if to_orient {
            self.update_around_node(i_node, n, a)?;
        }
        Ok(true)
    }

    /// AverageNormal(iNode, NodeIndices, TData, PISeg, PINod, X, Y, Z): the sum of the
    /// normals of the triangles around the node (through a neighbour when a triangle
    /// is degenerated at it); false when none or when the sum is null.
    pub(super) fn average_normal(
        &self,
        i_node: i32,
        n: &NodeH,
        a: &Arrays,
    ) -> Result<(bool, Xyz), Failure> {
        let mut ok = false;
        let (mut j_node, mut k_node) = (0, 0);
        let mut sum = [0.; 3];
        let nd_sg = pin(n)?.indices().nd_sg;
        let mut iiii = nd_sg;
        while iiii != 0 && !ok {
            let sg = seg(&a.s, iiii)?;
            for t in [sg.conex1, sg.conex2] {
                if t != 0 {
                    self.add_normal_on_triangle(t, i_node, &mut j_node, a, &mut sum, &mut ok)?;
                }
            }
            iiii = if sg.lst_sg1 == i_node {
                sg.nxt_sg1
            } else {
                sg.nxt_sg2
            };
        }
        if j_node != 0 {
            let mut iiii = nd_sg;
            while iiii != 0 && !ok {
                let sg = seg(&a.s, iiii)?;
                for t in [sg.conex1, sg.conex2] {
                    if t != 0 {
                        self.add_normal_on_triangle(t, j_node, &mut k_node, a, &mut sum, &mut ok)?;
                    }
                }
                iiii = if sg.lst_sg1 == j_node {
                    sg.nxt_sg1
                } else {
                    sg.nxt_sg2
                };
            }
        }
        let d = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
        if ok && d < 1.0e-10 {
            ok = false;
        }
        Ok((ok, Xyz::new(sum[0], sum[1], sum[2])))
    }

    /// AddNormalOnTriangle(iTri, iNode, jNode, TData, PINod, X, Y, Z, OK): adds the
    /// unit-scaled normal of a non-degenerated triangle; at a degenerated side through
    /// the node, `j_node` becomes the other end of that side.
    fn add_normal_on_triangle(
        &self,
        i_tri: i32,
        i_node: i32,
        j_node: &mut i32,
        a: &Arrays,
        sum: &mut [f64; 3],
        ok: &mut bool,
    ) -> Result<(), Failure> {
        let t = tri(&a.t, i_tri)?;
        let p1 = data(&node_h(&a.n, t.node1)?)?.point;
        let p2 = data(&node_h(&a.n, t.node2)?)?.point;
        let p3 = data(&node_h(&a.n, t.node3)?)?.point;
        let d1 = p2 - p1;
        let d1_norm = d1.modulus();
        if d1_norm < 1.0e-10 {
            if t.node1 == i_node {
                *j_node = t.node2;
            } else if t.node2 == i_node {
                *j_node = t.node1;
            }
            return Ok(());
        }
        let d2 = p3 - p2;
        let d2_norm = d2.modulus();
        if d2_norm < 1.0e-10 {
            if t.node2 == i_node {
                *j_node = t.node3;
            } else if t.node3 == i_node {
                *j_node = t.node2;
            }
            return Ok(());
        }
        let d3_norm = (p1 - p3).modulus();
        if d3_norm < 1.0e-10 {
            if t.node3 == i_node {
                *j_node = t.node1;
            } else if t.node1 == i_node {
                *j_node = t.node3;
            }
            return Ok(());
        }
        let dn = (1. / (d1_norm * d2_norm)) * (d1 ^ d2);
        if dn.modulus() > 1.0e-10 {
            *ok = true;
            sum[0] += dn.x();
            sum[1] += dn.y();
            sum[2] += dn.z();
        }
        Ok(())
    }
}
