use std::time::Duration;

use crate::context::{mono_thread, multi_thread};

///A builder for BufferPool
pub struct BufferPoolBuilder {
    pub max_nb_buffer: usize,
    pub min_nb_buffer: usize,
    pub buffer_size: usize,
    pub over_buffer_lifetime_opt: Option<Duration>,
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
    ///Create a new BufferPoolBuilder
    pub fn new() -> Self {
        Self::default()
    }

    ///Set the maximum of buffer in the pool
    pub fn set_max_number_of_buffer(&mut self, number: usize) -> &mut Self {
        self.max_nb_buffer = number;
        self
    }
    ///Set the minimum of buffer in the pool
    pub fn set_min_number_of_buffer(&mut self, number: usize) -> &mut Self {
        self.min_nb_buffer = number;
        if self.max_nb_buffer < number {
            self.max_nb_buffer = number;
        }
        self
    }
    ///Set the size of each buffer in the pool
    pub fn set_buffer_size(&mut self, number: usize) -> &mut Self {
        self.buffer_size = number;
        self
    }
    ///Only work on Multi-thread pool: Set the maximum inactivity time for excess buffers before being deleted
    pub fn set_over_buffer_lifetime(&mut self, new_duration: Duration) -> &mut Self {
        self.over_buffer_lifetime_opt = Some(new_duration);
        self
    }

    ///Build a mono thread pool from this builder
    pub fn build_mono_thread(&self) -> mono_thread::BufferPool {
        mono_thread::BufferPool::from_builder(self)
    }
    ///Build a multi thread pool from this builder
    pub fn build_multi_thread(&self) -> multi_thread::BufferPool {
        multi_thread::BufferPool::from_builder(self)
    }
}