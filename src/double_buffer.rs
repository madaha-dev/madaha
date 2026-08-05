use std::sync::{Arc, Mutex};

/// Double buffer (scheme A: Mutex short critical section + Arc snapshot, pure safe)
///
/// - Read side: `snapshot()` does an Arc clone inside a ~20ns critical section; subsequent data usage is fully lock-free
/// - Write side: `write_with()` mutates a clone based on the **latest front** (no drift, full semantics)
/// - Commit: `swap()` exchanges front/back (called at the end of the write-side batch)
///
/// Compared with RwLock: no writer starvation, no reader/writer mutual-exclusion wait chain;
/// Compared with scheme B (arc-swap): zero dependencies, write side clones from front, avoiding per-field incremental drift.
#[derive(Debug)]
pub struct DoubleBuffered<T> {
    inner: Mutex<Inner<T>>,
}

#[derive(Debug)]
struct Inner<T> {
    front: Arc<T>,
    back: Arc<T>,
}

impl<T: Clone> DoubleBuffered<T> {
    /// Constructor: front is the initially visible value, the back workspace is initialized as a clone of front
    pub fn new(front: T) -> Self {
        let front = Arc::new(front);
        let back = Arc::new((*front).clone());
        Self {
            inner: Mutex::new(Inner { front, back }),
        }
    }

    /// Read-side snapshot: Arc clone in a short critical section, usable lock-free after return
    pub fn snapshot(&self) -> Arc<T> {
        self.inner.lock().expect("DoubleBuffered lock poisoned").front.clone()
    }

    /// Write-side modification: accumulated into back within a batch (starting point = clone of front after the last swap)
    pub fn write_with(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self.inner.lock().expect("DoubleBuffered lock poisoned");
        f(Arc::make_mut(&mut guard.back));
    }

    /// Commit: exchange front/back, reset back to a clone of the latest front (starting point for the next write)
    pub fn swap(&self) {
        let mut guard = self.inner.lock().expect("DoubleBuffered lock poisoned");
        let Inner { front, back } = &mut *guard;
        std::mem::swap(front, back);
        *back = Arc::new((**front).clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_reflects_swap() {
        let db = DoubleBuffered::new(vec![1, 2]);
        assert_eq!(*db.snapshot(), vec![1, 2]);

        db.write_with(|v| v.push(3));
        assert_eq!(*db.snapshot(), vec![1, 2]); // not visible until committed
        db.swap();
        assert_eq!(*db.snapshot(), vec![1, 2, 3]); // visible after commit
    }

    #[test]
    fn consecutive_writes_coalesce() {
        let db = DoubleBuffered::new(0i32);
        db.write_with(|v| *v += 1);
        db.write_with(|v| *v += 1); // accumulated within the batch
        assert_eq!(*db.snapshot(), 0);
        db.swap();
        assert_eq!(*db.snapshot(), 2);
    }

    #[test]
    fn write_after_swap_isolation() {
        let db = DoubleBuffered::new(10i32);
        db.swap();
        assert_eq!(*db.snapshot(), 10); // back reset to a clone of front
        db.write_with(|v| *v = 99);
        assert_eq!(*db.snapshot(), 10); // not committed
        db.swap();
        assert_eq!(*db.snapshot(), 99);
    }

    #[test]
    fn snapshot_arc_ownership_outlives_swap() {
        let db = DoubleBuffered::new(String::from("a"));
        let snap = db.snapshot(); // holds an Arc
        db.write_with(|v| v.push('x'));
        db.swap();
        // the old snapshot is still readable (guaranteed by Arc ownership); the new snapshot has the new value
        assert_eq!(&*snap, "a");
        assert_eq!(&*db.snapshot(), "ax");
    }
}
