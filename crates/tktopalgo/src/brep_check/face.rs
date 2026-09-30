// Created on: 1995-12-15
// Created by: Jacques GOUSSARD
// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Face.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Face`: the checker of a face (a surface; wires that do not intersect,
//! nest correctly and are oriented). `IntersectWires` is in `face_wires.rs`,
//! `ClassifyWires` and `OrientationOfWires` in `face_classify.rs`.
use super::explore::holds_same;
use super::result::{BRepCheckResult, ResultBase, SharedStatusList, add_status, close_status_list};
use super::status::BRepCheckStatus;
use rustycad_tkbrep::brep::access::face_tshape;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `myMapImb`: each wire (reversed when it bounds a hole) and the wires inside it.
pub(super) type WireNesting = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The own fields of `BRepCheck_Face`.
pub(super) struct FaceFields {
    /// `myIntdone`, `myIntres`: `IntersectWires` has run, and its result.
    pub(super) intdone: bool,
    pub(super) intres: BRepCheckStatus,
    /// `myImbdone`, `myImbres`: `ClassifyWires` has run, and its result.
    pub(super) imbdone: bool,
    pub(super) imbres: BRepCheckStatus,
    /// `myOridone`, `myOrires`: `OrientationOfWires` has run, and its result.
    pub(super) oridone: bool,
    pub(super) orires: BRepCheckStatus,
    pub(super) map_imb: WireNesting,
    /// `myGctrl`.
    pub(super) gctrl: bool,
}

/// `BRepCheck_Face`: checks that a face has a surface and, through the members the
/// analyzer calls, that its wires do not intersect (`IntersectWires`), nest with one
/// outer wire (`ClassifyWires`) and are oriented accordingly (`OrientationOfWires`).
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tktopalgo::brep_check::{BRepCheckFace, BRepCheckResult, BRepCheckStatus};
/// let mut f = Face::default();
/// BRepBuilder::default().make_face(&mut f);
/// let checker = BRepCheckFace::new(&f).unwrap();
/// assert_eq!(checker.status().unwrap().first(), Some(&BRepCheckStatus::NoSurface));
/// ```
pub struct BRepCheckFace {
    transient: StandardTransient,
    base: ResultBase,
    pub(super) fields: SharedState<FaceFields>,
}
impl_transient!(BRepCheckFace, "BRepCheck_Face", dyn BRepCheckResult, transient,
    interfaces: [dyn BRepCheckResult]);

impl BRepCheckFace {
    /// `BRepCheck_Face(F)`: `Init(F)`, then no wire check done and the geometric
    /// controls on; `BRepCheckFace::new(&face)?`.
    pub fn new(face: &Face) -> Result<Self, Failure> {
        let checker = Self {
            transient: StandardTransient::default(),
            base: ResultBase::new(),
            fields: SharedState::new(FaceFields {
                intdone: false,
                intres: BRepCheckStatus::NoError,
                imbdone: false,
                imbres: BRepCheckStatus::NoError,
                oridone: false,
                orires: BRepCheckStatus::NoError,
                map_imb: WireNesting::default(),
                gctrl: true,
            }),
        };
        checker.init(face)?;
        let mut fields = checker.fields.write();
        fields.intdone = false;
        fields.imbdone = false;
        fields.oridone = false;
        fields.gctrl = true;
        drop(fields);
        Ok(checker)
    }

    /// `SetUnorientable()`: adds `UnorientableShape` to the statuses of the face;
    /// `checker.set_unorientable()?`.
    pub fn set_unorientable(&self) -> Result<(), Failure> {
        self.set_status(BRepCheckStatus::UnorientableShape)
    }

    /// `SetStatus(theStatus)`; `checker.set_status(BRepCheckStatus::InvalidToleranceValue)?`.
    pub fn set_status(&self, status: BRepCheckStatus) -> Result<(), Failure> {
        add_status(&self.base.own_list()?, status);
        Ok(())
    }

    /// `IsUnorientable()`: the result of `OrientationOfWires` when it has run, else
    /// whether `UnorientableShape` was recorded; `checker.is_unorientable()?`.
    pub fn is_unorientable(&self) -> Result<bool, Failure> {
        {
            let fields = self.fields.read();
            if fields.oridone {
                return Ok(fields.orires != BRepCheckStatus::NoError);
            }
        }
        let list = self.base.own_list()?;
        let found = list
            .read()
            .iter()
            .any(|s| *s == BRepCheckStatus::UnorientableShape);
        Ok(found)
    }

    /// `GeometricControls(B)`: turning the controls on makes the wire checks run again;
    /// `checker.set_geometric_controls(false)`.
    pub fn set_geometric_controls(&self, controls: bool) {
        let mut fields = self.fields.write();
        if fields.gctrl != controls {
            if controls {
                fields.intdone = false;
                fields.imbdone = false;
                fields.oridone = false;
            }
            fields.gctrl = controls;
        }
    }

    /// `GeometricControls()`; `checker.geometric_controls()`.
    pub fn geometric_controls(&self) -> bool {
        self.fields.read().gctrl
    }

    /// The face being checked.
    pub(super) fn face(&self) -> Face {
        Face::unchecked(self.base.shape())
    }

    /// `myMap(myShape)`.
    pub(super) fn own_list(&self) -> Result<SharedStatusList, Failure> {
        self.base.own_list()
    }
}

impl BRepCheckResult for BRepCheckFace {
    fn result_base(&self) -> &ResultBase {
        &self.base
    }

    /// `InContext(S)`: `SubshapeNotInShape` when the face is not in `context`.
    fn in_context(&self, context: &Shape) -> Result<(), Failure> {
        let Some(list) = self.base.bind_context_list(context) else {
            return Ok(());
        };
        if !holds_same(context, ShapeEnum::Face, &self.base.shape())? {
            add_status(&list, BRepCheckStatus::SubshapeNotInShape);
            return Ok(());
        }
        close_status_list(&list);
        Ok(())
    }

    /// `Minimum()`: `NoSurface` for a face without surface.
    fn minimum(&self) -> Result<(), Failure> {
        if self.base.is_minimum() {
            return Ok(());
        }
        let list = self.base.bind_own_list();
        let shape = self.base.shape();
        if face_tshape(&shape, "BRepCheck_Face::Minimum")?
            .surface()
            .is_null()
        {
            add_status(&list, BRepCheckStatus::NoSurface);
        }
        close_status_list(&list);
        self.base.set_minimum(true);
        Ok(())
    }

    /// `Blind()`: nothing more than the minimum.
    fn blind(&self) {
        self.base.set_blind(true);
    }
}
