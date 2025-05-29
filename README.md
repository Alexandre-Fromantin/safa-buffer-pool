Safa-buffer-pool is a quick and simple tool to create buffer pools in a mono or multi thread context.

# Overview

To create a pool you must use BufferPoolBuilder, define or not these parameters and used the build_mono_thread or build_multi_thread functions according to the context.


### Lifetime

When the number of buffers is greater than the minimum defined, they are removed according to the life of the over buffer(the over buffersare that are not used between interval of the life).  
A(first call) --> lifetime --> B(end call): All buffers not used between moments A and B are drop (keeps the minimum buffer).  
⚠️In Mono-Thread context, the lifetime does not work automatically, you must use reduce_allocated_buffer() which will calculate the buffer overages.

# Exemple

### Mono-Thread