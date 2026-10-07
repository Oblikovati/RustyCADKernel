//! The `Approx_SweepFunction` members of `BRepBlend_AppFuncRoot` and its three classes.
use super::{BRepBlendAppFuncRoot, LineKind, SharedAppFunction, clamped_tolerances};
use crate::blend::SectionPoles;
use crate::blend_func::linalg::{vector, vget};
use crate::brep_blend::BRepBlendLineHandle;
use crate::chfi_ds::lock;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkgeombase::approx::{SweepD1, SweepD2, SweepFunction};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};

impl SweepFunction for BRepBlendAppFuncRoot {
    /// `D0(Param, First, Last, Poles, Poles2d, Weigths)`: the section at the line point
    /// of `param`; false when the point is not found.
    fn d0(
        &self,
        param: f64,
        _first: f64,
        _last: f64,
        poles: &mut Array1<'_, Point3>,
        poles2d: &mut Array1<'_, Point2>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        let mut state = self.state();
        let mut func = lock(&self.func);
        if !self.search_point(&mut *func, param, &mut state)? {
            return Ok(false);
        }
        func.section(
            &state.pnt,
            SectionPoles {
                poles,
                poles2d,
                weights,
            },
        )?;
        Ok(true)
    }

    /// `D1(...)`: the section and its first derivatives at the line point of `param`.
    fn d1(
        &self,
        param: f64,
        _first: f64,
        _last: f64,
        values: SweepD1<'_, '_>,
    ) -> Result<bool, Failure> {
        let mut state = self.state();
        let mut func = lock(&self.func);
        if !self.search_point(&mut *func, param, &mut state)? {
            return Ok(false);
        }
        func.section_d1(&state.pnt, values)
    }

    /// `D2(...)`: up to the second derivatives.
    fn d2(
        &self,
        param: f64,
        _first: f64,
        _last: f64,
        values: SweepD2<'_, '_>,
    ) -> Result<bool, Failure> {
        let mut state = self.state();
        let mut func = lock(&self.func);
        if !self.search_point(&mut *func, param, &mut state)? {
            return Ok(false);
        }
        func.section_d2(&state.pnt, values)
    }

    /// `Nb2dCurves()`: the 2d poles of `GetShape`.
    fn nb_2d_curves(&self) -> Result<i32, Failure> {
        Ok(lock(&self.func).get_shape()?.3)
    }

    /// `SectionShape(NbPoles, NbKnots, Degree)`.
    fn section_shape(&self) -> Result<(i32, i32, i32), Failure> {
        let (nb_poles, nb_knots, degree, _) = lock(&self.func).get_shape()?;
        Ok((nb_poles, nb_knots, degree))
    }

    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        lock(&self.func).knots(knots)
    }

    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        lock(&self.func).mults(mults)
    }

    fn is_rational(&self) -> Result<bool, Failure> {
        lock(&self.func).is_rational()
    }

    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        lock(&self.func).nb_intervals(shape)
    }

    fn intervals(&self, t: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        lock(&self.func).intervals(t, shape)
    }

    /// `SetInterval(First, Last)`: `Func.Set(First, Last)`.
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        lock(&self.func).set_interval(first, last)
    }

    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        lock(&self.func).resolution(index, tol)
    }

    /// `GetTolerance(BoundTol, SurfTol, AngleTol, Tol3d)`: the 3d tolerances of the
    /// function, written at `1..=Tol3d.Length()`, as natively.
    fn get_tolerance(
        &self,
        bound_tol: f64,
        surf_tol: f64,
        angle_tol: f64,
        tol3d: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let len = tol3d.length();
        let (mut v3d, mut v1d) = (vector(len)?, vector(len)?);
        lock(&self.func).get_tolerance_3d(bound_tol, surf_tol, angle_tol, &mut v3d, &mut v1d)?;
        for ii in 1..=len {
            put(tol3d, ii, vget(&v3d, ii)?)?;
        }
        Ok(())
    }

    /// `SetTolerance(Tol3d, Tol2d)`: the solver tolerances.
    fn set_tolerance(&self, tol3d: f64, tol2d: f64) -> Result<(), Failure> {
        let mut state = self.state();
        state.tolerance = clamped_tolerances(&*lock(&self.func), tol3d, tol2d)?;
        let tolerance = state.tolerance.clone();
        state.solver.set_tolerance(&tolerance)
    }

    fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
        Ok(self.bary)
    }

    /// `MaximalSection()`: `GetSectionSize`.
    fn maximal_section(&self) -> Result<f64, Failure> {
        lock(&self.func).get_section_size()
    }

    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        lock(&self.func).get_minimal_weight(weights)
    }
}

/// Defines a class of `BRepBlend_AppFuncRoot` with its line kind.
macro_rules! app_func_class {
    ($(#[$doc:meta])* $class:ident, $name:literal, $kind:expr) => {
        $(#[$doc])*
        pub struct $class {
            root: BRepBlendAppFuncRoot,
        }
        impl_transient!($class, $name, BRepBlendAppFuncRoot, root, interfaces: [dyn SweepFunction]);

        impl $class {
            /// The native constructor `(Line, Func, Tol3d, Tol2d)`.
            pub fn new(line: &BRepBlendLineHandle, func: &SharedAppFunction, tol3d: f64, tol2d: f64) -> Result<Self, Failure> {
                Ok(Self { root: BRepBlendAppFuncRoot::new(line, func, (tol3d, tol2d), $kind)? })
            }
        }

        impl std::ops::Deref for $class {
            type Target = BRepBlendAppFuncRoot;
            fn deref(&self) -> &BRepBlendAppFuncRoot {
                &self.root
            }
        }

        impl SweepFunction for $class {
            fn d0(&self, param: f64, first: f64, last: f64, poles: &mut Array1<'_, Point3>, poles2d: &mut Array1<'_, Point2>, weights: &mut Array1<'_, f64>) -> Result<bool, Failure> {
                self.root.d0(param, first, last, poles, poles2d, weights)
            }
            fn d1(&self, param: f64, first: f64, last: f64, values: SweepD1<'_, '_>) -> Result<bool, Failure> {
                self.root.d1(param, first, last, values)
            }
            fn d2(&self, param: f64, first: f64, last: f64, values: SweepD2<'_, '_>) -> Result<bool, Failure> {
                self.root.d2(param, first, last, values)
            }
            fn nb_2d_curves(&self) -> Result<i32, Failure> {
                self.root.nb_2d_curves()
            }
            fn section_shape(&self) -> Result<(i32, i32, i32), Failure> {
                self.root.section_shape()
            }
            fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
                self.root.knots(knots)
            }
            fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
                self.root.mults(mults)
            }
            fn is_rational(&self) -> Result<bool, Failure> {
                self.root.is_rational()
            }
            fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
                self.root.nb_intervals(shape)
            }
            fn intervals(&self, t: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
                self.root.intervals(t, shape)
            }
            fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
                self.root.set_interval(first, last)
            }
            fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
                self.root.resolution(index, tol)
            }
            fn get_tolerance(&self, bound_tol: f64, surf_tol: f64, angle_tol: f64, tol3d: &mut Array1<'_, f64>) -> Result<(), Failure> {
                self.root.get_tolerance(bound_tol, surf_tol, angle_tol, tol3d)
            }
            fn set_tolerance(&self, tol3d: f64, tol2d: f64) -> Result<(), Failure> {
                self.root.set_tolerance(tol3d, tol2d)
            }
            fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
                self.root.barycentre_of_surf()
            }
            fn maximal_section(&self) -> Result<f64, Failure> {
                self.root.maximal_section()
            }
            fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
                self.root.get_minimal_weight(weights)
            }
        }
    };
}

app_func_class!(
    /// `BRepBlend_AppFunc`: the sweep function of a blend between two surfaces (line
    /// points on both surfaces).
    ///
    /// ```ignore
    /// let func = BRepBlendAppFunc::new(&line, &shared_function, 1e-4, 1e-5)?;
    /// ```
    BRepBlendAppFunc,
    "BRepBlend_AppFunc",
    LineKind::Surfaces
);

app_func_class!(
    /// `BRepBlend_AppFuncRst`: the sweep function of a blend between a surface and a
    /// restriction.
    ///
    /// ```ignore
    /// let func = BRepBlendAppFuncRst::new(&line, &shared_function, 1e-4, 1e-5)?;
    /// ```
    BRepBlendAppFuncRst,
    "BRepBlend_AppFuncRst",
    LineKind::SurfaceRestriction
);

app_func_class!(
    /// `BRepBlend_AppFuncRstRst`: the sweep function of a blend between two restrictions.
    ///
    /// ```ignore
    /// let func = BRepBlendAppFuncRstRst::new(&line, &shared_function, 1e-4, 1e-5)?;
    /// ```
    BRepBlendAppFuncRstRst,
    "BRepBlend_AppFuncRstRst",
    LineKind::Restrictions
);
