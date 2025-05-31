use std::{cmp::min, mem::ManuallyDrop};

use crate::builder::BufferPoolBuilder;

///Unique BufferPool
pub struct BufferPool {
    total_nb_buffer: usize,

    max_nb_buffer: usize,
    min_nb_buffer: usize,
    buffer_size: usize,

    min_available_nb_buffer: usize,

    all_available_buffer: Vec<Box<[u8]>>,
}

impl BufferPool {
    pub fn get(&mut self) -> Option<ManuallyDrop<Box<[u8]>>> {
        if let Some(new_buffer) = self.all_available_buffer.pop() {
            let nb_available_buffer = self.all_available_buffer.len();
            if nb_available_buffer < self.min_available_nb_buffer {
                self.min_available_nb_buffer = nb_available_buffer;
            }

            return Some(ManuallyDrop::new(new_buffer));
        }

        if self.total_nb_buffer == self.max_nb_buffer {
            return None;
        }
        self.total_nb_buffer += 1;

        let buffer_size = self.buffer_size;

        Some(ManuallyDrop::new(vec![0u8; buffer_size].into_boxed_slice()))
    }

    pub fn free(&mut self, buffer: Box<[u8]>){
        self.all_available_buffer.push(buffer);
    }
    
    pub fn clean_excess_buffer(&mut self) {
        let total_droppable_buffer = self.total_nb_buffer - self.min_nb_buffer;
        let nb_drop_buffer = min(total_droppable_buffer, self.min_available_nb_buffer);

        let nb_buffer_to_keep = self.all_available_buffer.len() - nb_drop_buffer;
        self.total_nb_buffer -= nb_drop_buffer;
        self.all_available_buffer.truncate(nb_buffer_to_keep);

        self.min_available_nb_buffer = self.all_available_buffer.len();
    }

    pub fn from_builder(builder: &BufferPoolBuilder) -> BufferPool{
        let mut all_buffer = Vec::with_capacity(builder.max_nb_buffer);

        for _ in 0..builder.min_nb_buffer {
            all_buffer.push(vec![0u8; builder.buffer_size].into_boxed_slice());
        }

        BufferPool {
                total_nb_buffer: builder.min_nb_buffer,

                max_nb_buffer: builder.max_nb_buffer,
                min_nb_buffer: builder.min_nb_buffer,
                buffer_size: builder.buffer_size,

                min_available_nb_buffer: builder.min_nb_buffer,

                all_available_buffer: all_buffer,
            }
    }
}
