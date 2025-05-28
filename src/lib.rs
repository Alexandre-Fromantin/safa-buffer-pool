use std::{mem::ManuallyDrop, sync::Arc};

use tokio::sync::Mutex;

///BufferPool for Multi-threading context
#[derive(Clone)]
pub struct MultiThreadBufferPool {
    inner_arc: Arc<Mutex<BufferPool>>,
}

unsafe impl Send for BufferPool {}

impl MultiThreadBufferPool {
    pub async fn get(&self) -> Option<BufferGuard> {
        let mut pool = self.inner_arc.lock().await;

        if let Some(new_buffer) = pool.all_available_buffer.pop() {
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
}

///Unique BufferPool
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

    pub fn set_max_number_of_buffer(&mut self, number: usize) -> &mut Self {
        self.max_nb_buffer = number;
        self
    }
    pub fn set_min_number_of_buffer(&mut self, number: usize) -> &mut Self {
        self.min_nb_buffer = number;
        if self.max_nb_buffer < number {
            self.max_nb_buffer = number;
        }
        self
    }
    pub fn set_buffer_size(&mut self, number: usize) -> &mut Self {
        self.buffer_size = number;
        self
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

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::BufferPoolBuilder;

    #[tokio::test]
    async fn basic_test() {
        let pool = BufferPoolBuilder::new().build_multi_thread_buffer_pool();

        let pool_cl = pool.clone();
        let A = tokio::spawn(async move {
            for _ in 0..100000 {
                println!("A: try to get a new buffer");
                let new_buffer_opt = pool_cl.get().await;
                if new_buffer_opt.is_none() {
                    println!("A: buffer no available");
                    break;
                }
                let mut new_buffer = new_buffer_opt.unwrap();
                new_buffer.buffer[0] = 0x01;
                new_buffer.buffer[1] = 0x02;
                new_buffer.buffer[2] = 0x03;
                println!("A: free a buffer");
            }
        });

        let pool_cl2 = pool.clone();
        let B = tokio::spawn(async move {
            for _ in 0..100000 {
                println!("B: try to get a new buffer");
                let new_buffer_opt = pool_cl2.get().await;
                if new_buffer_opt.is_none() {
                    println!("B: buffer no available");
                    break;
                }
                let mut new_buffer = new_buffer_opt.unwrap();
                new_buffer.buffer[0] = 0x01;
                new_buffer.buffer[1] = 0x02;
                new_buffer.buffer[2] = 0x03;
                println!("B: free a buffer");
            }
        });

        let _ = A.await;
        let _ = B.await;
    }

    #[test]
    fn big_pool() {
        //10Go Pool (10240 buffer of 1Mo)
        let pool = BufferPoolBuilder::new()
            .set_buffer_size(1024 * 1024 * 1024 * 50)
            .set_min_number_of_buffer(1)
            .build_multi_thread_buffer_pool();

        sleep(Duration::from_secs(5));
    }
}
