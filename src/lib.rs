pub mod context;
pub mod builder;

#[cfg(test)]
mod tests {

    #[cfg(feature = "async")]
    use std::time::Duration;

    #[cfg(feature = "async")]
    use tokio::{task, time::sleep};

    use crate::builder::BufferPoolBuilder;


    #[cfg(feature = "async")]
    #[tokio::test]
    async fn basic_test() {
        let pool = BufferPoolBuilder::new().build_multi_thread();

        let pool_cl = pool.clone();
        let a = tokio::spawn(async move {
            for _ in 0..10000 {
                println!("A: try to get a new buffer");

                let new_buffer_opt = pool_cl.get().await;

                if new_buffer_opt.is_none() {
                    println!("A: buffer no available");
                    break;
                }
                let mut new_buffer = new_buffer_opt.unwrap();
                new_buffer[0] = 0x01;
                new_buffer[1] = 0x02;
                new_buffer[2] = 0x03;
                println!("A: free a buffer");
            }
        });

        let pool_cl2 = pool.clone();
        let b = tokio::spawn(async move {
            for _ in 0..10000 {
                println!("B: try to get a new buffer");

                let new_buffer_opt = pool_cl2.get().await;

                if new_buffer_opt.is_none() {
                    println!("B: buffer no available");
                    break;
                }
                let mut new_buffer = new_buffer_opt.unwrap();
                new_buffer[0] = 0x01;
                new_buffer[1] = 0x02;
                new_buffer[2] = 0x03;
                println!("B: free a buffer");
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

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn over_buffer() {
        let pool = BufferPoolBuilder::new()
            .set_buffer_size(1024 * 1024)
            .set_min_number_of_buffer(100)
            .set_max_number_of_buffer(200)
            .set_over_buffer_lifetime(Duration::from_secs(2))
            .build_multi_thread();

        for _ in 0..150 {
            let pool_cl = pool.clone();
            task::spawn(async move {
                let _buffer = pool_cl.get().await;
                sleep(Duration::from_secs(3)).await;
            });
        }

        sleep(Duration::from_secs(4)).await;

        for _ in 0..120 {
            let pool_cl = pool.clone();
            task::spawn(async move {
                let _buffer = pool_cl.get().await;
                sleep(Duration::from_secs(3)).await;
            });
        }

        sleep(Duration::from_secs(5)).await;
    }
}
