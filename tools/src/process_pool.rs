use std::os::unix::io::RawFd;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use serde::{Serialize, de::DeserializeOwned};

pub struct ProcessPool<Req, Resp> {
    workers: Mutex<Vec<Sender<Req>>>,
    collector_rx: Mutex<Receiver<(usize, Result<Resp, String>)>>,
    child_pids: Vec<libc::pid_t>,
}

fn write_msg<T: Serialize, W: std::io::Write>(writer: &mut W, msg: &T) -> std::io::Result<()> {
    let bytes = bincode::serialize(msg).map_err(std::io::Error::other)?;
    let len = bytes.len() as u64;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}

fn read_msg<T: DeserializeOwned, R: std::io::Read>(reader: &mut R) -> std::io::Result<T> {
    let mut len_bytes = [0u8; 8];
    reader.read_exact(&mut len_bytes)?;
    let len = u64::from_le_bytes(len_bytes);
    let mut bytes = vec![0u8; len as usize];
    reader.read_exact(&mut bytes)?;
    let msg = bincode::deserialize(&bytes).map_err(std::io::Error::other)?;
    Ok(msg)
}


fn pin_to_core(core_id: usize) {
    unsafe {
        let mut cpu_set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_SET(core_id, &mut cpu_set);
        let ret = libc::sched_setaffinity(
            0,
            std::mem::size_of::<libc::cpu_set_t>(),
            &cpu_set,
        );
        if ret != 0 {
            log::warn!("Failed to pin process to core {}", core_id);
        } else {
            log::info!("Pinned process to core {}", core_id);
        }
    }
}

fn run_worker<Req, Resp, F>(read_fd: RawFd, write_fd: RawFd, mut f: F) -> !
where
    Req: DeserializeOwned,
    Resp: Serialize,
    F: FnMut(Req) -> Resp,
{
    use std::os::unix::io::FromRawFd;
    let mut reader = std::io::BufReader::new(unsafe { std::fs::File::from_raw_fd(read_fd) });
    let mut writer = std::io::BufWriter::new(unsafe { std::fs::File::from_raw_fd(write_fd) });

    loop {
        let req: Req = match read_msg(&mut reader) {
            Ok(r) => r,
            Err(_) => break,
        };
        let resp = f(req);
        if write_msg(&mut writer, &resp).is_err() {
            break;
        }
    }
    std::process::exit(0);
}

impl<Req, Resp> ProcessPool<Req, Resp>
where
    Req: Serialize + DeserializeOwned + Send + Clone + 'static,
    Resp: Serialize + DeserializeOwned + Send + 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(Req) -> Resp + Send + 'static,
    {
        let num_cpus = std::thread::available_parallelism().map_or(4, |n| n.get());
        log::info!("Starting ProcessPool with {} worker processes", num_cpus);

        let (collector_tx, collector_rx) = channel();
        let mut workers = Vec::new();
        let mut child_pids = Vec::new();

        let f = std::sync::Arc::new(std::sync::Mutex::new(f));

        for core_id in 0..num_cpus {
            let mut parent_to_child = [0; 2];
            let mut child_to_parent = [0; 2];
            unsafe {
                if libc::pipe(parent_to_child.as_mut_ptr()) < 0 {
                    panic!("Failed to create parent_to_child pipe");
                }
                if libc::pipe(child_to_parent.as_mut_ptr()) < 0 {
                    panic!("Failed to create child_to_parent pipe");
                }
            }

            let f_clone = f.clone();
            let pid = unsafe { libc::fork() };
            if pid < 0 {
                panic!("Failed to fork");
            } else if pid == 0 {
                // Child process
                unsafe {
                    libc::close(parent_to_child[1]);
                    libc::close(child_to_parent[0]);
                }
                pin_to_core(core_id);
                let mut handler = f_clone.lock().unwrap();
                run_worker(parent_to_child[0], child_to_parent[1], &mut *handler);
            } else {
                // Parent process
                unsafe {
                    libc::close(parent_to_child[0]);
                    libc::close(child_to_parent[1]);
                }
                child_pids.push(pid);

                let (tx, rx) = channel();
                let collector_tx = collector_tx.clone();

                let parent_write_fd = parent_to_child[1];
                let parent_read_fd = child_to_parent[0];

                std::thread::spawn(move || {
                    use std::os::unix::io::FromRawFd;
                    let mut writer = std::io::BufWriter::new(unsafe { std::fs::File::from_raw_fd(parent_write_fd) });
                    let mut reader = std::io::BufReader::new(unsafe { std::fs::File::from_raw_fd(parent_read_fd) });

                    loop {
                        let task = match rx.recv() {
                            Ok(t) => t,
                            Err(_) => break, // Sender dropped (parent dropped)
                        };

                        if let Err(e) = write_msg(&mut writer, &task) {
                            log::error!("Worker {} coordinator write error: {:?}", core_id, e);
                            let _ = collector_tx.send((core_id, Err(format!("Write error: {:?}", e))));
                            break;
                        }

                        match read_msg(&mut reader) {
                            Ok(resp) => {
                                let _ = collector_tx.send((core_id, Ok(resp)));
                            }
                            Err(e) => {
                                log::error!("Worker {} coordinator read error: {:?}", core_id, e);
                                let _ = collector_tx.send((core_id, Err(format!("Read error: {:?}", e))));
                                break;
                            }
                        }
                    }
                });

                workers.push(tx);
            }
        }

        Self {
            workers: Mutex::new(workers),
            collector_rx: Mutex::new(collector_rx),
            child_pids,
        }
    }

    pub fn num_workers(&self) -> usize {
        self.workers.lock().unwrap().len()
    }

    pub fn send(&self, worker_idx: usize, req: Req) -> Result<(), std::sync::mpsc::SendError<Req>> {
        self.workers.lock().unwrap()[worker_idx].send(req)
    }

    pub fn recv(&self) -> Result<(usize, Result<Resp, String>), std::sync::mpsc::RecvError> {
        self.collector_rx.lock().unwrap().recv()
    }

    pub fn execute(&self, requests: &[Req], mut on_progress: impl FnMut()) -> Vec<Result<Resp, String>> {
        let num_workers = self.num_workers();
        let mut free_workers: Vec<usize> = (0..num_workers).collect();
        let mut outcomes = Vec::new();
        outcomes.resize_with(requests.len(), || None);

        let mut next_task_idx = 0;
        let mut active_tasks = 0;
        let mut worker_task_idx = vec![None; num_workers];

        while next_task_idx < requests.len() || active_tasks > 0 {
            // Assign tasks to free workers
            while let Some(worker_idx) = free_workers.pop() {
                if next_task_idx >= requests.len() {
                    free_workers.push(worker_idx);
                    break;
                }
                worker_task_idx[worker_idx] = Some(next_task_idx);
                let send_res = self.send(worker_idx, requests[next_task_idx].clone());
                if send_res.is_ok() {
                    next_task_idx += 1;
                    active_tasks += 1;
                } else {
                    worker_task_idx[worker_idx] = None;
                }
            }

            // Wait for a response from any worker
            if active_tasks > 0 {
                let (worker_idx, response) = self.recv().unwrap();
                match response {
                    Ok(result) => {
                        if let Some(task_idx) = worker_task_idx[worker_idx] {
                            outcomes[task_idx] = Some(Ok(result));
                            on_progress();
                            active_tasks -= 1;
                        }
                        worker_task_idx[worker_idx] = None;
                        free_workers.push(worker_idx);
                    }
                    Err(err) => {
                        log::error!("Worker {} reported error: {}", worker_idx, err);
                        if let Some(task_idx) = worker_task_idx[worker_idx] {
                            outcomes[task_idx] = Some(Err(err));
                            on_progress();
                            active_tasks -= 1;
                        }
                        worker_task_idx[worker_idx] = None;
                    }
                }
            }
        }

        outcomes
            .into_iter()
            .map(|o| o.unwrap_or_else(|| Err("Task did not complete".to_string())))
            .collect()
    }

    pub fn broadcast(&self, req: Req) -> Vec<Result<Resp, String>> {
        let num_workers = self.num_workers();
        for idx in 0..num_workers {
            let _ = self.send(idx, req.clone());
        }

        let mut responses = Vec::new();
        responses.resize_with(num_workers, || None);

        for _ in 0..num_workers {
            let (worker_idx, response) = self.recv().unwrap();
            responses[worker_idx] = Some(response);
        }

        responses
            .into_iter()
            .map(|o| o.unwrap_or_else(|| Err("Worker did not respond".to_string())))
            .collect()
    }
}

impl<Req, Resp> Drop for ProcessPool<Req, Resp> {
    fn drop(&mut self) {
        self.workers.lock().unwrap().clear();
        for &pid in &self.child_pids {
            let mut status = 0;
            unsafe {
                libc::waitpid(pid, &mut status, 0);
            }
        }
    }
}
