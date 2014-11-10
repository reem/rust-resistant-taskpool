#![feature(phase)]
#![allow(unused_mut)]

#[phase(plugin)] extern crate stainless;
extern crate "taskpool" as pool;

pub use pool::TaskPool;
pub use std::iter::AdditiveIterator;

pub const NUMTASKS: uint = 4u;

describe! taskpool {
    before_each {
        let mut pool = TaskPool::new(NUMTASKS);
    }

    it "should run" {
        let (tx, rx) = channel();
        for _ in range(0, NUMTASKS) {
            let tx = tx.clone();
            pool.execute(proc() {
                tx.send(1u);
            });
        }
        assert_eq!(rx.iter().take(NUMTASKS).sum(), NUMTASKS);
    }

    it "should recover from subtask panics" {
        // Panic all the existing tasks.
        for _ in range(0, NUMTASKS) {
            pool.execute(proc() {
                panic!("Muahaha");
            });
        }

        // Ensure new tasks were spawned to compensate.
        let (tx, rx) = channel();
        for _ in range(0, NUMTASKS) {
            let tx = tx.clone();
            pool.execute(proc() {
                tx.send(1u);
            });
        }
        assert_eq!(rx.iter().take(NUMTASKS).sum(), NUMTASKS);
    }

    it "should not panic while dropping if subtasks failed" {
        // Panic all the existing tasks.
        for _ in range(0, NUMTASKS) {
            pool.execute(proc() {
                panic!("Muahaha");
            });
        }

        drop(pool);
    }

    it "should be Send" {
        fn is_send<S: Send>(_: S) {}
        is_send(pool);
    }
}

