pub fn mpsc_exp() {
    use std::sync::mpsc::channel;
    use std::thread;

    // Create a simple streaming channel
    let (tx, rx) = channel();
    thread::spawn(move || {
        tx.send(10).unwrap();
    });
    assert_eq!(rx.recv().unwrap(), 10);
}

pub fn mpsc_producer() {
    use std::sync::mpsc::channel;
    use std::thread;

    // Create a shared channel that can be sent along from many threads
    // where tx is the sending half (tx for transmission), and rx is the receiving
    // half (rx for receiving).
    let (tx, rx) = channel();
    for i in 0..10 {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(i).unwrap();
        });
    }

    for _ in 0..10 {
        let j = rx.recv().unwrap();
        println!("j: {}", j);
    }
}

pub fn mpsc_sync() {
    use std::sync::mpsc::sync_channel;
    use std::thread;

    let (tx, rx) = sync_channel(3);

    for _ in 0..3 {
        // It would be the same without thread and clone here
        // since there will still be one `tx` left.
        let tx = tx.clone();
        // cloned tx dropped within thread
        thread::spawn(move || tx.send("ok").unwrap());
    }

    // Drop the last sender to stop `rx` waiting for message.
    // The program will not complete if we comment this out.
    // **All** `tx` needs to be dropped for `rx` to have `Err`.
    drop(tx);

    // Unbounded receiver waiting for all senders to complete.
    while let Ok(msg) = rx.recv() {
        println!("{msg}");
    }

    println!("completed");
}

pub fn mpsc_receiver_error() {
    use std::sync::mpsc::channel;

    // The call to recv() will return an error because the channel has already
    // hung up (or been deallocated)
    let (tx, rx) = channel::<i32>();
    drop(tx);
    println!("is error: {}, error is {:?}", rx.recv().is_err(), rx.recv());
}
