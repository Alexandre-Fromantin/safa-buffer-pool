use safa_buffer_pool::builder::BufferPoolBuilder;
use std::thread;

fn main() {
    //10Go Pool (10240 buffer of 1Mo)
    let pool = BufferPoolBuilder::new()
        .set_buffer_size(1024 * 1024)
        .set_min_number_of_buffer(10240)
        .build_multi_thread();

    let pool_clone = pool.clone();
    thread::spawn(move || {
        let buffer1_option = pool_clone.get();
        if buffer1_option.is_none() {
            println!("none buffer available");
            return;
        }
        let mut buffer1 = buffer1_option.unwrap();

        buffer1[0] = 0x01;
        buffer1[1] = 0x02;
        buffer1[2] = 0x03;

        //auto free buffer1
    });

    thread::spawn(move|| {
        let buffer2_option = pool.get();
        if buffer2_option.is_none() {
            println!("none buffer available");
            return;
        }
        let buffer2 = buffer2_option.unwrap();

        drop(buffer2); //free buffer2

        println!("buffer2 free");
    });
}
