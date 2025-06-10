use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::Thread,
};

use datastructurs::intrusive_linked_list::{IntrusiveList, ListLink};
use datastructurs_macros::IntoIntrusiveList;

struct State {
    name: String,
    fun_number: i64,
}

type SharedState = Arc<Mutex<State>>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type WorkItem<T> = Box<dyn FnOnce(SharedState) -> Result<T>>;

#[derive(IntoIntrusiveList)]
struct Task<T> {
    work: WorkItem<T>,
    #[accessor(AccReady)]
    link_ready: ListLink,
    #[accessor(AccProgress)]
    link_in_progress: ListLink,
    #[accessor(AccPrio)]
    link_priority: ListLink,
}

struct WorkProvider<Id: Ord, Res> {
    state: SharedState,
    results: HashMap<Id, Result<Res>>,
    threads: Vec<Thread>,
    list_priority: IntrusiveList<Task<Result<Res>>, AccPrio>,
    list_in_progress: IntrusiveList<Task<Result<Res>>, AccProgress>,
    list_ready: IntrusiveList<Task<Result<Res>>, AccReady>,
}

impl<Id: Ord, Res> WorkProvider<Id, Res> {
    pub fn new() -> Self {
        todo!()
    }
    pub fn add_work(&self, work: WorkItem<Res>, priority: bool) {
        todo!()
    }
    fn get_work(&self) -> Task<WorkItem<Res>> {
        todo!()
    }
    fn submit(&self, id: Id, res: Result<Res>) {
        todo!()
    }
    pub fn get_results(&self) -> &HashMap<Id, Result<Res>> {
        &self.results
    }
    pub fn is_done(&self) -> bool {
        self.list_ready.is_empty() && self.list_in_progress.is_empty()
    }
}

fn new_work<T, F>(f: F) -> WorkItem<T>
where
    F: FnOnce(SharedState) -> Result<T>,
{
    todo!()
}

fn main() {
    let mut wp: WorkProvider<u64, f64> = WorkProvider::new();

    for i in 0..10_000 {
        if i % 19 == 0 {
            wp.add_work(
                new_work(|state| {
                    let mut state = state.lock().unwrap();
                    state.fun_number *= 13;
                    Ok(state.fun_number as f64)
                }),
                true,
            );
        } else {
            ()
        }
    }
}
