pub use lazy_static;
use std::sync::{Arc, Mutex};
pub use serde::{self,Deserialize, Serialize};
pub use serde_json;
pub use local_ip_address::local_broadcast_ip;
pub use goon_proc_macros::{global, goon_init, declare_global, goon_update};
pub struct Global<T> {
    inner: Arc<Mutex<T>>,
}
impl<T> Global<T> {
    pub fn new(initial_value: T) -> Self {
        Global {
            inner: Arc::new(Mutex::new(initial_value)),
        }
    }

    pub fn clone(&self) -> Self {
        Global {
            inner: self.inner.clone(),
        }
    }

    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, T>, std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> {
        self.inner.lock()
    }

    pub fn try_lock(&self) -> Result<std::sync::MutexGuard<'_, T>, std::sync::TryLockError<std::sync::MutexGuard<'_, T>>>{
        self.inner.try_lock()
    }
}


#[macro_export]
macro_rules! lock_globals {
    ( $node:ident; $mutex:ident ; $code:block ) => {
        if let Ok(mut $mutex) = $mutex.lock() {
            let ret = {$code};

            $node.update(stringify!($mutex).to_string(), &*$mutex);
            Some(ret)
        } else {
            None
        }
    };
    ( |$($tail:ident),* | =>  $code:block ) => {
        if let Ok(node) = NODE.clone().lock() {
            lock_globals!(node; $($tail),* ; $code)
        } else {
            None
        }
    };
    ($node:ident; $head:ident, $($tail:ident),* ; $code:block ) => {
        if let Ok(mut $head) = $head.lock() {
            $node.update(stringify!($head).to_string(), &*$head);
            lock_globals!($node; $($tail),* ; $code)
        } else {
            None
        }
    };
}
#[macro_export]
macro_rules! read_globals {
    ( $mutex:ident ; $code:block ) => {
        if let Ok($mutex) = $mutex.lock() {
            let ret = {$code};
            Some(ret)
        } else {
            None
        }
    };
    ( | $($tail:ident),* | => $code:block ) => {
        read_globals!($($tail),* ; $code)
    };
    ( $head:ident, $($tail:ident),* ; $code:block ) => {
        if let Ok($head) = $head.lock() {
            read_globals!($($tail),* ; $code)
        } else {
            None
        }
    };
}





