use std::{
    cell::RefCell,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut, Index, IndexMut},
    rc::Rc,
};

use crate::builder::BufferPoolBuilder;

use super::common::BufferPool as RawBufferPool;

///BufferPool for Mono-thread context
#[derive(Clone)]
pub struct BufferPool {
    inner_rc: Rc<RefCell<RawBufferPool>>,
}

impl BufferPool {
    ///Get a new buffer from the pool
    /// 
    ///Return None if none buffer available
    pub fn get(&self) -> Option<BufferGuard> {
        let mut pool = self.inner_rc.borrow_mut();

        pool.get().map(|buffer| BufferGuard{
                pool: self.clone(),
                buffer
            })
    }

    ///Optimize the number of buffer by deleted excess buffer of the pool
    pub fn clean_excess_buffer(&mut self) {
        let mut pool = self.inner_rc.borrow_mut();

        pool.clean_excess_buffer();
    }

    ///Like BufferPoolBuilder.build_mono_thread()
    pub fn from_builder(builder: &BufferPoolBuilder) -> Self {
        Self {
            inner_rc: Rc::new(RefCell::new(RawBufferPool::from_builder(builder))),
        }
    }
}

///A buffer guard for auto-drop of buffer, useable like a buffer
pub struct BufferGuard {
    pool: BufferPool,
    buffer: ManuallyDrop<Box<[u8]>>,
}

impl Drop for BufferGuard {
    fn drop(&mut self) {
        let mut pool = self.pool.inner_rc.borrow_mut();
        let buffer = unsafe{ManuallyDrop::take(&mut self.buffer)};

        pool.free(buffer);
    }
}

impl Deref for BufferGuard {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for BufferGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl Index<usize> for BufferGuard {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.buffer[idx]
    }
}
impl IndexMut<usize> for BufferGuard {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.buffer[idx]
    }
}
