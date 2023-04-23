use std::any::Any;
use std::hash::Hash;
use std::{collections::HashSet, sync::Arc};

use dashmap::DashSet;
use get_size::GetSize;
use tokio::time::Instant;

use lazy_static::lazy_static;

use crate::cached::Cached;

pub struct MemoryManager {
    data: DashSet<DynData>,
}

lazy_static! {
    pub static ref MEMORY_MANAGER: MemoryManager = MemoryManager::new();
}

struct DynData(Arc<dyn Data + Send + Sync + 'static>);

impl Hash for DynData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.addr().hash(state);
        // state.write_usize(self as *const DynData as usize)
    }
}

impl PartialEq for DynData {
    fn eq(&self, other: &Self) -> bool {
        self.0.addr() == other.0.addr()
    }
}

impl Eq for DynData {}

// impl Hash for DynData {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.0.get_size().hash(state);
//     }
// }

// impl PartialEq for DynData {
//     fn eq(&self, other: &Self) -> bool {
//         // self.0.type_id() == other.0.type_id()
//         let a = &self.0 as &dyn Any;

//     }
// }

pub trait Data {
    fn addr(&self) -> usize;
    fn get_size(&self) -> usize;
    fn get_last_accessed(&self) -> Option<Instant>;
    fn free(&self);
}

impl<T> Data for Cached<T>
where
    T: Clone + Send + Sync + 'static,
    T: GetSize + Eq,
{
    fn addr(&self) -> usize {
        self as *const _ as usize
    }

    fn get_size(&self) -> usize {
        match self.try_get_sync() {
            Some(Ok(x)) => x.get_size(),
            _ => 0,
        }
    }

    fn get_last_accessed(&self) -> Option<Instant> {
        self.get_last_accessed()
    }

    fn free(&self) {
        let _ = self.clear();
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

    pub fn enroll<T: Data + Send + Sync + 'static>(&self, data: Arc<T>) {
        self.data
            .insert(DynData(data as Arc<dyn Data + Send + Sync + 'static>));
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn enroll<T: Data + Send + Sync + 'static>(data: Arc<T>) {
    MEMORY_MANAGER.enroll(data);
}