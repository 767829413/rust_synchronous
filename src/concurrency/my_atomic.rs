pub fn atomic_exp() {
    use std::sync::atomic::{AtomicI64, Ordering};

    let atomic_num = AtomicI64::new(0);

    // 原子加载
    let _num = atomic_num.load(Ordering::Relaxed);

    // 原子加法并返回旧值
    let old = atomic_num.fetch_add(10, Ordering::SeqCst);

    // 原子比较并交换
    _ = atomic_num.compare_exchange(old, 100, Ordering::SeqCst, Ordering::Acquire);

    // 原子交换
    let _swapped = atomic_num.swap(200, Ordering::Release);

    // 原子存储
    atomic_num.store(1000, Ordering::Relaxed);
}

pub fn atomic_ordering_relaxed() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread;

    // 创建一个原子布尔值，并用 Arc 包装起来, 设置布尔值为 true
    let atomic_bool = Arc::new(AtomicBool::new(false));

    let atomic_bool_clone = Arc::clone(&atomic_bool);
    // 创建一个生产者线程，
    let producer_thread = thread::spawn(move || {
        // 这里可能会有指令重排，因为使用了 Ordering::Relaxed
        atomic_bool_clone.store(true, Ordering::Relaxed);
    });

    // 创建一个消费者线程，检查布尔值的状态
    let atomic_bool_clone = Arc::clone(&atomic_bool);
    let consumer_thread = thread::spawn(move || {
        // 这里可能会有指令重排，因为使用了 Ordering::Relaxed
        let value = atomic_bool_clone.load(Ordering::Relaxed);
        println!("Received value: {}", value);
    });

    // 等待线程完成
    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
}

pub fn atomic_ordering_acquire() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread;

    // 创建一个原子布尔值
    let atomic_bool = Arc::new(AtomicBool::new(false));

    // 创建一个生产者线程，设置布尔值为 true
    let atomic_bool_clone = Arc::clone(&atomic_bool);
    let producer_thread = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_secs(1));
        // 设置布尔值为 true
        atomic_bool_clone.store(true, Ordering::Release);
    });

    // 创建一个消费者线程，读取布尔值的状态
    let atomic_bool_clone = Arc::clone(&atomic_bool);
    let consumer_thread = thread::spawn(move || {
        // 等待直到读取到布尔值为 true
        while !atomic_bool_clone.load(Ordering::Acquire) {
            // 这里可能进行自旋，直到获取到 Acquire 顺序的布尔值
            // 注意：在实际应用中，可以使用更高级的同步原语而不是自旋
            println!("Wait value");
            thread::sleep(std::time::Duration::from_secs(1));
        }

        println!("Received value: true");
    });

    // 等待线程完成
    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
}

pub fn atomic_ordering_release() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread;
    // 创建一个原子布尔值
    let atomic_bool = Arc::new(AtomicBool::new(false));

    // 创建一个生产者线程，设置布尔值为 true
    let atomic_bool_clone = Arc::clone(&atomic_bool);
    let producer_thread = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_secs(1));
        // 设置布尔值为 true
        atomic_bool_clone.store(true, Ordering::Release);
    });

    // 创建一个消费者线程，读取布尔值的状态
    let atomic_bool_clone = Arc::clone(&atomic_bool);
    let consumer_thread = thread::spawn(move || {
        // 等待直到读取到布尔值为 true
        while !atomic_bool_clone.load(Ordering::Acquire) {
            // 这里可能进行自旋，直到获取到 Release 顺序的布尔值
            // 注意：在实际应用中，可以使用更高级的同步原语而不是自旋
            println!("Wait value");
            thread::sleep(std::time::Duration::from_secs(1));
        }

        println!("Received value: true");
    });

    // 等待线程完成
    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
}

pub fn atomic_ordering_acqrel() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread;
    // 创建一个原子布尔值
    let atomic_bool = Arc::new(AtomicBool::new(false));

    // 创建一个生产者线程，设置布尔值为 true
    let atomic_bool_clone = Arc::clone(&atomic_bool);
    let producer_thread = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_secs(1));
        // 设置布尔值为 true
        atomic_bool_clone.store(true, Ordering::Release);
    });

    // 创建一个消费者线程，读取布尔值的状态
    let atomic_bool_clone = Arc::clone(&atomic_bool);
    let consumer_thread = thread::spawn(move || {
        // 等待直到读取到布尔值为 true
        while !atomic_bool_clone.load(Ordering::Acquire) {
            // 这里可能进行自旋，直到获取到 Acquire 顺序的布尔值
            // 注意：在实际应用中，可以使用更高级的同步原语而不是自旋
            println!("Wait value");
            thread::sleep(std::time::Duration::from_secs(1));
        }

        println!("Received value: true");
    });

    // 等待线程完成
    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
}

pub fn atomic_ordering_seqcst() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread;
    // 创建一个原子布尔值
    let atomic_bool = Arc::new(AtomicBool::new(false));

    // 创建一个生产者线程，设置布尔值为 true
    let atomic_bool_clone = Arc::clone(&atomic_bool);
    let producer_thread = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_secs(1));
        // 设置布尔值为 true
        atomic_bool_clone.store(true, Ordering::SeqCst);
    });

    // 创建一个消费者线程，读取布尔值的状态
    let atomic_bool_clone = Arc::clone(&atomic_bool);
    let consumer_thread = thread::spawn(move || {
        // 等待直到读取到布尔值为 true
        while !atomic_bool_clone.load(Ordering::SeqCst) {
            // 这里可能进行自旋，直到获取到 SeqCst 顺序的布尔值
            // 注意：在实际应用中，可以使用更高级的同步原语而不是自旋
            println!("Wait value");
            thread::sleep(std::time::Duration::from_secs(1));
        }

        println!("Received value: true");
    });

    // 等待线程完成
    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
}
