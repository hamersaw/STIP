use image::ImageFormat;
use protobuf::{self, Image, ImageFormat as ProtoImageFormat, LoadFormat as ProtoLoadFormat, LoadReply, LoadRequest, SearchReply, SearchRequest, Task, TaskListReply, TaskListRequest, TaskShowReply, TaskShowRequest, DataManagement};
use swarm::prelude::Dht;
use tonic::{Request, Response, Status};

use crate::data::DataManager;
use crate::task::{TaskHandle, TaskManager, TaskStatus};
use crate::task::load::{LoadEarthExplorerTask, LoadFormat};

use std::sync::{Arc, RwLock};

pub struct DataManagementImpl {
    data_manager: Arc<DataManager>,
    dht: Arc<RwLock<Dht>>,
    task_manager: Arc<RwLock<TaskManager>>,
}

impl DataManagementImpl {
    pub fn new(data_manager: Arc<DataManager>, dht: Arc<RwLock<Dht>>,
            task_manager: Arc<RwLock<TaskManager>>) -> DataManagementImpl {
        DataManagementImpl {
            data_manager: data_manager,
            dht: dht,
            task_manager: task_manager,
        }
    }
}

#[tonic::async_trait]
impl DataManagement for DataManagementImpl {
    async fn load(&self, request: Request<LoadRequest>)
            -> Result<Response<LoadReply>, Status> {
        trace!("LoadDirectoryRequest: {:?}", request);
        let request = request.get_ref();

        // initialize task
        let image_format = match ProtoImageFormat
                ::from_i32(request.image_format).unwrap() {
            ProtoImageFormat::Jpeg => ImageFormat::Jpeg,
            ProtoImageFormat::Tiff => ImageFormat::Tiff,
        };

        let load_format = match ProtoLoadFormat
                ::from_i32(request.load_format).unwrap() {
            ProtoLoadFormat::Landsat => LoadFormat::Landsat,
            ProtoLoadFormat::Sentinel => LoadFormat::Sentinel,
        };

        let task = LoadEarthExplorerTask::new(self.dht.clone(),
            request.directory.clone(), request.file.clone(),
            image_format, load_format, request.precision as usize,
            request.thread_count as u8);

        // execute task using task manager
        let task_id = {
            let mut task_manager = self.task_manager.write().unwrap();
            task_manager.execute(task).unwrap() // TODO - handle error
        };

        // initialize reply
        let reply = LoadReply {
            task_id: task_id,
        };

        Ok(Response::new(reply))
    }

    async fn search(&self, request: Request<SearchRequest>)
            -> Result<Response<SearchReply>, Status> {
        trace!("SearchRequest: {:?}", request);
        let request = request.get_ref();

        // search for the requested images
        // TODO - handle error on search_images
        let images = self.data_manager.search_images(
                &request.geohash, &request.platform).unwrap().iter()
            .map(|x| Image {
                coverage: x.coverage,
                geohash: x.geohash.clone(),
                lat_min: x.lat_min,
                lat_max: x.lat_max,
                long_min: x.long_min,
                long_max: x.long_max,
                path: x.path.clone(),
                platform: x.platform.clone(),
                precision: x.precision as u32,
            }).collect();

        // initialize reply
        let reply = SearchReply {
            images: images,
        };

        Ok(Response::new(reply))
    }

    async fn task_list(&self, request: Request<TaskListRequest>)
            -> Result<Response<TaskListReply>, Status> {
        trace!("TaskListRequest: {:?}", request);

        // populate tasks from task_manager
        let mut tasks = Vec::new();
        {
            let task_manager = self.task_manager.read().unwrap();
            for (task_id, task_handle) in task_manager.iter() {
                // convert TaskHandle to protobuf
                let task = to_protobuf(*task_id, task_handle);

                // add to tasks
                tasks.push(task);
            }
        }

        // initialize reply
        let reply = TaskListReply {
            tasks: tasks,
        };

        Ok(Response::new(reply))
    }

    async fn task_show(&self, request: Request<TaskShowRequest>)
            -> Result<Response<TaskShowReply>, Status> {
        trace!("TaskShowRequest: {:?}", request);
        let request = request.get_ref();

        // populate task from task_manager
        let task = {
            let task_manager = self.task_manager.read().unwrap();
            match task_manager.get(&request.id) {
                None => None,
                Some(task_handle) =>
                    Some(to_protobuf(request.id, task_handle)),
            }
        };

        // initialize reply
        let reply = TaskShowReply {
            task: task,
        };

        Ok(Response::new(reply))
    }
}

fn to_protobuf(task_id: u64, task_handle: &Arc<RwLock<TaskHandle>>) -> Task {
    // get read lock on TaskHandle
    let task_handle = task_handle.read().unwrap();
    
    // compile task status
    let status = match task_handle.get_status() {
        TaskStatus::Complete => protobuf::TaskStatus::Complete,
        TaskStatus::Failure(_) => protobuf::TaskStatus::Failure,
        TaskStatus::Running => protobuf::TaskStatus::Running,
    };

    // initialize task protobuf
    Task {
        id: task_id,
        completion_percent: task_handle
            .get_completion_percent().unwrap_or(0.0),
        status: status as i32,
    }
}
