mod error;
mod ring_buf;

pub use error::{RecvError, SendError, TryRecvError};

use ring_buf::RingBuf;
use std::{sync::Arc, thread};

/// The sending side of a channel.
pub struct Sender<T> {
    channel: Arc<RingBuf<T>>,
}

/// The receiving side of a channel.
pub struct Receiver<T> {
    channel: Arc<RingBuf<T>>,
}

/// Creates a new bounded MPSC channel with a specified capacity.
///
/// The capacity must be a power of two.
///
/// # Panics
///
/// Panics if the capacity is not a power of two.
pub fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(RingBuf::new(cap));
    let sender = Sender {
        channel: Arc::clone(&channel),
    };
    let receiver = Receiver { channel };
    (sender, receiver)
}

impl<T> Sender<T> {
    /// Sends a value down the channel.
    ///
    /// This method will block if the channel's buffer is full.
    ///
    /// An error is returned if the receiver has been dropped.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        // To check for disconnection, we see if the Arc has only one reference left.
        // If so, it must be this Sender, meaning the Receiver is gone.
        // We use a relaxed ordering because we don't need to synchronize memory with this check.
        if Arc::strong_count(&self.channel) == 1 {
            return Err(SendError(value));
        }

        let mut current_value = value;
        loop {
            // Attempt to push the value into the ring buffer.
            match self.channel.push(current_value) {
                Ok(()) => return Ok(()),
                Err(v) => {
                    // The buffer is full. We store the value back and yield.
                    current_value = v;
                    thread::yield_now(); // Yield to allow the receiver to catch up.

                    // After yielding, we must re-check for disconnection.
                    if Arc::strong_count(&self.channel) == 1 {
                        return Err(SendError(current_value));
                    }
                }
            }
        }
    }
}

// Implement Clone to allow for multiple producers.
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            channel: Arc::clone(&self.channel),
        }
    }
}

impl<T> Receiver<T> {
    /// Receives a value from the channel.
    ///
    /// This method will block until a message is available.
    ///
    /// An error is returned if the channel is empty and all senders have been dropped.
    pub fn recv(&self) -> Result<T, RecvError> {
        loop {
            // Attempt to pop a value from the buffer.
            match self.channel.pop() {
                Some(value) => return Ok(value),
                None => {
                    // Buffer is empty. Check if senders are still connected.
                    // If the strong count is 1, only this Receiver holds the Arc.
                    if Arc::strong_count(&self.channel) == 1 {
                        return Err(RecvError::Disconnected);
                    }
                    // Yield to allow senders to produce a message.
                    thread::yield_now();
                }
            }
        }
    }

    /// Attempts to receive a value from the channel without blocking.
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        match self.channel.pop() {
            Some(value) => Ok(value),
            None => {
                // Buffer is empty. Check for disconnection.
                if Arc::strong_count(&self.channel) == 1 {
                    Err(TryRecvError::Disconnected)
                } else {
                    Err(TryRecvError::Empty)
                }
            }
        }
    }
}
