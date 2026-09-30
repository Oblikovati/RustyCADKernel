// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_ContAna.cxx (the cone) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The contour generatrices of a cone.
use super::cont_ana::{ContapContAna, trigonometric_roots};
use super::geometry::point_of;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Cone3, Direction3, Point3, Xyz};

/// The frame of a cone and the equation `Coefcos cos(t) + Coefsin sin(t) = Coefcst` of
/// its contour generatrices.
struct ConeGeneratrices {
    points: [Point3; 2],
    dirs: [Direction3; 2],
    prm: f64,
}

struct ConeEquation {
    coefcos: f64,
    coefsin: f64,
    norm1: f64,
    norm2: f64,
    xdir: Xyz,
    ydir: Xyz,
}

impl ConeEquation {
    fn new(c: &Cone3, coefcos: f64, coefsin: f64) -> Self {
        let norm1 = coefcos * coefcos + coefsin * coefsin;
        let position = c.position();
        Self {
            coefcos,
            coefsin,
            norm1,
            norm2: norm1.sqrt(),
            xdir: position.x_direction().xyz(),
            ydir: position.y_direction().xyz(),
        }
    }

    /// The two generatrices from the apex for the constant `coefcst`, with the axial
    /// weight `zweight` on `zdir`: `(points, directions)`, or `None` when the equation
    /// has no root (`|Coefcst| < norm2` fails, NaN included). Returns also `prm`.
    fn generatrices(
        &self,
        apex: Point3,
        coefcst: f64,
        zweight: f64,
        zdir: Xyz,
    ) -> Result<Option<ConeGeneratrices>, Failure> {
        let has_roots = coefcst.abs() < self.norm2;
        if !has_roots {
            return Ok(None);
        }
        let prm = (self.norm1 - coefcst * coefcst).sqrt();
        let roots = trigonometric_roots(self.coefcos, self.coefsin, coefcst, prm, self.norm1);
        Ok(Some(
            self.locate_generatrices(apex, zweight, zdir, roots, prm)?,
        ))
    }
    fn locate_generatrices(
        &self,
        apex: Point3,
        zweight: f64,
        zdir: Xyz,
        roots: [(f64, f64); 2],
        prm: f64,
    ) -> Result<ConeGeneratrices, Failure> {
        let mut points = [apex; 2];
        let mut dirs = [Direction3::default(); 2];
        for (k, (cost, sint)) in roots.into_iter().enumerate() {
            let mut dirxyz = Xyz::default();
            dirxyz.set_linear_form_three([cost, sint, zweight], [self.xdir, self.ydir, zdir]);
            dirs[k] = Direction3::from_xyz(dirxyz)?;
            points[k] = point_of(apex.xyz() + dirxyz);
        }
        Ok(ConeGeneratrices { points, dirs, prm })
    }
}

impl ContapContAna {
    /// Perform(gp_Cone, gp_Dir): the two generatrices whose normal is normal to `d`;
    /// none when `d` is inside the cone of the axis; `contour.perform_cone(&c, d)?`.
    pub fn perform_cone(&mut self, c: &Cone3, d: Direction3) -> Result<(), Failure> {
        self.done = false;
        let tgtalpha = c.semi_angle().tan();
        let position = c.position();
        let equation = ConeEquation::new(
            c,
            d.dot(position.x_direction()),
            d.dot(position.y_direction()),
        );
        let coefcst = d.dot(c.axis().direction()) * tgtalpha;
        let zdir = c.axis().direction().xyz();
        self.store_pair(equation.generatrices(c.apex(), coefcst, 1. / tgtalpha, zdir)?);
        self.done = true;
        Ok(())
    }

    /// Perform(gp_Cone, gp_Dir, Ang): the generatrices of draft angle `ang`: up to two on
    /// each side of the axis (four in all); when only the second side has roots its two
    /// lines are the first two; `contour.perform_cone_draft(&c, d, 0.1)?`.
    pub fn perform_cone_draft(
        &mut self,
        c: &Cone3,
        d: Direction3,
        angle: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        self.nb_sol = 0;
        let draft = ConeDraftEquation::new(c, d, angle)?;
        self.store_draft_pair(0, draft.solve(c, false)?);
        self.store_draft_pair(2, draft.solve(c, true)?);
        self.done = true;
        Ok(())
    }

    /// Perform(gp_Cone, gp_Pnt Eye): the two generatrices tangent to the planes
    /// through `eye` and the apex; `contour.perform_cone_eye(&c, eye)?`.
    pub fn perform_cone_eye(&mut self, c: &Cone3, eye: Point3) -> Result<(), Failure> {
        self.done = false;
        let tgtalpha = c.semi_angle().tan();
        let mut apexeye = eye.xyz();
        apexeye -= c.apex().xyz();
        let position = c.position();
        let coefcos = apexeye.dot(position.x_direction().xyz());
        let coefsin = apexeye.dot(position.y_direction().xyz());
        let equation = ConeEquation::new(c, coefcos, coefsin);
        let coefcst = apexeye.dot(c.axis().direction().xyz()) * tgtalpha;
        let zdir = c.axis().direction().xyz();
        self.store_pair(equation.generatrices(c.apex(), coefcst, 1. / tgtalpha, zdir)?);
        self.done = true;
        Ok(())
    }

    /// Two generatrices, or none.
    fn store_pair(&mut self, pair: Option<ConeGeneratrices>) {
        let Some(ConeGeneratrices { points, dirs, prm }) = pair else {
            self.nb_sol = 0;
            return;
        };
        self.typ_l = CurveType::Line;
        self.nb_sol = 2;
        (self.pt[0], self.pt[1], self.dir[0], self.dir[1], self.prm) =
            (points[0], points[1], dirs[0], dirs[1], prm);
    }
    fn store_draft_pair(&mut self, offset: usize, pair: Option<ConeGeneratrices>) {
        let Some(ConeGeneratrices { points, dirs, prm }) = pair else {
            return;
        };
        self.typ_l = CurveType::Line;
        self.nb_sol += 2;
        self.pt[offset..offset + 2].copy_from_slice(&points);
        self.dir[offset..offset + 2].copy_from_slice(&dirs);
        self.prm = prm;
        if offset == 2 && self.nb_sol == 2 {
            self.pt[..2].copy_from_slice(&points);
            self.dir[..2].copy_from_slice(&dirs);
        }
    }
}
struct ConeDraftEquation {
    equation: ConeEquation,
    cosa: f64,
    sina: f64,
    coefcst1: f64,
    coefnz: f64,
    zdir: Xyz,
}
impl ConeDraftEquation {
    fn new(c: &Cone3, d: Direction3, angle: f64) -> Result<Self, Failure> {
        let (cosa, sina) = (c.semi_angle().cos(), c.semi_angle().sin());
        let position = c.position();
        let equation = ConeEquation::new(
            c,
            d.dot(position.x_direction()),
            d.dot(position.y_direction()),
        );
        let mut zdir = c.axis().direction().xyz();
        if !c.direct()? {
            zdir.reverse();
        }
        Ok(Self {
            equation,
            cosa,
            sina,
            coefcst1: (std::f64::consts::PI * 0.5 + angle).cos(),
            coefnz: d.dot(c.axis().direction()) * sina,
            zdir,
        })
    }
    fn solve(&self, c: &Cone3, second: bool) -> Result<Option<ConeGeneratrices>, Failure> {
        let (constant, weight) = if second {
            (
                (self.coefcst1 - self.coefnz) / self.cosa,
                -self.cosa / self.sina,
            )
        } else {
            (
                (self.coefcst1 + self.coefnz) / self.cosa,
                self.cosa / self.sina,
            )
        };
        self.equation
            .generatrices(c.apex(), constant, weight, self.zdir)
    }
}
