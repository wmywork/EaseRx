use crate::tracing_setup::tracing_init;
use easerx::{State, StateStore};
use futures_signals::signal::SignalExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::time::Instant;
use tracing::{info, warn};

mod tracing_setup;

#[derive(Debug, Clone)]
struct CollectionState {
    vec: Vec<usize>,
    arc_vec: Arc<Mutex<Vec<usize>>>,
    str: String,
    arc_str: Arc<RwLock<String>>,
    map: HashMap<usize, usize>,
    arc_map: Arc<RwLock<HashMap<usize, usize>>>,
}

impl State for CollectionState {}

impl Default for CollectionState {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
            arc_vec: Arc::new(Mutex::new(Vec::default())),
            map: HashMap::default(),
            arc_map: Arc::new(RwLock::new(HashMap::default())),
            str: String::with_capacity(TEST_LEN),
            arc_str: Arc::new(RwLock::new(String::with_capacity(TEST_LEN))),
        }
    }
}

impl CollectionState {
    fn arc_vec_push(self, x: usize) -> CollectionState {
        self.arc_vec.lock().unwrap().push(x);
        self
    }

    fn arc_vec_len(&self) -> usize {
        self.arc_vec.lock().unwrap().len()
    }

    fn vec_push(mut self, x: usize) -> CollectionState {
        self.vec.push(x);
        self
    }

    fn vec_len(&self) -> usize {
        self.vec.len()
    }

    fn arc_str_push(self, value: char) -> Self {
        self.arc_str.write().unwrap().push(value);
        self
    }
    fn arc_str_len(&self) -> usize {
        self.arc_str.read().unwrap().len()
    }

    fn str_push(mut self, value: char) -> Self {
        self.str.push(value);
        self
    }

    fn str_len(&self) -> usize {
        self.str.len()
    }

    fn arc_map_insert(self, key: usize, value: usize) -> CollectionState {
        self.arc_map.write().unwrap().insert(key, value);
        self
    }

    fn arc_map_len(&self) -> usize {
        self.arc_map.read().unwrap().len()
    }

    fn map_insert(mut self, key: usize, value: usize) -> CollectionState {
        self.map.insert(key, value);
        self
    }

    fn map_len(&self) -> usize {
        self.map.len()
    }
}
const TEST_LEN: usize = 30_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_init();

    info!("==========================================");
    warn!("Test a huge collection: arc_vec (recommended)");
    //Create store
    let store = Arc::new(StateStore::new(CollectionState::default()));

    let last_tick = Instant::now();
    for i in 0..TEST_LEN {
        store.set_state(move |state| state.arc_vec_push(i))?;
    }

    store
        .to_signal()
        .stop_if(|state| state.arc_vec_len() == TEST_LEN)
        .for_each(|state| {
            if state.arc_vec_len() == TEST_LEN {
                info!("  Main thread | arc_vec len is :{:?}", TEST_LEN);
            };
            async {}
        })
        .await;

    let elapsed = last_tick.elapsed();
    info!("  Main thread | elapsed is :{:?}", elapsed);

    info!("==========================================");
    warn!("Test a huge collection: vec (Not recommended)");
    //Create store
    let store = Arc::new(StateStore::new(CollectionState::default()));

    let last_tick = Instant::now();
    for i in 0..TEST_LEN {
        store.set_state(move |state| state.vec_push(i))?;
    }

    store
        .to_signal()
        .stop_if(|state| state.vec_len() == TEST_LEN)
        .for_each(|state| {
            if state.vec_len() == TEST_LEN {
                info!("  Main thread | vec len is :{:?}", TEST_LEN);
            };
            async {}
        })
        .await;

    let elapsed = last_tick.elapsed();
    info!("  Main thread | elapsed is :{:?}", elapsed);

    info!("==========================================");
    warn!("Test a huge String: arc_str (recommended)");
    //Create store
    let store = Arc::new(StateStore::new(CollectionState::default()));

    let last_tick = Instant::now();
    for _ in 0..TEST_LEN {
        store.set_state(move |state| state.arc_str_push('a'))?;
        tokio::task::yield_now().await;
    }

    store
        .to_signal()
        .stop_if(|state| state.arc_str_len() == TEST_LEN)
        .for_each(|state| {
            if state.arc_str_len() == TEST_LEN {
                info!("  Main thread | arc_str len is :{:?}", TEST_LEN);
            };
            async {}
        })
        .await;

    let elapsed = last_tick.elapsed();
    info!("  Main thread | elapsed is :{:?}", elapsed);

    info!("==========================================");
    warn!("Test a huge String: str (Not recommended)");
    //Create store
    let store = Arc::new(StateStore::new(CollectionState::default()));

    let last_tick = Instant::now();
    for _i in 0..TEST_LEN {
        store.set_state(move |state| state.str_push('b'))?;
        tokio::task::yield_now().await;
    }

    store
        .to_signal()
        .stop_if(|state| state.str_len() == TEST_LEN)
        .for_each(|state| {
            if state.str_len() == TEST_LEN {
                info!("  Main thread | str len is :{:?}", TEST_LEN);
            };
            async {}
        })
        .await;

    let elapsed = last_tick.elapsed();
    info!("  Main thread | elapsed is :{:?}", elapsed);

    info!("==========================================");
    warn!("Test a huge collection: arc_map (recommended)");
    //Create store
    let store = Arc::new(StateStore::new(CollectionState::default()));

    let last_tick = Instant::now();
    for i in 0..TEST_LEN {
        store.set_state(move |state| state.arc_map_insert(i, i))?;
    }

    store
        .to_signal()
        .stop_if(|state| state.arc_map_len() == TEST_LEN)
        .for_each(|state| {
            if state.arc_map_len() == TEST_LEN {
                info!("  Main thread | arc_map len is :{:?}", TEST_LEN);
            };
            async {}
        })
        .await;

    let elapsed = last_tick.elapsed();
    info!("  Main thread | elapsed is :{:?}", elapsed);

    info!("==========================================");
    warn!("Test a huge collection: map (Not recommended)");
    //Create store
    let store = Arc::new(StateStore::new(CollectionState::default()));

    let last_tick = Instant::now();
    for i in 0..TEST_LEN {
        store.set_state(move |state| state.map_insert(i, i))?;
    }

    store
        .to_signal()
        .stop_if(|state| state.map_len() == TEST_LEN)
        .for_each(|state| {
            if state.map_len() == TEST_LEN {
                info!("  Main thread | map len is :{:?}", TEST_LEN);
            };
            async {}
        })
        .await;

    let elapsed = last_tick.elapsed();
    info!("  Main thread | elapsed is :{:?}", elapsed);

    info!("==========================================");
    info!("  Main thread | Finish");
    Ok(())
}
