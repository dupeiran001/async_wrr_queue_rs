
# Async WRR Queue

this is a wrapping of weighted round-robin schedule algorithm, utilizing atomic 
operation and cache queue in order to avoid lock latency or the schedule latency.
And we have used an async RwLock (feature `default` or `tokio`) to overcome the 
conflict of select instance and recalculate queue.

- async interface for tokio
- Atomic operation aimed to provide the best run-time performance 
- dynamic insert supported

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