use std::sync::Arc;

use tokio::sync::Mutex;

///BufferPool for Multi-threading context
#[derive(Clone, Default)]
pub struct MultiThreadBufferPool {
    inner_arc: Arc<Mutex<BufferPool>>,
}

impl MultiThreadBufferPool {
    pub async fn get(&self) -> Option<BufferGuard> {
        let mut pool = self.inner_arc.lock().await;
        if let Some(new_buffer) = pool.all_available_buffer.pop() {
            return Some(BufferGuard {
                pool: self.clone(),
                buffer: new_buffer,
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
            buffer: vec![0u8; buffer_size].into_boxed_slice(),
        })
    }
}

///Unique BufferPool
#[derive(Default)]
struct BufferPool {
    total_nb_buffer: usize,
    max_nb_buffer: usize,
    min_nb_buffer: usize,
    buffer_size: usize,
    all_available_buffer: Vec<Box<[u8]>>,
}

pub struct BufferPoolBuilder {
    max_nb_buffer: usize,
    min_nb_buffer: usize,
    buffer_size: usize,
}

impl Default for BufferPoolBuilder {
    fn default() -> Self {
        Self {
            max_nb_buffer: 1024,
            min_nb_buffer: 1024,
            buffer_size: 10240,
        }
    }
}

impl BufferPoolBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_max_number_of_buffer(&mut self, number: usize) {
        self.max_nb_buffer = number;
    }
    pub fn set_min_number_of_buffer(&mut self, number: usize) {
        self.min_nb_buffer = number;
    }
    pub fn set_buffer_size(&mut self, number: usize) {
        self.buffer_size = number;
    }

    pub fn build_multi_thread_buffer_pool(&self) -> MultiThreadBufferPool {
        let mut all_buffer = Vec::with_capacity(self.max_nb_buffer);

        for _ in 0..self.min_nb_buffer {
            all_buffer.push(vec![0u8; self.buffer_size].into_boxed_slice());
        }

        MultiThreadBufferPool {
            inner_arc: Arc::new(Mutex::new(BufferPool {
                total_nb_buffer: self.min_nb_buffer,
                max_nb_buffer: self.max_nb_buffer,
                min_nb_buffer: self.min_nb_buffer,
                buffer_size: self.buffer_size,
                all_available_buffer: all_buffer,
            })),
        }
    }
}

pub struct BufferGuard {
    pool: MultiThreadBufferPool,
    buffer: Box<[u8]>,
}

impl Drop for BufferGuard {
    fn drop(&mut self) {
        let pool = std::mem::take(&mut self.pool);
        let buffer = std::mem::take(&mut self.buffer);

        tokio::spawn(async move {
            let mut pool = pool.inner_arc.lock().await;
            pool.all_available_buffer.push(buffer);
        });
    }
}
