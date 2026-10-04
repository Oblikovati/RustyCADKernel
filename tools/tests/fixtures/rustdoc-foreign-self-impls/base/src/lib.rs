pub mod gce {
    pub struct MakeCirc(pub f64);
    pub struct MakeEllipse(pub f64);
}

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
