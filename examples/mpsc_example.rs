use simple_channels::mpsc;
use std::thread;

fn main() {
    // Define the number of producer threads and how many messages each will send.
    const NUM_PRODUCERS: usize = 4;
    const MESSAGES_PER_PRODUCER: usize = 100;

    println!(
        "Spawning {} producer threads, each sending {} numbers.",
        NUM_PRODUCERS, MESSAGES_PER_PRODUCER
    );

    // Create a channel with a capacity that can handle some burst traffic.
    let (tx, rx) = mpsc::bounded(64);

    // A vector to hold the handles of the producer threads we spawn.
    let mut producer_handles = Vec::new();

    // Spawn the producer threads.
    for i in 0..NUM_PRODUCERS {
        // Clone the sender for each new thread. This is how you get multiple producers.
        let tx_clone = tx.clone();

        let handle = thread::spawn(move || {
            // Each thread sends a unique range of numbers.
            // For example:
            // Thread 0 sends 0..100
            // Thread 1 sends 100..200
            // ...and so on.
            let start = i * MESSAGES_PER_PRODUCER;
            let end = start + MESSAGES_PER_PRODUCER;

            for number in start..end {
                tx_clone.send(number).unwrap();
            }
            // The sender for this thread is dropped here when the thread exits.
        });
        producer_handles.push(handle);
    }

    // IMPORTANT: Drop the original sender in the main thread.
    // This is crucial because the receiver's `recv()` loop will only terminate
    // (by returning an `Err`) when ALL senders have been dropped.
    drop(tx);

    println!("Main thread is now receiving all numbers...");

    // The main thread acts as the single consumer.
    // We will collect all the numbers sent from the producer threads.
    let mut received_numbers = Vec::new();

    // This `while let` loop will automatically break when the channel is empty
    // AND all senders have been dropped.
    while let Ok(number) = rx.recv() {
        received_numbers.push(number);
    }

    println!(
        "Receiver loop finished. Total numbers received: {}",
        received_numbers.len()
    );

    // Wait for all producer threads to finish their work.
    // This ensures we don't exit the main function prematurely.
    for handle in producer_handles {
        handle.join().unwrap();
    }

    // Verify that we received all the messages.
    let expected_count = NUM_PRODUCERS * MESSAGES_PER_PRODUCER;
    assert_eq!(received_numbers.len(), expected_count);

    println!(
        "\nSuccessfully received all {} numbers from {} threads.",
        expected_count, NUM_PRODUCERS
    );

    // Optional: Sort and print a few numbers to see the result.
    received_numbers.sort();
    println!("First 10 numbers (sorted): {:?}", &received_numbers[0..10]);
    println!(
        "Last 10 numbers (sorted): {:?}",
        &received_numbers[expected_count - 10..]
    );
}
