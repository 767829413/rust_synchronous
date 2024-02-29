pub fn condvar_exp() {
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;

    // 创建一个 Mutex 和 Condvar，用于共享状态和线程协调
    let mutex = Arc::new(Mutex::new(false));
    let condvar = Arc::new(Condvar::new());

    // 创建多个线程
    let mut handles = vec![];

    for id in 0..3 {
        let mutex = Arc::clone(&mutex);
        let condvar = Arc::clone(&condvar);

        let handle = thread::spawn(move || {
            // 获取 Mutex 锁
            let mut guard = mutex.lock().unwrap();

            // 线程等待条件变量为 true
            while !*guard {
                guard = condvar.wait(guard).unwrap();
            }

            // 条件满足后执行的操作
            println!("Thread {} woke up", id);
        });

        handles.push(handle);
    }

    // 模拟条件满足后唤醒等待的线程
    thread::sleep(std::time::Duration::from_secs(2));

    // 修改条件，并唤醒等待的线程
    {
        let mut guard = mutex.lock().unwrap();
        *guard = true;
        condvar.notify_all();
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
}
