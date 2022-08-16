#[cfg(loom)]
pub(crate) use loom::sync::atomic::AtomicU64;
#[cfg(loom)]
pub(crate) use loom::sync::Arc;
#[cfg(loom)]
pub(crate) use loom::sync::RwLock;
#[cfg(loom)]
pub(crate) use loom::sync::RwLockWriteGuard;

#[cfg(not(loom))]
pub(crate) use std::sync::Arc;
#[cfg(not(loom))]
pub(crate) use tokio::sync::RwLock;
#[cfg(not(loom))]
pub(crate) use tokio::sync::RwLockWriteGuard;
