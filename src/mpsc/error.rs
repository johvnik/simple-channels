use core::fmt;
use std::error::Error;

/// An error returned from the `send` method.
///
/// The error contains the value that could not be sent. This occurs when the
/// channel is disconnected, meaning all receivers have been dropped.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(pub T);

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SendError").finish()
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "sending on a disconnected channel".fmt(f)
    }
}

impl<T: Send> Error for SendError<T> {}

/// An error returned from the `recv` and `try_recv` methods.
///
/// This error indicates that the channel is empty and disconnected, meaning
/// all senders have been dropped and no more messages will be sent.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RecvError {
    /// The channel is empty and disconnected.
    Disconnected,
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "receiving on an empty and disconnected channel".fmt(f)
    }
}

impl Error for RecvError {}

/// An error returned from the `try_recv` method.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError {
    /// The channel is currently empty, but is still connected.
    Empty,
    /// The channel is empty and disconnected.
    Disconnected,
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryRecvError::Empty => "receiving on an empty channel".fmt(f),
            TryRecvError::Disconnected => "receiving on an empty and disconnected channel".fmt(f),
        }
    }
}

impl Error for TryRecvError {}
