use std::{
    collections::HashMap,
    ops::{AddAssign, SubAssign},
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

use datastructurs::{
    intrusive_linked_list::{IntrusiveList, ListLink},
    trace,
};
use datastructurs_macros::IntoIntrusiveList;

#[derive(Debug)]
struct State {
    #[allow(unused)]
    name: String,
    fun_number: i64,
}

const POW_2_31: i64 = 2i64.pow(31);

type SharedState = Arc<RwLock<State>>;
type Result<T> = std::result::Result<T, String>;
type WorkItem<T> = Box<dyn FnOnce(SharedState) -> Result<T>>;

#[derive(IntoIntrusiveList)]
struct Task<T> {
    id: usize,
    work: Option<WorkItem<T>>,
    #[accessor(AccReady)]
    link_ready: ListLink,
    #[accessor(AccPrio)]
    link_priority: ListLink,
}

#[derive(Debug)]
struct WorkProvider<Res> {
    state: SharedState,
    results: HashMap<usize, Result<Res>>,
    threads: Vec<JoinHandle<Result<()>>>,
    list_priority: IntrusiveList<Task<Res>, AccPrio>,
    list_ready: IntrusiveList<Task<Res>, AccReady>,
    next_id: usize,
}

impl<Res: Send + Sync + 'static> WorkProvider<Res> {
    pub fn new() -> Arc<RwLock<Self>> {
        let state = SharedState::new(RwLock::new(State {
            name: "gündriel".to_string(),
            fun_number: 3,
        }));
        let results = HashMap::new();

        let wp = Self {
            state,
            results,
            threads: Vec::new(),
            list_priority: Default::default(),
            list_ready: Default::default(),
            next_id: Default::default(),
        };
        let shared_wp = Arc::new(RwLock::new(wp));

        for tid in 0..1 {
            let wp = shared_wp.clone();
            shared_wp.write().unwrap().threads.push(
                thread::Builder::new()
                    .name(format!("{tid}"))
                    .spawn(move || Self::worker_thread_main(wp, tid))
                    .expect("could not spawn thread"),
            )
        }

        shared_wp
    }

    pub fn add_work(&mut self, work: WorkItem<Res>, priority: bool) {
        trace!(
            "Before add: ready.len={}, ready.head={:?}",
            self.list_ready.len(),
            self.list_ready.head
        );
        let id = self.next_id();
        let mut task = Box::new(Task {
            id,
            work: Some(work),
            link_ready: Default::default(),
            link_priority: Default::default(),
        });
        if priority {
            self.list_priority.push_back(&mut task);
        }
        self.list_ready.push_back(task);
        trace!(
            "After add: ready.len={}, ready.head={:?}",
            self.list_ready.len(),
            self.list_ready.head
        );
    }

    #[must_use]
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1usize;
        id
    }

    fn get_work(&mut self) -> Option<&mut Task<Res>> {
        trace!(
            "Before get_work: ready.len={}, ready.head={:?}",
            self.list_ready.len(),
            self.list_ready.head
        );
        let rlen = self.list_ready.len();
        let plen = self.list_priority.len();

        // BUG: pop sometimes returns None even if there are elements inside?!

        if let Some(prio_job) = self.list_priority.pop_front() {
            // self.list_ready.remove(prio_job);
            trace!("get_work: prio=true");
            Some(prio_job)
        } else if let Some(job) = self.list_ready.pop_front() {
            trace!("get_work: prio=false");
            Some(job)
        } else {
            trace!("get_work: list_ready and list_priority were empty!");
            debug_assert_eq!(rlen, 0, "They were not actually empty!");
            debug_assert_eq!(plen, 0, "They were not actually empty!");
            None
        }
    }

    fn submit(&mut self, id: usize, res: Result<Res>) {
        if let Some(_res) = self.results.insert(id, res) {
            panic!("Result duplicate: {id}")
        }
    }

    pub fn results(&self) -> &HashMap<usize, Result<Res>> {
        &self.results
    }

    pub fn is_done(&self) -> bool {
        self.list_ready.is_empty() && self.threads.iter().all(|th| th.is_finished())
    }

    pub fn keep_running(&self) -> bool {
        !self.is_done()
    }

    pub fn get_state(&self) -> SharedState {
        self.state.clone()
    }

    fn worker_thread_main(wp: Arc<RwLock<WorkProvider<Res>>>, tid: usize) -> Result<()> {
        #[cfg(debug_assertions)]
        macro_rules! thread_trace {
            ($($stuff:tt)+) => {
                println!("datastructu_rs::{}::{}::t{tid}: {}", file!(), line!(),format_args!($($stuff)+))
            };
        }
        #[cfg(not(debug_assertions))]
        macro_rules! thread_trace {
            ($($stuff:tt)+) => {
                ()
            };
        }

        while wp.read().unwrap().keep_running() {
            let mut wp_lock = wp.write().unwrap();
            thread_trace!("Getting work");
            let task = match wp_lock.get_work() {
                Some(task) if task.work.is_some() => task,
                _ => {
                    drop(wp_lock);
                    thread_trace!("No work available");
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    continue;
                }
            };
            let id = task.id;
            let work: Box<dyn FnOnce(SharedState) -> Result<Res>> =
                task.work.take().expect("work was already taken");
            drop(wp_lock);
            let shared_state = wp.read().unwrap().get_state();
            thread_trace!("Running work {id}");
            let res: Result<Res> = exec_work::<_, Res>(work, shared_state);
            thread_trace!("Submitting work for {id}");
            wp.write().unwrap().submit(id, res);
        }
        Ok(())
    }
}

fn exec_work<F, Res>(f: F, state: SharedState) -> Result<Res>
where
    F: FnOnce(SharedState) -> Result<Res>,
{
    f(state)
}

fn new_work<T, F>(f: F) -> WorkItem<T>
where
    F: FnMut(SharedState) -> Result<T>,
    F: 'static,
{
    Box::new(f)
}

fn shit_rng(seed: i64) -> i64 {
    (65539 * seed) % POW_2_31
}

fn main() {
    let wp: Arc<RwLock<WorkProvider<f64>>> = WorkProvider::new();

    println!("Set up work");
    for i in 0..40 {
        queue_work(i, wp.clone());
    }
    trace!("{}", wp.read().unwrap().list_ready.debug_nodes());

    println!("Waiting for completion");
    let mut i = 0;
    while !wp.read().unwrap().is_done() {
        std::thread::sleep(std::time::Duration::from_millis(40));
        if i % 10 == 0 {
            // queue_work(i, wp.clone());
            trace!("work ready: {}", wp.read().unwrap().list_ready.len());
        }
        i += 1;
    }

    println!("{:=^80}", "RESULTS");
    for (id, res) in wp.read().unwrap().results().iter() {
        println!("{id:06} | {res:?}")
    }
}

fn queue_work(i: usize, wp: Arc<RwLock<WorkProvider<f64>>>) {
    if i % 19 == 0 {
        wp.write().unwrap().add_work(
            new_work(|state| {
                let mut state = state.write().unwrap();
                state.fun_number = state.fun_number.wrapping_mul(13);
                state.fun_number.sub_assign(3);
                Ok(state.fun_number as f64)
            }),
            true,
        );
    } else {
        wp.write().unwrap().add_work(
            new_work(|state| {
                let mut state_lock = state.write().unwrap();
                state_lock.fun_number.add_assign(1);
                drop(state_lock);
                Ok(shit_rng(state.read().unwrap().fun_number) as f64 / 18.4)
            }),
            true,
        );
    }
}

impl<T> std::fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("work", &self.work.is_some())
            .field("link_ready", &self.link_ready)
            .field("link_priority", &self.link_priority)
            .finish()
    }
}

unsafe impl<T: Send> Send for Task<T> {}
unsafe impl<T: Sync> Sync for Task<T> {}
unsafe impl<T: Send> Send for WorkProvider<T> {}
