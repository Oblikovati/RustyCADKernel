// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_ShellFaceSet,
// TopOpeBRepBuild_ShellFaceClassifier, TopOpeBRepBuild_Area3dBuilder,
// TopOpeBRepBuild_SolidAreaBuilder and TopOpeBRepBuild_SolidBuilder (.hxx/.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solid builder: the faces of a shell/face set grouped into shells and solids by
//! the shell/face classifier.
use super::area_builder::TopOpeBRepBuildAreaBuilder;
use super::block_builder::TopOpeBRepBuildBlockBuilder;
use super::composite_classifier::TopOpeBRepBuildCompositeClassifier;
use super::loops::{
    TopOpeBRepBuildBlockIterator, TopOpeBRepBuildLoop, TopOpeBRepBuildLoopBase,
    TopOpeBRepBuildLoopClassifier, TopOpeBRepBuildLoopSet, TopOpeBRepBuildPlainLoopSet,
};
use super::shape_set::{TopOpeBRepBuildShapeSet, TopOpeBRepBuildShapeSetBase};
use crate::top_ope_brep_tool::TopOpeBRepToolSolidClassifier;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, Shell, Solid};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::Point3;
use std::ops::{Deref, DerefMut};

/// The shells and faces of a solid (`TopOpeBRepBuild_ShellFaceSet`), a shape set of
/// faces connected by edges; `let mut sfs = TopOpeBRepBuildShellFaceSet::with_solid(&so)?;`.
pub struct TopOpeBRepBuildShellFaceSet {
    base: TopOpeBRepBuildShapeSetBase,
    my_solid: Solid,
}

impl TopOpeBRepBuildShellFaceSet {
    /// `TopOpeBRepBuild_ShellFaceSet()`: no solid; `TopOpeBRepBuildShellFaceSet::new()?`.
    pub fn new() -> Result<Self, Failure> {
        Ok(Self {
            base: TopOpeBRepBuildShapeSetBase::new(ShapeEnum::Edge, true)?,
            my_solid: Solid::default(),
        })
    }

    /// `TopOpeBRepBuild_ShellFaceSet(S, Addr)`: the set of the solid `s`
    /// (Standard_TypeMismatch for another shape); `TopOpeBRepBuildShellFaceSet::with_solid(&so)?`.
    pub fn with_solid(s: &Shape) -> Result<Self, Failure> {
        let mut sfs = Self::new()?;
        sfs.my_solid = topods::solid(s)?;
        Ok(sfs)
    }

    /// `Solid()`; `sfs.solid()`.
    pub fn solid(&self) -> &Solid {
        &self.my_solid
    }
}

impl TopOpeBRepBuildShapeSet for TopOpeBRepBuildShellFaceSet {
    fn shape_set_base(&self) -> &TopOpeBRepBuildShapeSetBase {
        &self.base
    }

    fn shape_set_base_mut(&mut self) -> &mut TopOpeBRepBuildShapeSetBase {
        &mut self.base
    }

    /// `AddStartElement(S)`: `ProcessAddStartElement`.
    fn add_start_element(&mut self, s: &Shape) -> Result<(), Failure> {
        self.process_add_start_element(s)
    }

    /// `SName(S, sb, sa)`: `sb`.
    fn sname(&self, _s: &Shape, sb: &str, _sa: &str) -> String {
        sb.to_string()
    }

    /// `SNameori(S, sb, sa)`: `sb`.
    fn snameori(&self, _s: &Shape, sb: &str, _sa: &str) -> String {
        sb.to_string()
    }
}

/// The classifier of shells and faces (`TopOpeBRepBuild_ShellFaceClassifier`) by a
/// point of a face against a shell; natively the first compare flag starts
/// uninitialized (false here); `let mut sfc = TopOpeBRepBuildShellFaceClassifier::new(&bb);`.
pub struct TopOpeBRepBuildShellFaceClassifier<'a> {
    my_block_builder: &'a TopOpeBRepBuildBlockBuilder,
    my_first_compare: bool,
    my_point3d: Point3,
    my_shell: Shell,
    my_builder: BRepBuilder,
    my_solid_classifier: TopOpeBRepToolSolidClassifier,
    my_face_shell_map: DataMap<Shape, Shape, ShapeMapHasher>,
}

impl<'a> TopOpeBRepBuildShellFaceClassifier<'a> {
    /// `TopOpeBRepBuild_ShellFaceClassifier(BB)`; `TopOpeBRepBuildShellFaceClassifier::new(&bb)`.
    pub fn new(bb: &'a TopOpeBRepBuildBlockBuilder) -> Self {
        Self {
            my_block_builder: bb,
            my_first_compare: false,
            my_point3d: Point3::default(),
            my_shell: Shell::default(),
            my_builder: BRepBuilder::default(),
            my_solid_classifier: TopOpeBRepToolSolidClassifier::new(),
            my_face_shell_map: DataMap::default(),
        }
    }

    /// `Clear()`: the solid classifier and the shells of the faces cleared;
    /// `sfc.clear()`.
    pub fn clear(&mut self) {
        self.my_solid_classifier.clear();
        self.my_face_shell_map.clear(false);
    }
}

impl TopOpeBRepBuildCompositeClassifier for TopOpeBRepBuildShellFaceClassifier<'_> {
    fn block_builder(&self) -> &TopOpeBRepBuildBlockBuilder {
        self.my_block_builder
    }

    /// `CompareShapes(B1, B2)`: the point of `b1` against the shell `b2`.
    fn compare_shapes(&mut self, b1: &Shape, b2: &Shape) -> Result<State, Failure> {
        self.reset_shape(b1)?;
        self.my_shell = topods::shell(b2)?;
        self.my_solid_classifier.load_shell(&self.my_shell)?;
        Ok(self.state())
    }

    /// `CompareElementToShape(F, SHE)`: the point of the face `f` against the shell.
    fn compare_element_to_shape(&mut self, f: &Shape, she: &Shape) -> Result<State, Failure> {
        self.reset_element(f)?;
        self.my_shell = topods::shell(she)?;
        self.my_solid_classifier.load_shell(&self.my_shell)?;
        Ok(self.state())
    }

    /// `ResetShape(SHE)`: `ResetElement` on the first face of the shell;
    /// Standard_NoSuchObject without face (the explorer read past its end).
    fn reset_shape(&mut self, she: &Shape) -> Result<(), Failure> {
        let f = Iterator::next(&mut Explorer::new(she, ShapeEnum::Face, ShapeEnum::Shape)?)
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_NoSuchObject",
                    "TopExp_Explorer::Current",
                    "TopExp_Explorer",
                )
            })?;
        self.reset_element(&topods::face(&f)?.into_shape())
    }

    /// `ResetElement(F)`: the point of the first vertex of the face, else the middle
    /// of its parameter range (the origin for another shape).
    fn reset_element(&mut self, f: &Shape) -> Result<(), Failure> {
        let t = f.shape_type()?;
        self.my_first_compare = true;
        if let Some(v) = Iterator::next(&mut Explorer::new(f, ShapeEnum::Vertex, ShapeEnum::Shape)?)
        {
            self.my_point3d = BRepTool::pnt(&topods::vertex(&v)?)?;
        } else if t == ShapeEnum::Face {
            let bas = BRepAdaptorSurface::with_face(&topods::face(f)?, true)?;
            let u = 0.5 * (bas.first_u_parameter()? + bas.last_u_parameter()?);
            let v = 0.5 * (bas.first_v_parameter()? + bas.last_v_parameter()?);
            self.my_point3d = bas.value(u, v)?;
        } else {
            self.my_point3d = Point3::new(0., 0., 0.);
        }
        Ok(())
    }

    /// `CompareElement(F)`: the first face starts a shell (or takes the shell already
    /// made for it: false), the next faces are added to it.
    fn compare_element(&mut self, f: &Shape) -> Result<bool, Failure> {
        if !self.my_first_compare {
            self.my_builder.add(&mut self.my_shell, f)?;
            return Ok(true);
        }
        self.my_first_compare = false;
        if let Some(sbid) = self.my_face_shell_map.seek(f) {
            self.my_shell = topods::shell(sbid)?;
            return Ok(false);
        }
        self.my_builder.make_shell(&mut self.my_shell);
        self.my_builder.add(&mut self.my_shell, f)?;
        self.my_face_shell_map
            .bind(f.clone(), self.my_shell.clone().into_shape());
        Ok(true)
    }

    /// `State()`: the point against the shell, at Precision::Confusion.
    fn state(&mut self) -> State {
        let tol3d = precision::confusion();
        let (shell, p) = (self.my_shell.clone(), self.my_point3d);
        // a failed classification leaves the classifier state, as natively unchecked
        let _ = self.my_solid_classifier.classify_shell(&shell, p, tol3d);
        self.my_solid_classifier.state()
    }
}

impl TopOpeBRepBuildLoopClassifier for TopOpeBRepBuildShellFaceClassifier<'_> {
    /// `Compare(L1, L2)`: the composite classification.
    fn compare(
        &mut self,
        l1: &Handle<dyn TopOpeBRepBuildLoop>,
        l2: &Handle<dyn TopOpeBRepBuildLoop>,
    ) -> Result<State, Failure> {
        self.composite_compare(l1, l2)
    }
}

/// The 3d area builder (`TopOpeBRepBuild_Area3dBuilder`): the areas of the base
/// algorithm; `TopOpeBRepBuildArea3dBuilder::with_loops(&mut ls, &mut lc, false)?`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildArea3dBuilder {
    parent: TopOpeBRepBuildAreaBuilder,
}

impl Deref for TopOpeBRepBuildArea3dBuilder {
    type Target = TopOpeBRepBuildAreaBuilder;

    fn deref(&self) -> &TopOpeBRepBuildAreaBuilder {
        &self.parent
    }
}

impl DerefMut for TopOpeBRepBuildArea3dBuilder {
    fn deref_mut(&mut self) -> &mut TopOpeBRepBuildAreaBuilder {
        &mut self.parent
    }
}

impl TopOpeBRepBuildArea3dBuilder {
    /// `TopOpeBRepBuild_Area3dBuilder()`; `TopOpeBRepBuildArea3dBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_Area3dBuilder(LS, LC, ForceClass)`;
    /// `TopOpeBRepBuildArea3dBuilder::with_loops(&mut ls, &mut lc, false)?`.
    pub fn with_loops(
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<Self, Failure> {
        let mut ab = Self::new();
        ab.init_area_builder(ls, lc, force_class)?;
        Ok(ab)
    }

    /// `InitAreaBuilder(LS, LC, ForceClass)`: the 3d algorithm, which is the base one;
    /// `ab.init_area_builder(&mut ls, &mut lc, false)?`.
    pub fn init_area_builder(
        &mut self,
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<(), Failure> {
        self.parent.init_area_builder(ls, lc, force_class)
    }
}

/// The solid area builder (`TopOpeBRepBuild_SolidAreaBuilder`);
/// `TopOpeBRepBuildSolidAreaBuilder::with_loops(&mut ls, &mut lc, false)?`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildSolidAreaBuilder {
    parent: TopOpeBRepBuildArea3dBuilder,
}

impl Deref for TopOpeBRepBuildSolidAreaBuilder {
    type Target = TopOpeBRepBuildArea3dBuilder;

    fn deref(&self) -> &TopOpeBRepBuildArea3dBuilder {
        &self.parent
    }
}

impl DerefMut for TopOpeBRepBuildSolidAreaBuilder {
    fn deref_mut(&mut self) -> &mut TopOpeBRepBuildArea3dBuilder {
        &mut self.parent
    }
}

impl TopOpeBRepBuildSolidAreaBuilder {
    /// `TopOpeBRepBuild_SolidAreaBuilder()`; `TopOpeBRepBuildSolidAreaBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_SolidAreaBuilder(LS, LC, ForceClass)`;
    /// `TopOpeBRepBuildSolidAreaBuilder::with_loops(&mut ls, &mut lc, false)?`.
    pub fn with_loops(
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<Self, Failure> {
        let mut ab = Self::new();
        ab.init_solid_area_builder(ls, lc, force_class)?;
        Ok(ab)
    }

    /// `InitSolidAreaBuilder(LS, LC, ForceClass)`; `ab.init_solid_area_builder(&mut ls, &mut lc, false)?`.
    pub fn init_solid_area_builder(
        &mut self,
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<(), Failure> {
        self.parent.init_area_builder(ls, lc, force_class)
    }
}

/// The solid builder (`TopOpeBRepBuild_SolidBuilder`): the solids, shells and faces of
/// a shell/face set; `let sb = TopOpeBRepBuildSolidBuilder::with_shell_face_set(&mut sfs, false)?;`.
#[derive(Default)]
pub struct TopOpeBRepBuildSolidBuilder {
    my_loop_set: TopOpeBRepBuildPlainLoopSet,
    my_block_iterator: TopOpeBRepBuildBlockIterator,
    my_block_builder: TopOpeBRepBuildBlockBuilder,
    my_solid_area_builder: TopOpeBRepBuildSolidAreaBuilder,
}

/// Standard_DomainError of `InitFace` on a shape loop.
fn domain_error() -> Failure {
    Failure::with_type(
        "Standard_DomainError",
        "TopOpeBRepBuild_SolidBuilder:InitFace",
        "TopOpeBRepBuild_SolidBuilder",
    )
}

impl TopOpeBRepBuildSolidBuilder {
    /// `TopOpeBRepBuild_SolidBuilder()`; `TopOpeBRepBuildSolidBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_SolidBuilder(SFS, ForceClass)`;
    /// `TopOpeBRepBuildSolidBuilder::with_shell_face_set(&mut sfs, false)?`.
    pub fn with_shell_face_set(
        sfs: &mut TopOpeBRepBuildShellFaceSet,
        force_class: bool,
    ) -> Result<Self, Failure> {
        let mut sb = Self::new();
        sb.init_solid_builder(sfs, force_class)?;
        Ok(sb)
    }

    /// `InitSolidBuilder(SFS, ForceClass)`: the loops of the set classified into solid
    /// areas; `sb.init_solid_builder(&mut sfs, false)?`.
    pub fn init_solid_builder(
        &mut self,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
        force_class: bool,
    ) -> Result<(), Failure> {
        self.make_loops(sfs)?;
        let mut sfc = TopOpeBRepBuildShellFaceClassifier::new(&self.my_block_builder);
        self.my_solid_area_builder.init_solid_area_builder(
            &mut self.my_loop_set,
            &mut sfc,
            force_class,
        )
    }

    /// `InitSolid()`: at the first solid, their count; `sb.init_solid()`.
    pub fn init_solid(&mut self) -> i32 {
        self.my_solid_area_builder.init_area()
    }

    /// `MoreSolid()`; `sb.more_solid()`.
    pub fn more_solid(&self) -> bool {
        self.my_solid_area_builder.more_area()
    }

    /// `NextSolid()`; `sb.next_solid()`.
    pub fn next_solid(&mut self) {
        self.my_solid_area_builder.next_area();
    }

    /// `InitShell()`: at the first shell of the solid, their count; `sb.init_shell()`.
    pub fn init_shell(&mut self) -> i32 {
        self.my_solid_area_builder.init_loop()
    }

    /// `MoreShell()`; `sb.more_shell()`.
    pub fn more_shell(&self) -> bool {
        self.my_solid_area_builder.more_loop()
    }

    /// `NextShell()`; `sb.next_shell()`.
    pub fn next_shell(&mut self) {
        self.my_solid_area_builder.next_loop();
    }

    /// `IsOldShell()`: the current shell is a shape loop; `sb.is_old_shell()?`.
    pub fn is_old_shell(&self) -> Result<bool, Failure> {
        Ok(self.my_solid_area_builder.loop_()?.is_shape())
    }

    /// `OldShell()`; `sb.old_shell()?`.
    pub fn old_shell(&self) -> Result<Shape, Failure> {
        Ok(self.my_solid_area_builder.loop_()?.shape())
    }

    /// `InitFace()`: at the first face of a block shell, their count;
    /// Standard_DomainError on an old shell; `sb.init_face()?`.
    pub fn init_face(&mut self) -> Result<i32, Failure> {
        let l = self.my_solid_area_builder.loop_()?.clone();
        if l.is_shape() {
            return Err(domain_error());
        }
        self.my_block_iterator = l.block_iterator();
        self.my_block_iterator.initialize();
        Ok(self.my_block_iterator.extent())
    }

    /// `MoreFace()`; `sb.more_face()`.
    pub fn more_face(&self) -> bool {
        self.my_block_iterator.more()
    }

    /// `NextFace()`; `sb.next_face()`.
    pub fn next_face(&mut self) {
        self.my_block_iterator.next();
    }

    /// `Face()`: the current face of the block; `sb.face()?`.
    pub fn face(&self) -> Result<&Shape, Failure> {
        self.my_block_builder.element_at(&self.my_block_iterator)
    }

    /// `MakeLoops(SS)` (private): the blocks of the set, then a shape loop per shape
    /// and a block loop per block.
    fn make_loops(&mut self, ss: &mut dyn TopOpeBRepBuildShapeSet) -> Result<(), Failure> {
        self.my_block_builder.make_block(ss)?;
        let ll = self.my_loop_set.change_list_of_loop();
        ll.clear();
        ss.init_shapes();
        while ss.more_shapes() {
            let shape_loop = Handle::new(TopOpeBRepBuildLoopBase::from_shape(ss.shape()?));
            ll.append(shape_loop.cast::<dyn TopOpeBRepBuildLoop>());
            ss.next_shape();
        }
        self.my_block_builder.init_block();
        while self.my_block_builder.more_block() {
            let bi = self.my_block_builder.block_iterator()?;
            let block_loop = Handle::new(TopOpeBRepBuildLoopBase::from_block(&bi));
            self.my_loop_set
                .change_list_of_loop()
                .append(block_loop.cast::<dyn TopOpeBRepBuildLoop>());
            self.my_block_builder.next_block();
        }
        Ok(())
    }
}
