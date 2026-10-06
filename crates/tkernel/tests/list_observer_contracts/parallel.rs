use rustycad_tkernel::ncollection::{List, ListObserver};
use std::sync::{Arc, Mutex};
type ObserverReading = (usize, bool, Option<usize>, Option<usize>);
struct ParallelDropProbe {
    observer: ListObserver,
    reading: Arc<Mutex<Option<ObserverReading>>>,
}
impl Drop for ParallelDropProbe {
    fn drop(&mut self) {
        let observer = self.observer.clone();
        let reading = std::thread::spawn(move || {
            (
                observer.size(),
                observer.is_empty(),
                observer.first_node(),
                observer.last_node(),
            )
        })
        .join()
        .unwrap();
        *self.reading.lock().unwrap() = Some(reading);
    }
}
#[test]
fn destructor_can_join_a_cross_thread_canonical_metadata_reader() {
    let mut list = List::default();
    let reading = Arc::new(Mutex::new(None));
    list.append(ParallelDropProbe {
        observer: list.observer(),
        reading: reading.clone(),
    });
    let head = list.observer().first_node();
    list.remove_first().unwrap();
    assert_eq!(*reading.lock().unwrap(), Some((1, true, None, head)));
    assert_eq!(list.observer().size(), 0);
}
