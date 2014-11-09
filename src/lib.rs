#![license = "MIT"]
#![deny(missing_docs)]
#![deny(warnings)]

//! A failure resistant, load balancing task pool.

use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, SeqCst};

enum InternalMessage {
    Quit,
    Job(proc(): Send)
}

struct Watcher {
    channel: Sender<()>
}

impl Drop for Watcher {
    fn drop(&mut self) {
        // Don't panic
        let _ = self.channel.send_opt(());
    }
}

/// A failure resistant, load balancing task pool.
///
/// Spawns n + 1 tasks and respawns on subtask panic.
pub struct TaskPool {
    tx: Sender<InternalMessage>,
    tasks: uint,
    done: Arc<AtomicBool>
}

impl TaskPool {
    /// Create a new TaskPool with n tasks.
    pub fn new(tasks: uint) -> TaskPool {
        let (job_tx, job_rx) = channel::<InternalMessage>();

        let (quit_tx, quit_rx) = channel::<()>();
        let done = Arc::new(AtomicBool::new(false));

        let job_rx = Arc::new(Mutex::new(job_rx));

        // Monitoring task that refreshes the task pool
        // if spawned tasks fail or end.
        let monitor_jobs = job_rx.clone();
        let monitor_done = done.clone();
        let monitor_quit = quit_tx.clone();
        spawn(proc() {
            for _ in quit_rx.iter() {
                if monitor_done.load(SeqCst) { break }
                spawn_in_pool(monitor_jobs.clone(), monitor_quit.clone())
            }
        });

        // Taskpool tasks.
        for _ in range(0, tasks) {
            spawn_in_pool(job_rx.clone(), quit_tx.clone());
        }

        TaskPool { tx: job_tx, tasks: tasks, done: done }
    }

    /// Run this job on any of the tasks in the taskpool.
    pub fn execute(&mut self, job: proc(): Send) {
        self.tx.send(Job(job))
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        self.done.swap(true, SeqCst);

        for _ in range(0, self.tasks) {
            // Don't double panic.
            self.tx.send(Quit);
        }
    }
}

fn spawn_in_pool(jobs: Arc<Mutex<Receiver<InternalMessage>>>, quit_tx: Sender<()>) {
    spawn(proc() {
        // Will alert the monitor task on failure.
        let w = Watcher { channel: quit_tx };

        loop {
            let message = jobs.lock().recv_opt();
            match message {
                Ok(Job(job)) => job(),
                Ok(Quit) => break,
                Err(..) => break
            }
        }

        drop(w);
    })
}

