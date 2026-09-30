// Rust translation of the conic-plane Perform overloads of IntAna_IntConicQuad.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_IntConicQuad::Perform(conic, gp_Pln, ...): the plane special cases.
use super::IntConicQuad;
use crate::int_ana::{QuadQuadGeo, Quadric, ResultType};
use crate::int_ana2d::AnaIntersection;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis2d, Axis3, Circle2, Circle3, Direction2, Ellipse3, Hyperbola3, Line2, Line3, Parabola3,
    Plane3, Point2, Point3, StandardDirection2, Vector2, Vector3,
};

impl IntConicQuad {
    /// Perform(gp_Lin, gp_Pln, Tolang, Tol, Len): parallel when the cosine of the
    /// line with the plane normal is below `tolang`, unless the line (of length
    /// `len`, from its projection on the plane) leaves the plane by more than `tol`;
    /// a parallel line is in the plane when its distance is below `tolang`.
    /// Otherwise one point, of parameter `-Dis / Direc`.
    /// `inter.perform_line_plane(line, plane, 1e-12, 0.0, 0.0)?`.
    pub fn perform_line_plane(
        &mut self,
        line: Line3,
        plane: Plane3,
        tolang: f64,
        tol: f64,
        len: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        let [a, b, c, d] = plane.coefficients()?;
        let origin: Point3 = line.location();
        let [al, bl, cl] = *line.direction().coordinates();
        let direc: f64 = a * al + b * bl + c * cl;
        let dis: f64 = a * origin.x() + b * origin.y() + c * origin.z() + d;
        self.parallel = false;
        if direc.abs() < tolang {
            self.parallel = true;
            if len != 0.0 && direc != 0.0 {
                // The distance from the far end of the projected segment to the plane.
                let p1 = Point3::new(
                    origin.x() - dis * a,
                    origin.y() - dis * b,
                    origin.z() - dis * c,
                );
                let p2 = Point3::new(p1.x() + len * al, p1.y() + len * bl, p1.z() + len * cl);
                if plane.distance(p2) > tol {
                    self.parallel = false;
                }
            }
        }
        if self.parallel {
            self.inquadric = dis.abs() < tolang;
        } else {
            self.inquadric = false;
            self.nbpts = 1;
            self.paramonc[0] = -dis / direc;
            let t: f64 = self.paramonc[0];
            self.pnts[0] = Point3::new(
                origin.x() + t * al,
                origin.y() + t * bl,
                origin.z() + t * cl,
            );
        }
        self.done = true;
        Ok(())
    }

    /// Perform(gp_Circ, gp_Pln, Tolang, Tol): intersects the plane of the circle
    /// with the plane (IntAna_QuadQuadGeo), then the circle with the line solution
    /// in the circle plane (IntAna2d). Parallel planes are parallel, the circle in
    /// the plane when `distance + Radius Tolang < Tol`; identical planes put the
    /// circle in the plane. `inter.perform_circle_plane(circle, plane, 1e-12, 1e-7)?`.
    pub fn perform_circle_plane(
        &mut self,
        circle: Circle3,
        plane: Plane3,
        tolang: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        let conic_plane = Plane3::from_position(Axis3::from_axis2(circle.position()));
        let planes = QuadQuadGeo::planes(conic_plane, plane, tolang, tol)?;
        if !planes.is_done() {
            return Ok(());
        }
        match planes.type_inter()? {
            ResultType::Empty => {
                self.parallel = true;
                let distmax: f64 = plane.distance(circle.location()) + circle.radius() * tolang;
                self.inquadric = distmax < tol;
                self.done = true;
                Ok(())
            }
            ResultType::Same => {
                self.inquadric = true;
                self.done = true;
                Ok(())
            }
            _ => self.circle_on_line(circle, conic_plane, planes.line(1)?),
        }
    }

    /// The points of `circle` on the line `solution` of its plane `conic_plane`,
    /// computed in the circle plane by IntAna2d; the parameter is the angle on the
    /// circle.
    fn circle_on_line(
        &mut self,
        circle: Circle3,
        conic_plane: Plane3,
        solution: Line3,
    ) -> Result<(), Failure> {
        self.inquadric = false;
        self.parallel = false;
        let origin: Point3 = conic_plane.location();
        let axex = Vector3::from(conic_plane.position().x_direction());
        let axey = Vector3::from(conic_plane.position().y_direction());
        let v0 = Vector3::between(origin, solution.location());
        let direction = Vector3::from(solution.direction());
        let line2d = Line2::new(
            Point2::new(axex.dot(v0), axey.dot(v0)),
            Direction2::from_vector(Vector2::new(axex.dot(direction), axey.dot(direction)))?,
        );
        let axis2d = Axis2d::new(
            Point2::new(0.0, 0.0),
            Direction2::from_standard(StandardDirection2::X),
        );
        let circle2d = Circle2::from_x_axis(axis2d, circle.radius(), true)?;
        let int2d = AnaIntersection::line_circle(line2d, circle2d)?;
        if !int2d.is_done() {
            return Ok(());
        }
        self.nbpts = int2d.nb_points()?;
        for i in 1..=self.nbpts {
            let result: Point2 = int2d.point(i)?.value();
            let (x, y) = (result.x(), result.y());
            self.pnts[(i - 1) as usize] = Point3::new(
                origin.x() + x * axex.x() + y * axey.x(),
                origin.y() + x * axex.y() + y * axey.y(),
                origin.z() + x * axex.z() + y * axey.z(),
            );
            self.paramonc[(i - 1) as usize] = int2d.point(i)?.param_on_second()?;
        }
        self.done = true;
        Ok(())
    }

    /// Perform(gp_Elips, gp_Pln, Tolang, Tol): the ellipse-quadric intersection
    /// with the plane as a quadric; the tolerances are unused, as natively.
    /// `inter.perform_ellipse_plane(ellipse, plane, 1e-12, 1e-7)?`.
    pub fn perform_ellipse_plane(
        &mut self,
        ellipse: Ellipse3,
        plane: Plane3,
        _tolang: f64,
        _tol: f64,
    ) -> Result<(), Failure> {
        self.perform_ellipse_quadric(ellipse, &Quadric::from_plane(plane)?)
    }

    /// Perform(gp_Parab, gp_Pln, Tolang): the parabola-quadric intersection with
    /// the plane as a quadric; the tolerance is unused, as natively.
    /// `inter.perform_parabola_plane(parabola, plane, 1e-12)?`.
    pub fn perform_parabola_plane(
        &mut self,
        parabola: Parabola3,
        plane: Plane3,
        _tolang: f64,
    ) -> Result<(), Failure> {
        self.perform_parabola_quadric(parabola, &Quadric::from_plane(plane)?)
    }

    /// Perform(gp_Hypr, gp_Pln, Tolang): the hyperbola-quadric intersection with
    /// the plane as a quadric; the tolerance is unused, as natively.
    /// `inter.perform_hyperbola_plane(hyperbola, plane, 1e-12)?`.
    pub fn perform_hyperbola_plane(
        &mut self,
        hyperbola: Hyperbola3,
        plane: Plane3,
        _tolang: f64,
    ) -> Result<(), Failure> {
        self.perform_hyperbola_quadric(hyperbola, &Quadric::from_plane(plane)?)
    }
}
