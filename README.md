# S3FIFO

This is a *pretty rough* Rust implementation of the [S3FIFO paper](https://dl.acm.org/doi/10.1145/3600006.3613147) presented at SOSP '23 by authors - "Yang, Juncheng and Zhang, Yazhuo and Qiu, Ziyue and Yue, Yao and Rashmi, K.V." Their paper submission repository is [here](https://github.com/Thesys-lab/sosp23-s3fifo).

**NOTE:** This is **NOT** a complete S3FIFO implementation. Currently, it does not guarantee thread safety, lacks extensive unit tests, and there's no evaluation using real-world traces. It's just me having some fun with Rust... and, admittedly, procrastinating on my own research...

Any contributions are warmly welcome! I will get back to this when I have more time.

## Key Intuition

Let me jot down some key intuitions behind their idea.

### One-Hit-Wonder

**Definition:** An object (cache object) that is only requested/accessed once within a certain period.

### High One-Hit-Wonder Ratio

The authors observed that in real-world access traces, the one-hit-wonder ratio is quite high (details are in the paper). With shorter trace sequences, this ratio increases. Therefore, quickly eliminating these one-hit-wonders becomes key.

### Relation to Cache Eviction Policy

Imagine a cache with infinite capacity; it could contain all trace data (access patterns for every object). However, in reality, cache capacity is finite. Thus, it captures only a short trace sequence. This suggests that a cache might contain a high ratio of one-hit-wonders.

### Their Approach

The core idea involves two FIFO queues: one for "temporary usage" (referred to as a small cache) and the main cache. When an object is evicted from the small cache and hasn't been accessed yet, it undergoes "quick demotion." Otherwise, it is inserted into the main cache. Many details are omitted here. Check the paper if you're interested.

## Why FIFO Queue Can Be Better Than LRU Policy

A FIFO queue, especially when implemented with a ring buffer, doesn't necessarily require locking for eviction policy implementation. In contrast, LRU is typically implemented using a doubly-linked list, which requires locking when manipulating pointers.

Through evaluation, the authors found that S3FIFO provides a lower miss rate compared to state-of-the-art LRU policies.
