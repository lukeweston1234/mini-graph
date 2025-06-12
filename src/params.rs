use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ParamBool {
    data: Arc<AtomicBool>
}
impl ParamBool {
    pub fn new(val: bool) -> Self {
        Self {
            data: Arc::new(AtomicBool::new(val))
        }
    }
    pub fn store(&self, val: bool){
        self.data.store(val, Ordering::Relaxed);
    }
    pub fn get(&self) -> bool{
        self.data.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct ParamU32 {
    data: Arc<AtomicU32>
}
impl ParamU32 {
    pub fn new(val: u32) -> Self {
        Self {
            data: Arc::new(AtomicU32::new(val))
        }
    }
    pub fn store(&self, val: u32){
        self.data.store(val, Ordering::Relaxed);
    }
    pub fn get(&self) -> u32{
        self.data.load(Ordering::Relaxed)
    }
}
#[derive(Clone)]
pub struct ParamF32 {
    data: Arc<AtomicF32>
}
impl ParamF32 {
    pub fn new(val: f32) -> Self {
        Self {
            data: Arc::new(AtomicF32::new(val))
        }
    }
    pub fn store(&self, val: f32){
        self.data.store(val, Ordering::Relaxed);
    }
    pub fn get(&self) -> f32{
        self.data.load(Ordering::Relaxed)
    }
    pub fn add(&self, val: f32) -> f32 {
        self.data.add(val, Ordering::Relaxed)
    }
}

struct AtomicF32 {
    storage: AtomicU32
}
impl AtomicF32 {
    fn new(value: f32) -> Self {
        let as_u32 = value.to_bits();
        Self {
            storage: AtomicU32::new(as_u32)
        }
    }
    fn store(&self, value: f32, order: Ordering) {
        let as_u32 = value.to_bits();
        self.storage.store(as_u32, order);
    }
    fn load(&self, order: Ordering) -> f32 {
        let as_u32 = self.storage.load(order);
        f32::from_bits(as_u32)
    }
    fn add(&self, val: f32, order: Ordering) -> f32 {
        self.storage.fetch_update(order, order, |current_bits|{
            let current = f32::from_bits(current_bits);
            Some((current + val).to_bits())
        })
        .map(f32::from_bits)
        .expect("fetch_update failed")
    }
}