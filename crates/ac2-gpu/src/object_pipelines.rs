//! The object pipelines, compiled off the frame thread. A model's pixel shaders are queued when it
//! is uploaded and each fixed-function state when it is first laid out; a worker translates and
//! compiles them in order (through the device's pipeline cache when it has one). Retail D3D
//! created its few fixed-function states on demand; here the translated pixel programs make
//! every new material a driver compile, which would stall the frame where it first appears.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use crate::object_state::PipelineKey;
use crate::Gpu;

enum Job {
    /// Module `id` is the object shader with this `shade` function.
    Module(u32, String),
    Pipeline(PipelineKey),
}

/// Object pipelines by state: ready, or compiling on the worker.
pub(crate) struct PipelineCompiler {
    /// `None` once dropped, which ends the worker.
    jobs: Option<Sender<Job>>,
    /// Joined on drop, so the worker's device handles never outlive the renderer's.
    worker: Option<std::thread::JoinHandle<()>>,
    done: Receiver<(PipelineKey, wgpu::RenderPipeline)>,
    ready: HashMap<PipelineKey, wgpu::RenderPipeline>,
    pending: HashSet<PipelineKey>,
}

impl PipelineCompiler {
    /// A worker over `gpu`'s device whose module 0 is `stages` (the stage cascade).
    pub(crate) fn new(
        gpu: &Gpu,
        layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
        stages: wgpu::ShaderModule,
    ) -> Self {
        let (jobs, queue) = channel::<Job>();
        let (finished, done) = channel();
        let device = gpu.device.clone();
        let layout = layout.clone();
        let cache = gpu.pipeline_cache().cloned();
        let worker = std::thread::Builder::new()
            .name("ac2 pipelines".into())
            .spawn(move || {
                let mut modules = vec![stages];
                for job in queue {
                    match job {
                        Job::Module(id, shade) => {
                            debug_assert_eq!(id as usize, modules.len());
                            modules.push(crate::object::object_module(&device, &shade));
                        }
                        Job::Pipeline(key) => {
                            let module = &modules[key.program as usize];
                            let pipeline =
                                key.create(&device, &layout, module, format, cache.as_ref());
                            if finished.send((key, pipeline)).is_err() {
                                return;
                            }
                        }
                    }
                }
            })
            .expect("spawn the pipeline compiler");
        Self {
            jobs: Some(jobs),
            worker: Some(worker),
            done,
            ready: HashMap::new(),
            pending: HashSet::new(),
        }
    }

    /// Queues module `id`'s translation; pipelines over it queue after it.
    pub(crate) fn add_module(&self, id: u32, shade: String) {
        self.send(Job::Module(id, shade));
    }

    fn send(&self, job: Job) {
        self.jobs
            .as_ref()
            .and_then(|jobs| jobs.send(job).ok())
            .expect("the pipeline compiler stopped");
    }

    /// Whether `key`'s pipeline is ready; queues it when it is neither ready nor queued.
    pub(crate) fn request(&mut self, key: PipelineKey) -> bool {
        if self.ready.contains_key(&key) {
            return true;
        }
        if self.pending.insert(key) {
            self.send(Job::Pipeline(key));
        }
        false
    }

    /// Takes the pipelines the worker finished.
    pub(crate) fn collect(&mut self) {
        while let Ok((key, pipeline)) = self.done.try_recv() {
            self.finish(key, pipeline);
        }
    }

    /// Waits until every queued pipeline is ready.
    pub(crate) fn wait_all(&mut self) {
        while !self.pending.is_empty() {
            let (key, pipeline) = self
                .done
                .recv_timeout(Duration::from_secs(60))
                .expect("the pipeline compiler stopped");
            self.finish(key, pipeline);
        }
    }

    pub(crate) fn is_ready(&self, key: &PipelineKey) -> bool {
        self.ready.contains_key(key)
    }

    pub(crate) fn pipeline(&self, key: &PipelineKey) -> &wgpu::RenderPipeline {
        &self.ready[key]
    }

    /// Whether any pipeline is still compiling.
    pub(crate) fn busy(&self) -> bool {
        !self.pending.is_empty()
    }

    fn finish(&mut self, key: PipelineKey, pipeline: wgpu::RenderPipeline) {
        self.pending.remove(&key);
        self.ready.insert(key, pipeline);
    }
}

impl Drop for PipelineCompiler {
    fn drop(&mut self) {
        self.jobs = None;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
