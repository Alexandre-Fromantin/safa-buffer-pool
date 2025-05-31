use std::{ mem::ManuallyDrop, ops::{ Deref, DerefMut, Index, IndexMut }, sync::{ Arc, Weak } };

use crate::builder::BufferPoolBuilder;

use super::common::BufferPool as RawBufferPool;

#[cfg(feature = "async")]
use tokio::{ sync::Mutex, time::sleep };
#[cfg(not(feature = "async"))]
use std::{ sync::Mutex, thread::sleep };

///BufferPool for Multi-thread context
#[derive(Clone)]
pub struct BufferPool {
    inner_arc: Arc<Mutex<RawBufferPool>>,
}

impl BufferPool {
    ///Get a new buffer from the pool
    ///
    ///Return None if none buffer available
    #[cfg(feature = "async")]
    pub async fn get(&self) -> Option<BufferGuard> {
        let mut pool = self.inner_arc.lock().await;

        pool.get().map(|buffer| BufferGuard {
            pool: self.clone(),
            buffer,
        })
    }
    #[cfg(not(feature = "async"))]
    pub fn get(&self) -> Option<BufferGuard> {
        let mut pool = self.inner_arc.lock().expect("mutex lock error");

        pool.get().map(|buffer| BufferGuard {
            pool: self.clone(),
            buffer,
        })
    }

    ///Optimize the number of buffer by deleted excess buffer of the pool
    ///
    /// Return if the pool was droped
    #[cfg(feature = "async")]
    async fn clean_excess_buffer(pool_weak: &Weak<Mutex<RawBufferPool>>) -> bool {
        if let Some(pool_arc) = pool_weak.upgrade() {
            let mut pool = pool_arc.lock().await;

            pool.clean_excess_buffer();

            return false;
        }
        true
    }
    #[cfg(not(feature = "async"))]
    fn clean_excess_buffer(pool_weak: &Weak<Mutex<RawBufferPool>>) -> bool {
        if let Some(pool_arc) = pool_weak.upgrade() {
            let mut pool = pool_arc.lock().expect("mutex lock error");

            pool.clean_excess_buffer();

            return false;
        }
        true
    }

    ///Like BufferPoolBuilder.build_multi_thread()
    pub fn from_builder(builder: &BufferPoolBuilder) -> Self {
        let pool = Self {
            inner_arc: Arc::new(Mutex::new(RawBufferPool::from_builder(builder))),
        };

        if let Some(over_buffer_lifetime) = builder.over_buffer_lifetime_opt {
            let pool_weak = Arc::downgrade(&pool.inner_arc);

            #[cfg(feature = "async")]
            tokio::spawn(async move {
                loop {
                    sleep(over_buffer_lifetime).await;

                    if BufferPool::clean_excess_buffer(&pool_weak).await {
                        return;
                    }
                }
            });

            #[cfg(not(feature = "async"))]
            std::thread::spawn(move || {
                loop {
                    sleep(over_buffer_lifetime);

                    if BufferPool::clean_excess_buffer(&pool_weak) {
                        return;
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

        #[cfg(feature = "async")]
        tokio::spawn(async move {
            let mut pool = pool_mtx.inner_arc.lock().await;
            pool.free(buffer);
        });

        #[cfg(not(feature = "async"))]
        std::thread::spawn(move || {
            let mut pool = pool_mtx.inner_arc.lock().expect("mutex lock error");
            pool.free(buffer);
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
