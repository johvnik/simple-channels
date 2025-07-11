use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
// NOTE: AtomicUsize is only available on platforms that support atomic
//       loads and stores of usize.
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Slot inside the ring buffer
struct Slot<T> {
    // MaybeUninit means "raw bytes big enough for T"
    // Use UnsafeCell for interior mutability
    data: UnsafeCell<MaybeUninit<T>>,
    empty: AtomicBool,
}

impl<T> Slot<T> {
    // The `const fn` lets us build an array of uninitialized slots at compile time
    // without needing `unsafe`.
    const fn new() -> Self {
        Self {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            empty: AtomicBool::new(true),
        }
    }
}

unsafe impl<T: Send> Send for Slot<T> {}
unsafe impl<T: Send> Sync for Slot<T> {}

/// Lock-free, single-consumer ring buffer.
pub(crate) struct RingBuf<T> {
    // A heap-allocated array of size CAP
    buf: Box<[Slot<T>]>,
    // Next write position (producers)
    head: AtomicUsize,
    mask: usize,
    // Next read  position (comsumer)
    tail: AtomicUsize,
}

impl<T> RingBuf<T> {
    pub(crate) fn new(cap: usize) -> Self {
        // Power-of-two capacity for performant masking
        // A slow modulo (`% CAP`) can now be a fast mitmask (`& CAP`) when wrapping indices.
        assert!(cap.is_power_of_two(), "CAP must be 2^n");

        let mut v = Vec::with_capacity(cap);
        v.resize_with(cap, || Slot::new());
        Self {
            buf: v.into_boxed_slice(),
            head: AtomicUsize::new(0),
            mask: cap - 1,
            tail: AtomicUsize::new(0),
        }
    }

    /// Producers push. Returns `Err(value)` when the buffer is full.
    pub(crate) fn push(&self, value: T) -> Result<(), T> {
        loop {
            // Load the current head to check the corresponding slot.
            let current_head = self.head.load(Ordering::Relaxed);
            let pos = current_head & self.mask;
            let slot = &self.buf[pos];

            // Check if the slot is empty.
            if !slot.empty.load(Ordering::Acquire) {
                // The slot is occupied. Before returning an error, we should check if
                // the buffer is actually full, as the consumer might be about to free a slot.
                // A simple check is to see if the head has lapped the tail.
                let tail_pos = self.tail.load(Ordering::Relaxed);
                if current_head.wrapping_sub(tail_pos) >= self.buf.len() {
                    return Err(value); // Buffer is genuinely full.
                }
                // If not full, another producer is likely using this slot, or we are waiting
                // for the consumer. Yielding can be polite, or just loop again.
                std::hint::spin_loop(); // or thread::yield_now();
                continue;
            }

            // Try to atomically claim this slot by incrementing head.
            // We use compare_exchange to ensure we only update it if it hasn't changed.
            match self.head.compare_exchange(
                current_head,
                current_head.wrapping_add(1),
                Ordering::Release, // Use Release on success to sync with the write below.
                Ordering::Relaxed, // Use Relaxed on failure, we'll just loop again.
            ) {
                Ok(_) => {
                    // Success! We have exclusive access to this slot.
                    // SAFETY: We are the only thread that can succeed in the CAS for this `pos`.
                    unsafe { (*slot.data.get()).write(value) };
                    slot.empty.store(false, Ordering::Release);
                    return Ok(());
                }
                Err(_) => {
                    // Another producer beat us to it. Loop and try again.
                    continue;
                }
            }
        }
    }

    /// Consumer pop. Returns `None` if the buffer is empty.
    pub(crate) fn pop(&self) -> Option<T> {
        let pos = self.tail.load(Ordering::Relaxed) & self.mask;
        let slot = &self.buf[pos];

        if slot.empty.load(Ordering::Acquire) {
            return None; // buffer is empty
        }

        // SAFETY: We are the single consumer, so we have exclusive access to read.
        //         Only call assume_init_read() once to avoid UB due to a double-free.
        let v = unsafe { (*slot.data.get()).assume_init_read() };
        slot.empty.store(true, Ordering::Release);
        // SAFETY: Wraps automatically if usize::MAX is reached. No UB.
        self.tail.fetch_add(1, Ordering::Relaxed);
        Some(v)
    }
}
