//! Copying an owning range preserves the original host's position.
use rustycad_tkernel::ncollection::forward_range::*;
#[derive(Clone)]
struct Host {
    position: usize,
}
struct Access;
impl ReadAccessor<Host> for Access {
    type Item = usize;
    type Value = usize;
    fn read(host: &Host) -> usize {
        host.position
    }
    fn snapshot(value: usize) -> usize {
        value
    }
}
impl ForwardHost for Host {
    type Accessors = AccessorSet<Present<Access>, Missing, Missing>;
    fn more(&self) -> bool {
        self.position < 2
    }
    fn advance(&mut self) {
        self.position += 1;
    }
}
#[test]
fn copied_host_and_exhausted_range_are_independent_and_safe() {
    let host = Host { position: 0 };
    let mut range = ForwardRange::from_copy(&host);
    assert_eq!(range.next(), Some(0));
    assert_eq!(host.position, 0);
    let sentinel = range.end();
    let mut iterator = range.iter();
    assert_eq!(*iterator.post_increment().unwrap(), 1);
    assert!(iterator == sentinel);
    assert!(iterator.current().is_none());
    assert!(iterator.post_increment().is_none());
    iterator.advance();
    assert_eq!(iterator.next(), None);
}
