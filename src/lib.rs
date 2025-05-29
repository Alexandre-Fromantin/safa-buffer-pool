use std::time::Duration;

use context::{mono_thread, multi_thread};
pub mod context;

///Unique BufferPool
struct BufferPool {
    total_nb_buffer: usize,

    max_nb_buffer: usize,
    min_nb_buffer: usize,
    buffer_size: usize,

    min_available_nb_buffer: usize,

    all_available_buffer: Vec<Box<[u8]>>,
}

pub struct BufferPoolBuilder {
    max_nb_buffer: usize,
    min_nb_buffer: usize,
    buffer_size: usize,
    over_buffer_lifetime_opt: Option<Duration>,
}

impl Default for BufferPoolBuilder {
    fn default() -> Self {
        Self {
            max_nb_buffer: 1024,
            min_nb_buffer: 1024,
            buffer_size: 10240,
            over_buffer_lifetime_opt: None,
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
    pub fn set_over_buffer_lifetime(&mut self, new_duration: Duration) -> &mut Self {
        self.over_buffer_lifetime_opt = Some(new_duration);
        self
    }

    pub fn build_mono_thread(&self) -> mono_thread::BufferPool {
        mono_thread::BufferPool::from_builder(self)
    }

    pub fn build_multi_thread(&self) -> multi_thread::BufferPool {
        multi_thread::BufferPool::from_builder(self)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::{task, time::sleep};

    use crate::BufferPoolBuilder;

    #[tokio::test]
    async fn basic_test() {
        let pool = BufferPoolBuilder::new().build_multi_thread();

        let pool_cl = pool.clone();
        let a = tokio::spawn(async move {
            for _ in 0..100000 {
                //println!("A: try to get a new buffer");
                let new_buffer_opt = pool_cl.get().await;
                if new_buffer_opt.is_none() {
                    //println!("A: buffer no available");
                    break;
                }
                let mut new_buffer = new_buffer_opt.unwrap();
                new_buffer[0] = 0x01;
                new_buffer[1] = 0x02;
                new_buffer[2] = 0x03;
                //println!("A: free a buffer");
            }
        });

        let pool_cl2 = pool.clone();
        let b = tokio::spawn(async move {
            for _ in 0..100000 {
                //println!("B: try to get a new buffer");
                let new_buffer_opt = pool_cl2.get().await;
                if new_buffer_opt.is_none() {
                    //println!("B: buffer no available");
                    break;
                }
                let mut new_buffer = new_buffer_opt.unwrap();
                new_buffer[0] = 0x01;
                new_buffer[1] = 0x02;
                new_buffer[2] = 0x03;
                //println!("B: free a buffer");
            }
        });

        let _ = a.await;
        let _ = b.await;
    }

    #[test]
    fn big_pool() {
        //10Go Pool (10240 buffer of 1Mo)
        let _pool = BufferPoolBuilder::new()
            .set_buffer_size(1024 * 1024)
            .set_min_number_of_buffer(10240)
            .build_mono_thread();
    }

    #[tokio::test]
    async fn over_buffer() {
        let pool = BufferPoolBuilder::new()
            .set_buffer_size(1024 * 1024)
            .set_min_number_of_buffer(100)
            .set_max_number_of_buffer(200)
            .set_over_buffer_lifetime(Duration::from_secs(10))
            .build_multi_thread();

        for _ in 0..150 {
            let pool_cl = pool.clone();
            task::spawn(async move {
                let _buffer = pool_cl.get().await;
                sleep(Duration::from_secs(11)).await;
            });
        }

        sleep(Duration::from_secs(25)).await;

        for _ in 0..120 {
            let pool_cl = pool.clone();
            task::spawn(async move {
                let _buffer = pool_cl.get().await;
                sleep(Duration::from_secs(15)).await;
            });
        }

        sleep(Duration::from_secs(25)).await;
    }
}
