use std::{
    collections::{HashMap, VecDeque},
    ops::SubAssign,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

use datastructurs::{
    intrusive_linked_list::{IntrusiveList, ListLink},
    trace,
};
use datastructurs_macros::IntoIntrusiveList;

struct State {
    name: String,
    fun_number: i64,
}

type SharedState = Arc<RwLock<State>>;
type Result<T> = std::result::Result<T, String>;
type WorkItem<T> = Box<dyn FnOnce(SharedState) -> Result<T>>;

#[derive(IntoIntrusiveList)]
struct Task<T> {
    id: usize,
    work: Option<WorkItem<T>>,
    #[accessor(AccReady)]
    link_ready: ListLink,
    #[accessor(AccProgress)]
    link_in_progress: ListLink,
    #[accessor(AccPrio)]
    link_priority: ListLink,
}

struct WorkProvider<Res> {
    state: SharedState,
    results: HashMap<usize, Result<Res>>,
    threads: Vec<JoinHandle<Result<()>>>,
    list_priority: IntrusiveList<Task<Res>, AccPrio>,
    list_in_progress: IntrusiveList<Task<Res>, AccProgress>,
    list_ready: IntrusiveList<Task<Res>, AccReady>,
    tasks: VecDeque<Task<Res>>,
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
            tasks: VecDeque::new(),
            state,
            results,
            threads: Vec::new(),
            list_priority: Default::default(),
            list_in_progress: Default::default(),
            list_ready: Default::default(),
            next_id: Default::default(),
        };
        let shared_wp = Arc::new(RwLock::new(wp));

        for tid in 0..2 {
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
        let id = self.next_id();
        let task = Task {
            id,
            work: Some(work),
            link_in_progress: Default::default(),
            link_ready: Default::default(),
            link_priority: Default::default(),
        };
        self.tasks.push_front(task);
        self.list_ready.push_back(&mut self.tasks[0]);
        if priority {
            self.list_priority.push_back(&mut self.tasks[0]);
        }
    }

    #[must_use]
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1usize;
        id
    }

    fn get_work(&mut self) -> Option<&mut Task<Res>> {
        if let Some(prio_job) = self.list_priority.pop_front() {
            Some(prio_job)
        } else if let Some(job) = self.list_ready.pop_front() {
            Some(job)
        } else {
            None
        }
    }

    fn submit(&mut self, id: usize, res: Result<Res>) {
        if let Some(_res) = self.results.insert(id, res) {
            panic!("Result duplicate: {id}")
        }
    }

    pub fn get_results(&self) -> &HashMap<usize, Result<Res>> {
        &self.results
    }

    pub fn is_done(&self) -> bool {
        self.list_ready.is_empty() && self.list_in_progress.is_empty()
    }

    pub fn keep_running(&self) -> bool {
        true
    }

    pub fn get_state(&self) -> SharedState {
        self.state.clone()
    }

    fn worker_thread_main(wp: Arc<RwLock<WorkProvider<Res>>>, tid: usize) -> Result<()> {
        macro_rules! thread_trace {
            ($($stuff:tt)+) => {
                println!("datastructu_rs::{}::{}::t{tid}: {}", file!(), line!(),format_args!($($stuff)+))
            };
        }

        while wp.read().unwrap().keep_running() {
            thread_trace!("Getting work");
            let mut wp_lock = wp.write().unwrap();
            let task = match wp_lock.get_work() {
                Some(stuff) => stuff,
                None => {
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

fn shit_rng(mut seed: i64) -> i64 {
    seed ^= 19314809;
    seed ^ 41752021957
}

fn main() {
    let wp: Arc<RwLock<WorkProvider<f64>>> = WorkProvider::new();

    for i in 0..10 {
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
                    let state = state.read().unwrap();
                    Ok(shit_rng(state.fun_number) as f64 / 18.4)
                }),
                true,
            );
        }
    }

    println!("Waiting for completion");
    while !wp.read().unwrap().is_done() {
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

    println!("{:=^80}", "RESULTS");
    for (id, res) in wp.read().unwrap().results.iter() {
        println!("{id:06} | {res:?}")
    }
}

unsafe impl<T: Send> Send for Task<T> {}
unsafe impl<T: Sync> Sync for Task<T> {}
unsafe impl<T: Send> Send for WorkProvider<T> {}
