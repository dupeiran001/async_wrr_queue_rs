use async_wrr_queue::WrrQueue;

#[cfg(feature = "blocking")]
fn main() {
    let mut queue = WrrQueue::new();
    queue.insert_many(vec![("a", 1usize), ("b", 2usize)]);
    queue.insert(("c", 3usize));
    queue.insert_many(vec![("d", 5usize), ("e", 2usize)]);
    let mut expected = [
        "d", "c", "b", "d", "e", "d", "c", "a", "d", "b", "e", "c", "d",
    ]
    .iter()
    .cycle();
    for _ in 0..30 {
        let select = queue.select();
        assert_eq!(expected.next().unwrap(), select.unwrap().data(),);
    }
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() {
    let mut queue = WrrQueue::new();
    queue.insert_many(vec![("a", 1usize), ("b", 2usize)]).await;
    queue.insert(("c", 3usize)).await;
    queue.insert_many(vec![("d", 5usize), ("e", 2usize)]).await;
    let mut expected = [
        "d", "c", "b", "d", "e", "d", "c", "a", "d", "b", "e", "c", "d",
    ]
    .iter()
    .cycle();
    for _ in 0..30 {
        let select = queue.select().await;
        assert_eq!(expected.next().unwrap(), select.unwrap().data(),);
    }
}
