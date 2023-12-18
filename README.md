
# Async WRR Queue

this is a wrapping of `weighted round-robin`
schedule algorithm, utilizing [atomic operation](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html) 
and cache queue in order to avoid lock latency or the schedule latency. And we have 
used an async [RwLock](https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html) 
(feature `default` or `tokio`) to overcome the conflict of select instance and 
recalculate queue.

![crate.io](https://img.shields.io/crates/v/async_wrr_queue.svg)
![github actions](https://github.com/dupeiran001/async_wrr_queue_rs/actions/workflows/rust.yml/badge.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

- async interface for tokio
- Atomic operation aimed to provide the best run-time performance 
- dynamic insert supported

more detailed documented [WrrQueue](https://docs.rs/async_wrr_queue/latest/async_wrr_queue/struct.WrrQueue.html) | 
[Instance](https://docs.rs/async_wrr_queue/latest/async_wrr_queue/struct.Instance.html)

## Example 

```rust
use async_wrr_queue::*;

#[tokio::main]
async fn main() {
    let mut queue = WrrQueue::new();
    
    // insert many 
    queue.insert_many(vec![("a", 1usize), ("b", 2usize)]).await;
    
    // insert one
    queue.insert(("c", 3usize)).await;
    queue.insert_many(vec![("d", 5usize), ("e", 2usize)]).await;
    let mut expected = [
        "d", "c", "b", "d", "e", "d", "c", "a", "d", "b", "e", "c", "d",
    ]
        .iter()
        .cycle();
    for _ in 0..30 {
        
        // schedule!
        let select = queue.select().await;
        assert_eq!(expected.next().unwrap(), select.unwrap().data(),);
    }
}
```

## features

- `default` : `tokio` 
- `tokio`    : async interface, using `tokio::sync::RwLock` to guarantee best performance
- `blocking` : not compatible with `tokio`, using `std::sync::RwLock` for blocking acquire