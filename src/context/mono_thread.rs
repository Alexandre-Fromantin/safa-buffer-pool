use std::{
    cell::RefCell,
    cmp::min,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut, Index, IndexMut},
    rc::Rc,
};

use crate::{BufferPool as RawBufferPool, BufferPoolBuilder};

///BufferPool for Mono-threading context
#[derive(Clone)]
pub struct BufferPool {
    inner_rc: Rc<RefCell<RawBufferPool>>,
}

impl BufferPool {
    pub fn get(&self) -> Option<BufferGuard> {
        let mut pool = self.inner_rc.borrow_mut();

        if let Some(new_buffer) = pool.all_available_buffer.pop() {
            let nb_available_buffer = pool.all_available_buffer.len();
            if nb_available_buffer < pool.min_available_nb_buffer {
                pool.min_available_nb_buffer = nb_available_buffer;
            }

            return Some(BufferGuard {
                pool: self.clone(),
                buffer: ManuallyDrop::new(new_buffer),
            });
        }

        if pool.total_nb_buffer == pool.max_nb_buffer {
            return None;
        }
        pool.total_nb_buffer += 1;

        let buffer_size = pool.buffer_size;

        Some(BufferGuard {
            pool: self.clone(),
            buffer: ManuallyDrop::new(vec![0u8; buffer_size].into_boxed_slice()),
        })
    }

    pub fn reduce_allocated_buffer(&mut self) {
        let mut pool = self.inner_rc.borrow_mut();

        let total_droppable_buffer = pool.total_nb_buffer - pool.min_nb_buffer;
        let nb_drop_buffer = min(total_droppable_buffer, pool.min_available_nb_buffer);

        let nb_buffer_to_keep = pool.all_available_buffer.len() - nb_drop_buffer;
        pool.total_nb_buffer -= nb_drop_buffer;
        pool.all_available_buffer.truncate(nb_buffer_to_keep);

        pool.min_available_nb_buffer = pool.all_available_buffer.len();
    }

    pub fn from_builder(builder: &BufferPoolBuilder) -> BufferPool {
        let mut all_buffer = Vec::with_capacity(builder.max_nb_buffer);

        for _ in 0..builder.min_nb_buffer {
            all_buffer.push(vec![0u8; builder.buffer_size].into_boxed_slice());
        }

        BufferPool {
            inner_rc: Rc::new(RefCell::new(RawBufferPool {
                total_nb_buffer: builder.min_nb_buffer,

                max_nb_buffer: builder.max_nb_buffer,
                min_nb_buffer: builder.min_nb_buffer,
                buffer_size: builder.buffer_size,

                min_available_nb_buffer: builder.min_nb_buffer,

                all_available_buffer: all_buffer,
            })),
        }
    }
}

pub struct BufferGuard {
    pool: BufferPool,
    buffer: ManuallyDrop<Box<[u8]>>,
}

impl Drop for BufferGuard {
    fn drop(&mut self) {
        let mut pool = self.pool.inner_rc.borrow_mut();
        let buffer = unsafe { ManuallyDrop::take(&mut self.buffer) };
        pool.all_available_buffer.push(buffer);
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
