use std::any::Any;
use std::hash::Hash;
use std::{collections::HashSet, sync::Arc};

use dashmap::DashSet;
use get_size::GetSize;
use tokio::time::Instant;

use lazy_static::lazy_static;

use crate::cached::{Cached, CachedInner};

pub struct MemoryManager {
    data: DashSet<CachedDyn>,
}

lazy_static! {
    pub static ref MEMORY_MANAGER: MemoryManager = MemoryManager::new();
}

pub struct CachedDyn(Arc<dyn CachedData + Send + Sync + 'static>);

// struct DynData(Arc<dyn Data>);

// impl Hash for DynData {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.0.addr().hash(state);
//         // state.write_usize(self as *const DynData as usize)
//     }
// }

// impl PartialEq for DynData {
//     fn eq(&self, other: &Self) -> bool {
//         self.0.addr() == other.0.addr()
//     }
// }

// impl Eq for DynData {}

impl Hash for CachedDyn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.addr().hash(state);
    }
}

impl PartialEq for CachedDyn {
    fn eq(&self, other: &Self) -> bool {
        // let hi = self.0.as_ref();
        // let hi = hi as *const dyn CachedData;
        // let hi = hi.to_raw_parts();
        // std::ptr::
        // self.0.as_ref().vtable_and_ptr()
        // Any::
        self.0.addr() == other.0.addr()
    }
}

impl Eq for CachedDyn {}

impl<T> Cached<T>
where
    T: Data + Clone + Send + Sync + 'static,
{
    pub fn clone_dyn(&self) -> CachedDyn {
        CachedDyn(Arc::clone(&self.0) as Arc<dyn CachedData + Send + Sync + 'static>)
    }

    pub fn enroll(&self) {
        MEMORY_MANAGER.enroll(self.clone_dyn());
    }
}

pub trait Data: Send + Sync + 'static {
    fn get_size(&self) -> usize;
    fn get_ref_count(&self) -> usize {
        1
    }
}

pub trait CachedData {
    fn free(&self);
    fn addr(&self) -> usize;
    fn get_size(&self) -> usize;
    fn get_ref_count(&self) -> usize;
    fn get_last_accessed(&self) -> Option<Instant>;
    // fn get_data(&self) -> Option<&dyn Data>;
}

impl<T: Data> CachedData for CachedInner<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn free(&self) {
        self.clear();
    }

    fn addr(&self) -> usize {
        self as *const CachedInner<T> as usize
    }

    fn get_size(&self) -> usize {
        match self.try_get_sync() {
            Some(Ok(x)) => x.get_size(),
            _ => 0,
        }
    }

    fn get_ref_count(&self) -> usize {
        match self.try_get_sync() {
            Some(Ok(x)) => x.get_ref_count(),
            _ => 0,
        }
    }

    fn get_last_accessed(&self) -> Option<Instant> {
        CachedInner::get_last_accessed(self)
    }
}

// impl<T: GetSize + Send + Sync + 'static> Data for T {
//     fn get_size(&self) -> usize {
//         GetSize::get_size(self)
//     }
// }

impl<T: GetSize + Send + Sync + 'static> Data for Arc<T> {
    fn get_size(&self) -> usize {
        GetSize::get_size(self)
    }

    fn get_ref_count(&self) -> usize {
        Arc::strong_count(self)
    }
}

impl<T> Data for Cached<T>
where
    T: Clone + Send + Sync + 'static,
    T: GetSize + Eq,
{
    fn get_size(&self) -> usize {
        match self.try_get_sync() {
            Some(Ok(x)) => x.get_size(),
            _ => 0,
        }
    }
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            data: DashSet::new(),
        }
    }

    pub fn evict_oldest(&self) {
        todo!()
    }

    pub fn print_stats(&self) {
        for x in self.data.iter() {
            println!("{:?}", x.0.get_size());
        }
    }

    pub fn enroll(&self, data: CachedDyn) {
        self.data.insert(data);
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

