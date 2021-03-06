use swarm::prelude::Dht;
use tokio::runtime::Builder;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use std::error::Error;
use std::hash::Hasher;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub mod coalesce;
pub mod fill;
pub mod split;
pub mod store;
pub mod open;

pub struct TaskHandle {
    completed_count: Arc<AtomicU32>,
    running: Arc<AtomicBool>,
    skipped_count: Arc<AtomicU32>,
    total_count: Arc<AtomicU32>,
}

impl TaskHandle {
    pub fn completed_count(&self) -> u32 {
        self.completed_count.load(Ordering::SeqCst)
    }

    pub fn running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn skipped_count(&self) -> u32 {
        self.skipped_count.load(Ordering::SeqCst)
    }

    pub fn total_count(&self) -> u32 {
        self.total_count.load(Ordering::SeqCst)
    }
}

pub struct TaskManager {
    tasks: HashMap<u64, TaskHandle>,
}

impl TaskManager {
    pub fn new() -> TaskManager {
        TaskManager {
            tasks: HashMap::new(),
        }
    }

    pub fn clear(&mut self) -> Result<(), Box<dyn Error>> {
        // retrieve list of complete ds    
        let complete_ids: Vec<u64> = self.tasks.iter()
            .filter(|(_, task_handle)| !task_handle.running())
            .map(|(id, _)| id.clone())
            .collect();

        // remove complete ids
        for complete_id in complete_ids.iter() {
            self.tasks.remove(complete_id);
        }

        Ok(())
    }

    pub fn iter(&self) -> Iter<u64, TaskHandle> {
        self.tasks.iter()
    }

    pub fn register(&mut self, task_handle: TaskHandle,
            task_id: Option<u64>) -> Result<u64, Box<dyn Error>> {
        // initialize task id
        let task_id = match task_id {
            Some(task_id) => task_id,
            None => rand::random::<u64>(),
        };

        // add TaskHandle to map
        info!("registering task [id={}]", task_id);
        self.tasks.insert(task_id, task_handle);

        // return task id
        Ok(task_id)
    }
}

#[tonic::async_trait]
pub trait Task<T: 'static + std::fmt::Debug + Send + Sync> {
    fn process(&self, record: &T) -> Result<(), Box<dyn Error>>;
    async fn records(&self) -> Result<Vec<T>, Box<dyn Error>>;

    fn start(self: Arc<Self>, thread_count: u8) 
            -> Result<TaskHandle, Box<dyn Error>>
            where Self: 'static + Send + Sync {
        info!("starting task [thread_count={}]", thread_count);
            
        // initialize instance variables
        let completed_count = Arc::new(AtomicU32::new(0));
        let running = Arc::new(AtomicBool::new(true));
        let skipped_count = Arc::new(AtomicU32::new(0));
        let total_count = Arc::new(AtomicU32::new(0));

        // initialize record channel
        let (sender, receiver) = crossbeam_channel::bounded(256);

        // start worker threads
        let mut join_handles = Vec::new();
        for _ in 0..thread_count {
            let completed_count = completed_count.clone();
            let skipped_count = skipped_count.clone();
            let receiver = receiver.clone();
            let self_clone = self.clone();

            let join_handle = std::thread::spawn(move || {
                // iterate over records
                loop {
                    // fetch next record
                    let record: T = match receiver.recv() {
                        Ok(record) => record,
                        Err(_) => break,
                    };

                    // process record
                    let result = self_clone.process(&record);

                    // process result
                    match result {
                        Ok(_) => completed_count.fetch_add(1,
                            Ordering::SeqCst),
                        Err(e) => {
                            warn!("skipping record '{:?}': {}",
                                record, e);
                            skipped_count.fetch_add(1, Ordering::SeqCst)
                        },
                    };
                }
            });

            join_handles.push(join_handle);
        }

        // initialize TaskHandle
        let task_handle = TaskHandle {
            completed_count: completed_count,
            skipped_count: skipped_count,
            running: running.clone(),
            total_count: total_count.clone(),
        };

        // start management thread
        let _ = std::thread::spawn(move || {
            // compute processing records
            let mut runtime = match Builder::new()
                    .basic_scheduler().enable_all().build() {
                Ok(runtime) => runtime,
                Err(e) => {
                    warn!("task failed to initialize runtime: {}", e);
                    running.store(false, Ordering::SeqCst);
                    return;
                },
            };

            let records = match runtime.block_on(self.records()) {
                Ok(records) => records,
                Err(e) => {
                    warn!("task failed to compile records: {}", e);
                    running.store(false, Ordering::SeqCst);
                    return;
                },
            };

            total_count.store(records.len() as u32, Ordering::SeqCst);

            // add items to pipeline
            debug!("registering records [count={}]", records.len());
            for record in records {
                if let Err(e) = sender.send(record) {
                    warn!("task failed to send record: {}", e);
                    break;
                }
            }
 
            // drop sender to signal worker threads
            drop(sender);

            // join worker threads
            for join_handle in join_handles {
                if let Err(e) = join_handle.join() {
                    warn!("task failed to join worker: {:?}", e);
                }
            }

            // complete TaskHandle
            running.store(false, Ordering::SeqCst);
        });

        Ok(task_handle)
    }
}

fn dht_lookup(dht: &Arc<Dht>, dht_key_length: i8,
        geocode: &str) -> Result<SocketAddr, Box<dyn Error>> {
    // compute dht geocode using dht_key_length
    let geocode = match dht_key_length {
        0 => geocode,
        x if x > 0 && x < geocode.len() as i8 =>
            &geocode[x as usize..],
        x if x < 0 && x > (-1 * geocode.len() as i8) =>
            &geocode[..(geocode.len() as i8 + x) as usize],
        _ => return Err(format!("dht key length '{}' invalid for '{}'",
                dht_key_length, geocode).into()),
    };

    // compute geocode hash
    let mut hasher = DefaultHasher::new();
    hasher.write(geocode.as_bytes());
    let hash = hasher.finish();

    // discover hash location
    match dht.locate(hash) {
        Some(node) => Ok(SocketAddr::new(node.get_ip_address().clone(),
            node.get_metadata("xfer_port").unwrap().parse::<u16>()?)),
        None => Err(format!("no dht node for hash {}", hash).into()),
    }
}
