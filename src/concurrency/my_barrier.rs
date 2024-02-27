pub fn barrier_exp() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    // 创建一个 Barrier，指定参与同步的线程数量
    let barrier = Arc::new(Barrier::new(3)); // 在这个例子中，有 3 个线程参与同步

    // 创建多个线程
    let mut handles = vec![];

    for id in 0..3 {
        let barrier = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            // 模拟一些工作
            println!("Thread {} working", id);
            thread::sleep(std::time::Duration::from_secs(id as u64));

            // 线程到达同步点
            barrier.wait();

            // 执行同步后的操作
            println!("Thread {} resumed", id);
        });

        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
}

pub fn barrier_loop() {
    use rand::Rng;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time;

    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    for index in 0..10 {
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            println!("before wait {}", index);
            let dur = rand::thread_rng().gen_range(100..1000);
            thread::sleep(std::time::Duration::from_millis(dur));

            //step1
            barrier.wait();
            println!("after wait {}", index);
            thread::sleep(time::Duration::from_secs(1));

            //step2
            barrier.wait();
            println!("after wait {}", index);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
