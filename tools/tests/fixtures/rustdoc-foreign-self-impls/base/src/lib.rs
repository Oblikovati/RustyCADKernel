pub mod gce {
    mod circ {
        pub struct MakeCirc(pub f64);
    }
    mod ellipse {
        pub struct MakeEllipse(pub f64);
    }
    pub use circ::MakeCirc;
    pub use ellipse::MakeEllipse;
}

mod private_traits {
    pub trait LocalBuild<T> {
        fn build_from(value: T) -> Self;
    }
}

pub use private_traits::LocalBuild as PublicBuild;

impl TryFrom<&gce::MakeCirc> for rustycad_tkmath::gp::Circle3 {
    type Error = ();
    fn try_from(maker: &gce::MakeCirc) -> Result<Self, Self::Error> {
        Ok(rustycad_tkmath::gp::Circle3(maker.0))
    }
}

impl TryFrom<gce::MakeCirc> for rustycad_tkmath::gp::Circle3 {
    type Error = ();
    fn try_from(maker: gce::MakeCirc) -> Result<Self, Self::Error> {
        Ok(rustycad_tkmath::gp::Circle3(maker.0))
    }
}

impl TryFrom<&gce::MakeEllipse> for rustycad_tkmath::gp::Circle3 {
    type Error = ();
    fn try_from(maker: &gce::MakeEllipse) -> Result<Self, Self::Error> {
        Ok(rustycad_tkmath::gp::Circle3(maker.0))
    }
}

impl private_traits::LocalBuild<&gce::MakeCirc> for rustycad_tkmath::gp::Circle3 {
    fn build_from(value: &gce::MakeCirc) -> Self {
        rustycad_tkmath::gp::Circle3(value.0)
    }
}

impl TryFrom<&gce::MakeCirc> for rustycad_tkmath::gp::GenericCircle<u32> {
    type Error = ();
    fn try_from(maker: &gce::MakeCirc) -> Result<Self, Self::Error> {
        Ok(rustycad_tkmath::gp::GenericCircle(maker.0 as u32))
    }
}

impl TryFrom<&gce::MakeCirc> for rustycad_tkmath::gp::GenericCircle<f64> {
    type Error = ();
    fn try_from(maker: &gce::MakeCirc) -> Result<Self, Self::Error> {
        Ok(rustycad_tkmath::gp::GenericCircle(maker.0))
    }
}
