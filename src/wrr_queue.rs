use crate::instance::Instance;
use log::error;
use num::integer::lcm;
use std::sync::atomic::{AtomicUsize, Ordering};

/// weighted round robin queue struct
///
/// WRR queue, each time new instance is inserted, balance queue need to be recalculated.
/// So minimizing the insert operation can improve performance.
///
/// `select` method requires only an atomic usize and a Read access to the RwLock.
/// There should be of no runtime performance issue.
///
/// example:
///
/// ```ignore
/// use async_wrr_queue::{WrrQueue, Instance};
/// use std::num::NonZeroUsize;
///
/// let mut queue = WrrQueue::new().insert_many([("data1", 1), ("data2", 2), ("data3", 3)]).await;
/// queue.insert(Instance::new_with_weight("data4", NonZeroUsize::new(4).unwrap())).await;
///
/// let selected1 = queue.select();
/// let selected2 = queue.select();
/// let selected3 = queue.select();
/// ```
pub struct WrrQueue<T: PartialEq> {
    instance_list: Vec<Instance<T>>,
    cur_idx: AtomicUsize,
    #[cfg(feature = "tokio")]
    select_queue: tokio::sync::RwLock<Vec<usize>>,
    #[cfg(feature = "blocking")]
    select_queue: std::sync::RwLock<Vec<usize>>,
}

impl<T: PartialEq> Default for WrrQueue<T> {
    /// create a default WRR Queue, with no data
    fn default() -> Self {
        WrrQueue {
            instance_list: Vec::new(),
            cur_idx: AtomicUsize::new(0),

            #[cfg(feature = "tokio")]
            select_queue: tokio::sync::RwLock::new(Vec::new()),
            #[cfg(feature = "blocking")]
            select_queue: std::sync::RwLock::new(Vec::new()),
        }
    }
}

impl<T: PartialEq> WrrQueue<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert_uncalculated(&mut self, instance: Instance<T>) -> bool {
        if self.instance_list.contains(&instance) {
            false
        } else {
            self.instance_list.push(instance);
            true
        }
    }

    fn clear_instance_uncalculated(&mut self) {
        self.instance_list = Default::default();
        self.cur_idx = Default::default();
        self.select_queue = Default::default();
    }

    fn delete_uncalculated(&mut self, instance: Instance<T>) -> bool {
        if self.instance_list.contains(&instance) {
            false
        } else {
            let index = self
                .instance_list
                .iter()
                .position(|x| *x == instance)
                .unwrap();
            self.instance_list.remove(index);
            true
        }
    }
}

#[cfg(feature = "tokio")]
impl<T: PartialEq> WrrQueue<T> {
    /// insert a new instance, and re-calculate request queue
    pub async fn insert(&mut self, instance: impl Into<Instance<T>>) -> bool {
        let res = self.insert_uncalculated(instance.into());
        self.recalculate_queue().await;
        res
    }

    /// insert a new instance vec, and re-calculate request queue
    /// recommended when have multiple instance to be inserted
    pub async fn insert_many<U>(&mut self, instance_list: impl Into<Vec<U>>) -> bool
    where
        T: PartialEq,
        U: Into<Instance<T>>,
    {
        let res = instance_list
            .into()
            .into_iter()
            .map(|i| self.insert_uncalculated(i.into()))
            .all(|t| t);
        self.recalculate_queue().await;
        res
    }

    /// return the selected instance, None if instance_list is empty
    /// NOTE: select operation used only atomic operation, and can be paralleled  
    pub async fn select(&mut self) -> Option<&Instance<T>> {
        if self.instance_list.is_empty() {
            None
        } else {
            let idx = self.cur_idx.fetch_add(1, Ordering::Relaxed);
            let read_lock = self.select_queue.read().await;
            let selected_seq_idx = idx % read_lock.len();
            let selected_instance_idx = read_lock.get(selected_seq_idx)?;
            self.instance_list.get(*selected_instance_idx)
        }
    }

    /// clear instance in the queue
    pub fn clear_instance(&mut self) {
        self.clear_instance_uncalculated();
    }

    /// delete certain instance
    pub async fn delete_instance(&mut self, instance: Instance<T>) -> bool {
        if self.delete_uncalculated(instance) {
            self.recalculate_queue().await;
            true
        } else {
            false
        }
    }

    async fn recalculate_queue(&mut self) {
        if self.instance_list.is_empty() {
            self.clear_instance();
            return;
        }
        let lcm = self
            .instance_list
            .iter()
            .map(Instance::weight)
            .fold(1usize, |acc, a| lcm(acc, a.get()));
        let mut queue = Vec::new();
        let weight_vec = self.instance_list.iter().fold(Vec::new(), |mut acc, a| {
            acc.push(a.weight().get());
            acc
        });
        let mut cur_weight_vec: Vec<isize> =
            weight_vec.clone().into_iter().map(|u| u as isize).collect();
        for _ in 0..=lcm {
            let selected = select_instance(&weight_vec, &mut cur_weight_vec);
            queue.push(selected);
        }

        let mut queue_lock = self.select_queue.write().await;
        queue_lock.clear();
        for i in queue {
            queue_lock.push(i);
        }
    }
}

#[cfg(feature = "blocking")]
impl<T: PartialEq> WrrQueue<T> {
    /// insert a new instance, and re-calculate request queue
    pub fn insert(&mut self, instance: impl Into<Instance<T>>) -> bool {
        let res = self.insert_uncalculated(instance.into());
        self.recalculate_queue();
        res
    }

    /// insert a new instance vec, and re-calculate request queue
    /// recommended when have multiple instance to be inserted
    pub fn insert_many<U>(&mut self, instance_list: impl Into<Vec<U>>) -> bool
    where
        T: PartialEq,
        U: Into<Instance<T>>,
    {
        let res = instance_list
            .into()
            .into_iter()
            .map(|i| self.insert_uncalculated(i.into()))
            .all(|t| t);
        self.recalculate_queue();
        res
    }

    /// return the selected instance, None if instance_list is empty
    /// NOTE: select operation used only atomic operation, and can be paralleled  
    pub fn select(&mut self) -> Option<&Instance<T>> {
        if self.instance_list.is_empty() {
            None
        } else {
            let idx = self.cur_idx.fetch_add(1, Ordering::Relaxed);
            let read_lock = self
                .select_queue
                .read()
                .expect("Read access acquired failed");
            let selected_seq_idx = idx % read_lock.len();
            let selected_instance_idx = read_lock.get(selected_seq_idx)?;
            self.instance_list.get(*selected_instance_idx)
        }
    }

    /// clear instance in the queue
    pub fn clear_instance(&mut self) {
        self.clear_instance_uncalculated();
    }

    /// delete certain instance
    pub fn delete_instance(&mut self, instance: Instance<T>) -> bool {
        if self.delete_uncalculated(instance) {
            self.recalculate_queue();
            true
        } else {
            false
        }
    }

    fn recalculate_queue(&mut self) {
        let lcm = self
            .instance_list
            .iter()
            .map(Instance::weight)
            .fold(1usize, |acc, a| lcm(acc, a.get()));
        let mut queue = Vec::new();
        let weight_vec = self.instance_list.iter().fold(Vec::new(), |mut acc, a| {
            acc.push(a.weight().get());
            acc
        });
        let mut cur_weight_vec: Vec<isize> =
            weight_vec.clone().into_iter().map(|u| u as isize).collect();
        for _ in 0..=lcm {
            let selected = select_instance(&weight_vec, &mut cur_weight_vec);
            queue.push(selected);
        }

        let mut queue_lock = self
            .select_queue
            .write()
            .expect("Write lock acquired failed");
        queue_lock.clear();
        for i in queue {
            queue_lock.push(i);
        }
    }
}

fn select_instance(weight_vec: &Vec<usize>, cur_weight: &mut [isize]) -> usize {
    if weight_vec.is_empty() {
        error!("failed to select an instance: instance list is empty");
        return 0;
    }
    let mut selected = 0;
    let mut acc = 0isize;
    for i in 0..weight_vec.len() {
        cur_weight[i] += weight_vec[i] as isize;
        acc += weight_vec[i] as isize;
        if cur_weight[selected] < cur_weight[i] {
            selected = i;
        }
    }
    cur_weight[selected] -= acc;
    selected
}
