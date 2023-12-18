use crate::instance::Instance;
use num::integer::lcm;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct WrrQueue<T: PartialEq> {
    instance_list: Vec<Instance<T>>,
    cur_idx: AtomicUsize,
    #[cfg(feature = "tokio")]
    select_queue: tokio::sync::RwLock<Vec<usize>>,
    #[cfg(feature = "blocking")]
    select_queue: std::sync::RwLock<Vec<usize>>,
}

impl<T: PartialEq> Default for WrrQueue<T> {
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

    fn insert_uncalculated(&mut self, instance: Instance<T>) -> bool
    where
        T: PartialEq,
    {
        if self.instance_list.contains(&instance) {
            false
        } else {
            self.instance_list.push(instance);
            true
        }
    }
}

#[cfg(feature = "tokio")]
impl<T: PartialEq> WrrQueue<T> {
    pub async fn insert(&mut self, instance: impl Into<Instance<T>>) -> bool {
        let res = self.insert_uncalculated(instance.into());
        self.recalculate_queue().await;
        res
    }

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

    async fn recalculate_queue(&mut self) {
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
    pub fn insert(&mut self, instance: impl Into<Instance<T>>) -> bool {
        let res = self.insert_uncalculated(instance.into());
        self.recalculate_queue();
        res
    }

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
