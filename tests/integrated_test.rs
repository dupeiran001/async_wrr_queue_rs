use async_wrr_queue::*;

#[cfg(feature = "tokio")]
#[tokio::test]
async fn tokio_test_usage() {
    let mut queue = WrrQueue::new();
    queue
        .insert_many(vec![("a".to_string(), 1usize), ("b".to_string(), 2usize)])
        .await;
    let mut expected = ["b", "a", "b"].iter().cycle();
    for _ in 0..20 {
        assert_eq!(
            expected.next().unwrap(),
            queue.select().await.unwrap().data()
        );
    }
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn tokio_test_all_equal() {
    let mut queue = WrrQueue::new();
    queue
        .insert_many(vec![("a".to_string(), 1usize), ("b".to_string(), 1usize)])
        .await;
    let mut expected = ["a", "b"].iter().cycle();
    for _ in 0..20 {
        assert_eq!(
            expected.next().unwrap(),
            queue.select().await.unwrap().data()
        );
    }
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn tokio_complex_test() {
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

#[cfg(feature = "blocking")]
#[test]
fn test_usage() {
    let mut queue = WrrQueue::new();
    queue.insert_many(vec![("a".to_string(), 1usize), ("b".to_string(), 2usize)]);
    let mut expected = ["b", "a", "b"].iter().cycle();
    for _ in 0..20 {
        assert_eq!(expected.next().unwrap(), queue.select().unwrap().data());
    }
}

#[cfg(feature = "blocking")]
#[test]
fn test_all_equal() {
    let mut queue = WrrQueue::new();
    queue.insert_many(vec![("a".to_string(), 1usize), ("b".to_string(), 1usize)]);
    let mut expected = ["a", "b"].iter().cycle();
    for _ in 0..20 {
        assert_eq!(expected.next().unwrap(), queue.select().unwrap().data());
    }
}

#[cfg(feature = "blocking")]
#[test]
fn complex_test() {
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
