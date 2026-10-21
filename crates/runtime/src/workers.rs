//! Experimental bounded host-side workers.
//!
//! A worker reuses only its host thread. Each [`WorkerRequest`] invokes through
//! [`crate::Tenant::invoke`], which creates and discards a fresh Store and
//! instance for that request (ADR-005). This module does not support WASI I/O
//! or custom host contexts.

use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use thiserror::Error;

use crate::{Component, InvocationReport, RuntimeError, Tenant, Val};

/// Fixed limits for an experimental [`WorkerPool`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerPoolConfig {
    /// Number of worker threads to create.
    pub workers: usize,
    /// Maximum number of accepted requests waiting for a worker.
    pub queue_capacity: usize,
}

impl WorkerPoolConfig {
    /// Creates a configuration with explicit worker and queue limits.
    #[must_use]
    pub const fn new(workers: usize, queue_capacity: usize) -> Self {
        Self {
            workers,
            queue_capacity,
        }
    }

    fn validate(self) -> Result<(NonZeroUsize, NonZeroUsize), WorkerPoolConfigError> {
        let workers = NonZeroUsize::new(self.workers).ok_or(WorkerPoolConfigError::ZeroWorkers)?;
        let queue_capacity = NonZeroUsize::new(self.queue_capacity)
            .ok_or(WorkerPoolConfigError::ZeroQueueCapacity)?;
        Ok((workers, queue_capacity))
    }
}

/// Invalid worker-pool configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum WorkerPoolConfigError {
    /// At least one worker is required.
    #[error("worker count must be greater than zero")]
    ZeroWorkers,
    /// At least one queued request must be representable.
    #[error("queue capacity must be greater than zero")]
    ZeroQueueCapacity,
}

/// A request that owns all invocation-specific state.
///
/// The component is an immutable parsed artifact. It is never an instantiated
/// guest and therefore does not weaken the fresh-instance invariant.
pub struct WorkerRequest {
    component: Arc<Component>,
    tenant: Tenant,
    function: String,
    arguments: Vec<Val>,
}

impl WorkerRequest {
    /// Creates a request for ordinary [`Tenant::invoke`] execution.
    #[must_use]
    pub fn new(
        component: Arc<Component>,
        tenant: Tenant,
        function: impl Into<String>,
        arguments: Vec<Val>,
    ) -> Self {
        Self {
            component,
            tenant,
            function: function.into(),
            arguments,
        }
    }
}

/// Rejection returned before a request enters the pool.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SubmitError {
    /// The bounded queue has no available slot.
    #[error("worker queue is full (capacity: {queue_capacity})")]
    QueueFull {
        /// Configured queue capacity.
        queue_capacity: usize,
    },
    /// The request has no finite fuel budget to bound a running worker.
    #[error("worker requests require a finite fuel budget")]
    UnboundedExecution,
    /// The pool no longer accepts work.
    #[error("worker pool is closed")]
    Closed,
}

/// Failure returned by an accepted worker request.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkerError {
    /// The ordinary runtime invocation failed.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    /// The request was removed before a worker started it.
    #[error("worker request was cancelled before execution")]
    Cancelled,
    /// The worker caught a Rust panic while processing the request.
    #[error("worker request panicked")]
    WorkerPanicked,
}

/// A cancellation request that cannot remove a queued job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CancelError {
    /// A worker already owns the request and it cannot be interrupted safely.
    #[error("worker request is already running")]
    AlreadyRunning,
    /// The request has already produced a result.
    #[error("worker request has already finished")]
    Finished,
}

/// The treatment of accepted work during pool shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownMode {
    /// Finish the requests already in the queue.
    Drain,
    /// Cancel requests that have not started, then await running work.
    RejectQueued,
}

/// Aggregated pool state without tenant or invocation payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerPoolStats {
    /// Configured worker count.
    pub workers: usize,
    /// Configured waiting-queue capacity.
    pub queue_capacity: usize,
    /// Requests currently waiting for a worker.
    pub queued: usize,
    /// Requests currently executing.
    pub running: usize,
    /// Successfully accepted requests.
    pub accepted: u64,
    /// Requests rejected because the queue was full.
    pub rejected: u64,
    /// Requests cancelled before execution.
    pub cancelled: u64,
    /// Requests completed successfully.
    pub completed: u64,
    /// Requests that returned a runtime error.
    pub failed: u64,
    /// Requests whose worker caught a Rust panic.
    pub panicked: u64,
}

/// Final state after [`WorkerPool::close`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownReport {
    /// Aggregated state after every worker has exited.
    pub stats: WorkerPoolStats,
}

/// An accepted request that can be cancelled while queued or awaited.
pub struct WorkerTicket {
    id: u64,
    inner: Arc<PoolInner>,
    control: Arc<Mutex<TicketState>>,
    receiver: Receiver<Result<InvocationReport, WorkerError>>,
}

impl WorkerTicket {
    /// Removes this request from the queue if no worker has started it.
    ///
    /// Running work is not forcefully interrupted. Its tenant fuel and resource
    /// limits remain the execution-containment mechanism.
    pub fn try_cancel(&self) -> Result<(), CancelError> {
        let mut state = lock_state(&self.inner);
        let Some(position) = state.queue.iter().position(|job| job.id == self.id) else {
            return match *self
                .control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
            {
                TicketState::Running => Err(CancelError::AlreadyRunning),
                TicketState::Finished => Err(CancelError::Finished),
                TicketState::Queued => unreachable!("queued ticket must be in the queue"),
            };
        };

        let job = state
            .queue
            .remove(position)
            .expect("position was obtained from this queue");
        set_ticket_state(&job.control, TicketState::Finished);
        let _ = job.response.send(Err(WorkerError::Cancelled));
        state.cancelled += 1;
        self.inner.wake.notify_all();
        Ok(())
    }

    /// Waits for this request's invocation report or terminal error.
    pub fn recv(self) -> Result<InvocationReport, WorkerError> {
        self.receiver
            .recv()
            .unwrap_or(Err(WorkerError::WorkerPanicked))
    }
}

/// Fixed-size host-thread pool for experimental ordinary tenant invocations.
pub struct WorkerPool {
    inner: Arc<PoolInner>,
    handles: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    /// Creates exactly the configured number of worker threads.
    ///
    /// # Errors
    ///
    /// Returns [`WorkerPoolConfigError`] when either configured capacity is zero.
    pub fn new(config: WorkerPoolConfig) -> Result<Self, WorkerPoolConfigError> {
        let (workers, queue_capacity) = config.validate()?;
        let inner = Arc::new(PoolInner {
            state: Mutex::new(PoolState::new(workers.get(), queue_capacity.get())),
            wake: Condvar::new(),
        });
        let mut handles = Vec::with_capacity(workers.get());
        for _ in 0..workers.get() {
            let worker_inner = Arc::clone(&inner);
            handles.push(thread::spawn(move || worker_loop(&worker_inner)));
        }
        Ok(Self { inner, handles })
    }

    /// Enqueues a request or returns explicit backpressure.
    pub fn submit(&self, request: WorkerRequest) -> Result<WorkerTicket, SubmitError> {
        let (sender, receiver) = mpsc::channel();
        let control = Arc::new(Mutex::new(TicketState::Queued));
        let mut state = lock_state(&self.inner);
        if state.lifecycle != Lifecycle::Accepting {
            return Err(SubmitError::Closed);
        }
        if request.tenant.config().limits.fuel_budget.is_none() {
            return Err(SubmitError::UnboundedExecution);
        }
        if state.queue.len() >= state.queue_capacity {
            state.rejected += 1;
            return Err(SubmitError::QueueFull {
                queue_capacity: state.queue_capacity,
            });
        }
        let id = state.next_id;
        state.next_id = state.next_id.wrapping_add(1);
        state.queue.push_back(Job {
            id,
            request,
            response: sender,
            control: Arc::clone(&control),
        });
        state.accepted += 1;
        self.inner.wake.notify_one();
        Ok(WorkerTicket {
            id,
            inner: Arc::clone(&self.inner),
            control: Arc::clone(&control),
            receiver,
        })
    }

    /// Returns aggregate operational counters.
    #[must_use]
    pub fn stats(&self) -> WorkerPoolStats {
        let state = lock_state(&self.inner);
        state.stats()
    }

    /// Stops admission, applies the selected queue policy, and joins workers.
    #[must_use]
    pub fn close(mut self, mode: ShutdownMode) -> ShutdownReport {
        {
            let mut state = lock_state(&self.inner);
            state.lifecycle = Lifecycle::Closing;
            if mode == ShutdownMode::RejectQueued {
                while let Some(job) = state.queue.pop_front() {
                    set_ticket_state(&job.control, TicketState::Finished);
                    let _ = job.response.send(Err(WorkerError::Cancelled));
                    state.cancelled += 1;
                }
            }
            self.inner.wake.notify_all();
        }
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
        let mut state = lock_state(&self.inner);
        state.lifecycle = Lifecycle::Closed;
        ShutdownReport {
            stats: state.stats(),
        }
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        if self.handles.is_empty() {
            return;
        }
        let mut state = lock_state(&self.inner);
        if state.lifecycle == Lifecycle::Accepting {
            state.lifecycle = Lifecycle::Closing;
            while let Some(job) = state.queue.pop_front() {
                set_ticket_state(&job.control, TicketState::Finished);
                let _ = job.response.send(Err(WorkerError::Cancelled));
                state.cancelled += 1;
            }
            self.inner.wake.notify_all();
        }
    }
}

struct PoolInner {
    state: Mutex<PoolState>,
    wake: Condvar,
}

struct PoolState {
    lifecycle: Lifecycle,
    queue: VecDeque<Job>,
    workers: usize,
    queue_capacity: usize,
    running: usize,
    next_id: u64,
    accepted: u64,
    rejected: u64,
    cancelled: u64,
    completed: u64,
    failed: u64,
    panicked: u64,
}

impl PoolState {
    fn new(workers: usize, queue_capacity: usize) -> Self {
        Self {
            lifecycle: Lifecycle::Accepting,
            queue: VecDeque::with_capacity(queue_capacity),
            workers,
            queue_capacity,
            running: 0,
            next_id: 0,
            accepted: 0,
            rejected: 0,
            cancelled: 0,
            completed: 0,
            failed: 0,
            panicked: 0,
        }
    }

    fn stats(&self) -> WorkerPoolStats {
        WorkerPoolStats {
            workers: self.workers,
            queue_capacity: self.queue_capacity,
            queued: self.queue.len(),
            running: self.running,
            accepted: self.accepted,
            rejected: self.rejected,
            cancelled: self.cancelled,
            completed: self.completed,
            failed: self.failed,
            panicked: self.panicked,
        }
    }
}

struct Job {
    id: u64,
    request: WorkerRequest,
    response: Sender<Result<InvocationReport, WorkerError>>,
    control: Arc<Mutex<TicketState>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Accepting,
    Closing,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TicketState {
    Queued,
    Running,
    Finished,
}

fn worker_loop(inner: &Arc<PoolInner>) {
    loop {
        let job = {
            let mut state = lock_state(inner);
            loop {
                if let Some(job) = state.queue.pop_front() {
                    set_ticket_state(&job.control, TicketState::Running);
                    state.running += 1;
                    break job;
                }
                if state.lifecycle != Lifecycle::Accepting {
                    return;
                }
                state = inner
                    .wake
                    .wait(state)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
        };

        let result = catch_unwind(AssertUnwindSafe(|| {
            job.request.tenant.invoke(
                &job.request.component,
                &job.request.function,
                &job.request.arguments,
            )
        }));
        let result = match result {
            Ok(Ok(report)) => Ok(report),
            Ok(Err(error)) => Err(WorkerError::Runtime(error)),
            Err(_) => Err(WorkerError::WorkerPanicked),
        };
        let outcome = match &result {
            Ok(_) => Completion::Success,
            Err(WorkerError::Runtime(_)) => Completion::RuntimeFailure,
            Err(WorkerError::WorkerPanicked) => Completion::Panic,
            Err(WorkerError::Cancelled) => unreachable!("running work cannot be cancelled"),
        };
        let _ = job.response.send(result);

        let mut state = lock_state(inner);
        state.running -= 1;
        match outcome {
            Completion::Success => state.completed += 1,
            Completion::RuntimeFailure => state.failed += 1,
            Completion::Panic => state.panicked += 1,
        }
        set_ticket_state(&job.control, TicketState::Finished);
        inner.wake.notify_all();
    }
}

fn lock_state(inner: &PoolInner) -> std::sync::MutexGuard<'_, PoolState> {
    inner
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn set_ticket_state(control: &Mutex<TicketState>, next: TicketState) {
    *control
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = next;
}

#[derive(Debug, Clone, Copy)]
enum Completion {
    Success,
    RuntimeFailure,
    Panic,
}
