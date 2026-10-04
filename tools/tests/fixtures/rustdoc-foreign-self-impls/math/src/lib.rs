pub mod gp {
    mod circle {
        pub struct Circle3(pub f64);
        pub struct GenericCircle<T>(pub T);
    }

    pub use circle::{Circle3, GenericCircle};
}
