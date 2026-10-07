//! Native successful growth transfers ownership without new Clone or Default bounds.
use rustycad_tkernel::ncollection::LinearVector;
#[test]
fn move_only_owned_payload_identity_survives_published_growth() {
    let owner = Box::new(17);
    let identity = std::ptr::from_ref(&*owner);
    let mut vector = LinearVector::with_capacity(1);
    vector.append(Some(owner));
    vector.reserve_with_relocation(3, Option::take);
    assert_eq!((vector.len(), vector.capacity()), (1, 3));
    assert_eq!(std::ptr::from_ref(&**vector[0].as_ref().unwrap()), identity);
}
#[test]
fn borrowed_growth_preserves_covariance_and_public_auto_traits() {
    fn traits<T: Send + Sync + Unpin + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    fn shorten<'a, 'b: 'a>(value: LinearVector<&'b str>) -> LinearVector<&'a str> {
        value
    }
    traits::<LinearVector<String>>();
    let owner = String::from("owner");
    let mut vector = LinearVector::with_capacity(1);
    vector.append(owner.as_str());
    vector.reserve_with_relocation(2, |value| *value);
    let vector = shorten(vector);
    assert_eq!(vector.as_slice(), [owner.as_str()]);
}
