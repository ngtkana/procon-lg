//! # procon-lg
//!
//! A procedural macro library for debugging recursive functions in competitive programming

use std::cell::RefCell;

// Thread-local depth counter for global depth management
thread_local! {
    static LG_DEPTH: RefCell<usize> = const { RefCell::new(0) };
}

/// RAII guard for managing recursion depth
/// Automatically increments depth on creation and decrements on drop
pub struct DepthGuard {
    depth: usize,
}

impl DepthGuard {
    /// Create a new depth guard, incrementing the current depth
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current depth level
    pub fn current_depth(&self) -> usize {
        self.depth
    }
}

impl Default for DepthGuard {
    fn default() -> Self {
        LG_DEPTH.with(|depth| {
            let mut d = depth.borrow_mut();
            *d += 1;
            DepthGuard { depth: *d - 1 }
        })
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        LG_DEPTH.with(|depth| {
            let mut d = depth.borrow_mut();
            *d -= 1;
        });
    }
}

// Re-export the procedural macro
pub use procon_lg_macros::lg_recur;
