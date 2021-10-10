use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityID(pub i32);
pub static IDS: Lazy<Arc<Mutex<Vec<i32>>>> = Lazy::new(|| {
    let mut vec = vec![0; 2048];
    for i in 0..2048 {
        vec[i] = i as i32;
    }
    vec.reverse();
    Arc::new(Mutex::new(vec))
});
impl EntityID {
    pub fn new() -> Self {
        Self(IDS.lock().unwrap().pop().unwrap())
/*         static NEXT: AtomicI8 = AtomicI8::new(0);
        Self(NEXT.fetch_add(1, Ordering::SeqCst)) */
    }
}
