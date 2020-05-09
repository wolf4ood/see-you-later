//! A runtime agnostic crete for scheduling async task.
//!
//! # Example
//!
//! Oneshot schedule
//!
//! ```rust,no_run
//! use see_you_later::once;
//! use std::sync::{
//!     atomic::{AtomicBool, Ordering},
//!     Arc,
//! };
//! use std::time::Duration;
//!
//! #[smol_potat::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let invoked = Arc::new(AtomicBool::new(false));
//!     let invoked1 = invoked.clone();
//!     let (_, task) = once(Duration::from_secs(1), || async {
//!         invoked1.store(true, Ordering::Relaxed);
//!     });
//!
//!     task.await;
//!
//!     assert_eq!(true, invoked.load(Ordering::Relaxed));
//!     Ok(())
//! }
//! ```
//!
//! Every x schedule
//! ```rust,no_run
//! use see_you_later::every;
//! use smol::{self, Task};
//! use std::time::Duration;
//! use wait_for_me::CountDownLatch;
//! #[smol_potat::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let latch = CountDownLatch::new(10);
//!     let inner_latch = latch.clone();
//!     let (cancel, task) = every(Duration::from_millis(100), || async {
//!         inner_latch.count_down().await;
//!     });
//!     Task::spawn(async move {
//!         latch.wait().await;
//!         cancel.cancel().await
//!     })
//!     .detach();
//!     task.await;
//!     Ok(())
//! }
//! ```
//!

#![deny(missing_docs)]

mod cancel;
mod every;
mod once;

pub use every::every;
pub use once::{once, DelayedTask};
