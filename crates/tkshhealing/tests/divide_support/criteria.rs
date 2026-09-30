//! Actual derived splitting criteria exercise native virtual dispatch.
use rustycad_tkernel::{
    impl_transient,
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkshhealing::shape_upgrade::{
    ShapeUpgradeSplitCurve2d, ShapeUpgradeSplitCurve3d, SplitCurve, SplitCurveData,
};
macro_rules! midpoint_criterion {
    ($name:ident,$base:ty,$native:literal) => {
        #[derive(Default)]
        pub struct $name {parent:$base}
        impl_transient!($name,$native,$base,parent,interfaces:[dyn SplitCurve]);
        impl SplitCurve for $name {
            fn split_curve_data(&self)->&SplitCurveData {self.parent.split_curve_data()}
            fn compute(&self)->Result<(),Failure> {
                self.parent.compute()?;
                let values=self.split_values();
                let (first,last)=values.sequence(|s|(*s.value(1).unwrap(),*s.value(s.len()).unwrap()));
                let middle=Handle::new(HSequence::default());
                middle.append((first+last)*0.5);
                self.set_split_values(&middle)?;
                self.split_curve_data().with_mut(|f|f.status=1);
                Ok(())
            }
            fn build(&self,segment:bool)->Result<(),Failure>{self.parent.build(segment)}
        }
    };
}
midpoint_criterion!(
    Midpoint2d,
    ShapeUpgradeSplitCurve2d,
    "Test_EdgeDivideMidpoint2d"
);
midpoint_criterion!(
    Midpoint3d,
    ShapeUpgradeSplitCurve3d,
    "Test_EdgeDivideMidpoint3d"
);
