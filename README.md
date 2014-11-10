# Resistant-Taskpool

> A mostly drop-in replacement for `std::sync::TaskPool` that is resistant to
> `panic!`.

The `std::sync` TaskPool will panic if any of its spawned tasks do
and will also panic during unwinding under the same condition, causing
a process abort from any spawned task panic.

This TaskPool spawns an additional monitor task, and monitors all
child tasks for panics and will start new tasks in the pool in the
event that a spawned task panics or returns.

Additionally, this TaskPool performs less allocation than
`std::sync::TaskPool` and uses an mpmc queue to do load balancing
on the child tasks instead of rotating through all spawned tasks.


## Example

```rust
let mut pool = TaskPool::new(8);

// Panic all the created tasks.
for _ in range(0, 8u) {
    pool.execute(proc() {
        panic!("muahaha");
    });
}

// The TaskPool will spawn new tasks to replace the old ones and will
// give out jobs to these new tasks as soon as they are ready.

// Send out new tasks
let (tx, rx) = channel();
for _ in range(0, 8u) {
    let tx = tx.clone();
    pool.execute(proc() {
        tx.send(2u);
    });
}

assert_eq!(rx.iter().take(8).sum(), 16);

// Dropping the task pool kills all of the spawned tasks
// but will also not panic if any of the tasks panic from
// their current job before receiving the kill message.
drop(pool);
```

