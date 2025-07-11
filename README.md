# simple-channels

[![Crates.io](https://img.shields.io/crates/v/simple-channels.svg)](https://crates.io/crates/simple-channels)
[![Docs.rs](https://docs.rs/simple-channels/badge.svg)](https://docs.rs/simple-channels)
[![Crates.io License](https://img.shields.io/crates/l/simple-channels)](https://crates.io/crates/simple-channels)

A collection of simple, thread-safe channel implementations in Rust, created for educational purposes. This project focuses on demonstrating the core principles of concurrent data structures using clear, concise code.

The initial implementation is a Multi-Producer, Single-Consumer (MPSC) channel.

## Core Components

1.  **Lock-Free `RingBuf`**: A fixed-size, circular buffer that uses atomic operations and a compare-and-swap (CAS) loop to manage concurrent access without mutexes.

2.  **MPSC Channel API**: A high-level `Sender`/`Receiver` API built on the `RingBuf`.
    * The `Sender` can be cloned to allow writes from multiple threads.
    * A single `Receiver` consumes values, enforcing the single-consumer model.

## Design Philosophy

The implementation prioritizes clarity and fundamental concepts over raw performance.

* **Blocking Strategy**: The channel uses a simple spin-and-yield approach (`std::thread::yield_now()`) for blocking. When the buffer is full or empty, threads yield their execution slot, avoiding more complex synchronization primitives like `park`/`unpark`.

* **State Management**: Shared channel state is managed via `Arc`, and disconnection is detected by checking the atomic reference count.

---

## Roadmap

This project can be extended by implementing other common channel types and features. Potential future work includes:

* **SPSC Channel**: A specialized channel for the Single-Producer, Single-Consumer use case.
* **MPMC Channel**: A more complex Multi-Producer, Multi-Consumer channel.
* **Unbounded Channel**: A channel variant that can grow dynamically.
* **Efficient Blocking**: Integration with a more sophisticated mechanism like `std::thread::park` to reduce CPU usage while waiting.
* **Benchmarking**: A benchmark suite to measure and compare performance during development.
* **Testing**: Try out deterministic simulation testing
