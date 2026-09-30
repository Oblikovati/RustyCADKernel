// Rust translation of ModelingData/TKG3d/Geom/Geom_Axis2Placement.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Axis2Placement`: a right-handed frame stored as a main axis plus X and Y
//! directions.
use super::axis_placement::GeomAxisPlacement;
use super::geometry::GeomGeometry;
use super::protected::ProtectedAccess;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Transform3};

#[derive(Clone, Copy)]
struct FrameState {
    axis: Axis1,
    vxdir: Direction3,
    vydir: Direction3,
}

/// A coordinate frame; `Handle::new(GeomAxis2Placement::new(frame))`.
pub struct GeomAxis2Placement {
    parent: StandardTransient,
    state: SharedState<FrameState>,
}
impl_transient!(
    GeomAxis2Placement,
    "Geom_Axis2Placement",
    dyn GeomAxisPlacement,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomAxisPlacement]
);
impl Clone for GeomAxis2Placement {
    // The native Copy uses the private constructor from the four stored values.
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

impl GeomAxis2Placement {
    fn stored(state: FrameState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Placement from a gp frame (`Geom_Axis2Placement(gp_Ax2)`); `GeomAxis2Placement::new(frame)`.
    pub fn new(frame: Axis2) -> Self {
        Self::stored(FrameState {
            axis: frame.axis(),
            vxdir: frame.x_direction(),
            vydir: frame.y_direction(),
        })
    }
    /// Frame from origin, main direction `n` and an X direction projected onto the
    /// plane normal to `n`; Standard_ConstructionError when `vx` is parallel to `n`
    /// (`Geom_Axis2Placement(P, N, Vx)`); `GeomAxis2Placement::from_directions(p, n, vx)?`.
    pub fn from_directions(
        location: Point3,
        n: Direction3,
        vx: Direction3,
    ) -> Result<Self, Failure> {
        let vxdir: Direction3 = n.cross_crossed(vx, n)?;
        let vydir: Direction3 = n.crossed(vxdir)?;
        Ok(Self::stored(FrameState {
            axis: Axis1::new(location, n),
            vxdir,
            vydir,
        }))
    }
    /// Replace the whole frame (`SetAx2`); `placement.set_ax2(frame)`.
    pub fn set_ax2(&self, frame: Axis2) {
        *self.state.write() = FrameState {
            axis: frame.axis(),
            vxdir: frame.x_direction(),
            vydir: frame.y_direction(),
        };
    }
    /// Recompute X from `vx` projected onto the plane normal to the main direction,
    /// then Y (`SetXDirection`); `placement.set_x_direction(vx)?`.
    pub fn set_x_direction(&self, vx: Direction3) -> Result<(), Failure> {
        let mut state = self.state.write();
        let main: Direction3 = state.axis.direction();
        state.vxdir = main.cross_crossed(vx, main)?;
        state.vydir = main.crossed(state.vxdir)?;
        Ok(())
    }
    /// Recompute X as `vy ^ main`, then Y (`SetYDirection`); `placement.set_y_direction(vy)?`.
    pub fn set_y_direction(&self, vy: Direction3) -> Result<(), Failure> {
        let mut state = self.state.write();
        let main: Direction3 = state.axis.direction();
        state.vxdir = vy.crossed(main)?;
        state.vydir = main.crossed(state.vxdir)?;
        Ok(())
    }
    /// The gp frame built from origin, main and X directions (`Ax2`); `placement.ax2()?`.
    pub fn ax2(&self) -> Result<Axis2, Failure> {
        let state: FrameState = *self.state.read();
        Axis2::with_x_direction(state.axis.location(), state.axis.direction(), state.vxdir)
    }
    /// The X direction (`XDirection`); `placement.x_direction()`.
    pub fn x_direction(&self) -> Direction3 {
        self.state.read().vxdir
    }
    /// The Y direction (`YDirection`); `placement.y_direction()`.
    pub fn y_direction(&self) -> Direction3 {
        self.state.read().vydir
    }
}
impl GeomGeometry for GeomAxis2Placement {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        let location: Point3 = state.axis.location().transformed(transform);
        state.axis.set_location(location);
        state.vxdir.transform(transform);
        state.vydir.transform(transform);
        let main: Direction3 = state.vxdir.crossed(state.vydir)?;
        state.axis.set_direction(main);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
}
impl GeomAxisPlacement for GeomAxis2Placement {
    fn axis(&self) -> Axis1 {
        self.state.read().axis
    }
    fn change_axis(&self, _access: ProtectedAccess, change: &mut dyn FnMut(&mut Axis1)) {
        change(&mut self.state.write().axis);
    }
    /// Replace the main direction and re-project X and Y; like the native member, the
    /// new main direction stays stored when the old X direction is parallel to it.
    fn set_direction(&self, direction: Direction3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.axis.set_direction(direction);
        state.vxdir = direction.cross_crossed(state.vxdir, direction)?;
        state.vydir = direction.crossed(state.vxdir)?;
        Ok(())
    }
}
