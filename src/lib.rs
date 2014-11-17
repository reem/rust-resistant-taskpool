#![deprecated = "std::sync::TaskPool has been replaced with this implementation"]
#![license = "MIT"]
#![deny(warnings, missing_docs)]
#![allow(deprecated)]

//! A failure resistant, load balancing task pool.

use std::sync::{Mutex, Arc};

enum MonitorMessage {
    Died,
    Kill
}

struct Watcher {
    monitor: Sender<MonitorMessage>
}

impl Drop for Watcher {
    fn drop(&mut self) {
        // Don't panic when this is dropped after the monitor task.
        let _ = self.monitor.send_opt(Died);
    }
}

/// A failure resistant, load balancing task pool.
///
/// Spawns n + 1 tasks and respawns on subtask panic.
pub struct TaskPool {
    jobs: Sender<proc(): Send>,
    monitor: Sender<MonitorMessage>,
}

impl TaskPool {
    /// Create a new TaskPool with n tasks.
    pub fn new(tasks: uint) -> TaskPool {
        let (job_tx, job_rx) = channel::<proc(): Send>();

        let (quit_tx, quit_rx) = channel::<MonitorMessage>();

        let job_rx = Arc::new(Mutex::new(job_rx));

        // Monitoring task that refreshes the task pool
        // if spawned tasks fail or end.
        let monitor_jobs = job_rx.clone();
        let monitor_quit = quit_tx.clone();
        spawn(proc() {
            for message in quit_rx.iter() {
                match message {
                    Died => spawn_in_pool(monitor_jobs.clone(), monitor_quit.clone()),
                    Kill => return
                }
            }
        });

        // Taskpool tasks.
        for _ in range(0, tasks) {
            spawn_in_pool(job_rx.clone(), quit_tx.clone());
        }

        TaskPool { jobs: job_tx, monitor: quit_tx }
    }

    /// Run this job on any of the tasks in the taskpool.
    pub fn execute(&mut self, job: proc(): Send) {
        self.jobs.send(job)
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        // Kill the monitor
        self.monitor.send(Kill);
    }
}

fn spawn_in_pool(jobs: Arc<Mutex<Receiver<proc(): Send>>>,
                 monitor: Sender<MonitorMessage>) {
    spawn(proc() {
        // Will alert the monitor task on failure.
        let w = Watcher { monitor: monitor };

        loop {
            let message = {
                // Only lock jobs for the time it takes
                // to get a job, not run it.
                let lock = jobs.lock();
                lock.recv_opt()
            };

            match message {
                Ok(job) => job(),
                Err(..) => break
            }
        }

        drop(w);
    })
}

