use std::{
    cmp::min,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut, Index, IndexMut},
    sync::Arc,
};

use tokio::{sync::Mutex, time::sleep};

use crate::{BufferPool as RawBufferPool, BufferPoolBuilder};

///BufferPool for Multi-thread context
#[derive(Clone)]
pub struct BufferPool {
    inner_arc: Arc<Mutex<RawBufferPool>>,
}

impl BufferPool {
    ///Get a new buffer from the pool
    /// 
    ///Return None if none buffer available
    pub async fn get(&self) -> Option<BufferGuard> {
        let mut pool = self.inner_arc.lock().await;

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

        drop(pool); //Free lock

        Some(BufferGuard {
            pool: self.clone(),
            buffer: ManuallyDrop::new(vec![0u8; buffer_size].into_boxed_slice()),
        })
    }

    ///Like BufferPoolBuilder.build_multi_thread()
    pub fn from_builder(builder: &BufferPoolBuilder) -> BufferPool {
        let mut all_buffer = Vec::with_capacity(builder.max_nb_buffer);

        for _ in 0..builder.min_nb_buffer {
            all_buffer.push(vec![0u8; builder.buffer_size].into_boxed_slice());
        }

        let pool = BufferPool {
            inner_arc: Arc::new(Mutex::new(RawBufferPool {
                total_nb_buffer: builder.min_nb_buffer,

                max_nb_buffer: builder.max_nb_buffer,
                min_nb_buffer: builder.min_nb_buffer,
                buffer_size: builder.buffer_size,

                min_available_nb_buffer: builder.min_nb_buffer,

                all_available_buffer: all_buffer,
            })),
        };

        if let Some(over_buffer_lifetime) = builder.over_buffer_lifetime_opt {
            let pool_weak = Arc::downgrade(&pool.inner_arc);
            tokio::spawn(async move {
                let mut pool_destroyed = false;
                while !pool_destroyed {
                    sleep(over_buffer_lifetime).await;

                    if let Some(pool_arc) = pool_weak.upgrade() {
                        let mut pool = pool_arc.lock().await;

                        let total_droppable_buffer = pool.total_nb_buffer - pool.min_nb_buffer;
                        let nb_drop_buffer =
                            min(total_droppable_buffer, pool.min_available_nb_buffer);

                        let nb_buffer_to_keep = pool.all_available_buffer.len() - nb_drop_buffer;
                        pool.total_nb_buffer -= nb_drop_buffer;
                        pool.all_available_buffer.truncate(nb_buffer_to_keep);

                        pool.min_available_nb_buffer = pool.all_available_buffer.len();
                    } else {
                        pool_destroyed = true;
                    }
                }
            });
        }

        pool
    }
}

///A buffer guard for auto-drop of buffer, useable like a buffer
pub struct BufferGuard {
    pool: BufferPool,
    buffer: ManuallyDrop<Box<[u8]>>,
}

impl Drop for BufferGuard {
    fn drop(&mut self) {
        let pool_mtx = self.pool.clone();
        let buffer = unsafe { ManuallyDrop::take(&mut self.buffer) }; //unsafe without risk

        tokio::spawn(async move {
            let mut pool = pool_mtx.inner_arc.lock().await;
            pool.all_available_buffer.push(buffer);
        });
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
