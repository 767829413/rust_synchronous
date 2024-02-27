use crossbeam::channel::bounded;
use std::sync::{Arc, LockResult, Mutex, RwLock};
use std::thread;
use std::time::Duration;

pub fn mutex_lock_exp() {
    // 创建一个可共享的可变整数
    let counter = Arc::new(Mutex::new(0));

    // 创建多个线程来增加计数器的值
    let mut handles = vec![];

    for _ in 0..11 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // 获取锁，确保只有一个线程能够访问计数器
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 打印最终的计数器值
    println!("Final count: {}", counter.lock().unwrap());
}

pub fn mutex_try_lock_exp() {
    // 创建一个可共享的可变整数
    let counter = Arc::new(Mutex::new(0));

    // 创建多个线程来增加计数器的值
    let mut handles = vec![];

    for _ in 0..5000 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // 尝试获取锁，如果获取失败就继续尝试或者放弃
            if let Ok(mut num) = counter.try_lock() {
                *num += 1;
            } else {
                println!("Thread failed to acquire lock.");
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 打印最终的计数器值
    println!("Final count: {}", *counter.lock().unwrap());
}

pub fn mutex_poisoning_exp() {
    // 创建一个可共享的可变整数
    let counter = Arc::new(Mutex::new(0));

    // 创建多个线程来增加计数器的值
    let mut handles = vec![];

    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // 获取锁
            let result: LockResult<_> = counter.lock();

            // 尝试获取锁，如果获取失败就打印错误信息
            match result {
                Ok(mut num) => {
                    *num += 1;
                    // 模拟 panic
                    if *num == 3 {
                        panic!("Simulated panic!");
                    }
                }
                Err(poisoned) => {
                    // 锁被 "poisoned"，处理错误
                    println!("Thread encountered a poisoned lock: {:?}", poisoned);

                    // 获取 MutexGuard，继续操作
                    let mut num = poisoned.into_inner();
                    *num += 1;
                }
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        let r = handle.join();
        match r {
            Ok(_ok) => {
                println!("Final task")
            }
            Err(e) => {
                println!("error task {:?}", e);
            }
        }
    }

    // 打印最终的计数器值
    let rs = counter.lock();
    match rs {
        Ok(num) => {
            println!("Final count: {}", num);
        }
        Err(e) => {
            // 锁被 "poisoned"，处理错误
            println!("Final print num failed: {}", e.to_string());
        }
    }
}

pub fn mutex_fast_release_scop_exp() {
    // 创建一个可共享的可变整数
    let counter = Arc::new(Mutex::new(0));

    // 创建多个线程来增加计数器的值
    let mut handles = vec![];

    for _ in 0..115 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // 进入一个新的作用域!!!!!!!!!!!!!!
            {
                // 获取锁
                let mut num = counter.lock().unwrap();
                *num += 1;
                // MutexGuard 在这个作用域结束时自动释放锁
            }

            // 在这里，锁已经被释放
            // 这里可以进行其他操作
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 打印最终的计数器值
    println!("Final count: {}", *counter.lock().unwrap());
}

pub fn mutex_fast_release_drop_exp() {
    // 创建一个可共享的可变整数
    let counter = Arc::new(Mutex::new(0));

    // 创建多个线程来增加计数器的值
    let mut handles = vec![];

    for _ in 0..33 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // 获取锁
            let mut num = counter.lock().unwrap();
            *num += 1;

            // 手动释放锁!!!!!!!!
            drop(num);
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 打印最终的计数器值
    println!("Final count: {}", *counter.lock().unwrap());
}

pub fn rwmutex_exp() {
    // 创建一个可共享的可变整数，使用RwLock包装
    let counter = Arc::new(RwLock::new(12));

    // 创建多个线程来读取计数器的值
    let mut read_handles = vec![];

    for _ in 0..3 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // 获取读取锁
            let num = counter.read().unwrap();
            println!("Reader {:?}: {}", thread::current().id(), *num);
        });
        read_handles.push(handle);
    }

    // 创建一个线程来写入计数器的值
    let write_handle = thread::spawn(move || {
        // 获取写入锁
        let mut num = counter.write().unwrap();
        *num += 1;
        println!(
            "Writer {:?}: Incremented counter to {}",
            thread::current().id(),
            *num
        );
    });

    // 等待所有读取线程完成
    for handle in read_handles {
        handle.join().unwrap();
    }

    // 等待写入线程完成
    write_handle.join().unwrap();
}

pub fn rwmutex_exp_write_wait() {
    // 创建一个可共享的可变整数，使用RwLock包装
    let counter = Arc::new(RwLock::new(99));

    // 创建一个线程持有读锁
    let read_handle = {
        let counter = Arc::clone(&counter);
        thread::spawn(move || {
            // 获取读锁
            let num = counter.read().unwrap();
            println!("Reader {:?}: {}", thread::current().id(), *num);

            // 休眠模拟读取操作
            thread::sleep(std::time::Duration::from_secs(3));
        })
    };

    // 创建一个线程请求写锁
    let write_handle = {
        let counter = Arc::clone(&counter);
        thread::spawn(move || {
            // 休眠一小段时间，确保读锁已经被获取
            // thread::sleep(std::time::Duration::from_secs(1));

            // 尝试获取写锁
            // 注意：这里会等待读锁被释放
            let mut num = counter.write().unwrap();
            *num += 1;
            println!(
                "Writer {:?}: Incremented counter to {}",
                thread::current().id(),
                *num
            );
        })
    };

    // 等待读取线程和写入线程完成
    read_handle.join().unwrap();
    write_handle.join().unwrap();
}

pub fn rwmutex_exp_read_wait() {
    // 创建一个可共享的可变整数，使用RwLock包装
    let counter = Arc::new(RwLock::new(333));

    // 创建一个线程持有读锁
    let read_handle = {
        let counter = counter.clone();
        thread::spawn(move || {
            // 获取读锁
            let num = counter.read().unwrap();
            println!("Reader#1: {}", *num);

            // 休眠模拟读取操作
            thread::sleep(std::time::Duration::from_secs(4));
        })
    };

    // 创建一个线程请求写锁
    let write_handle = {
        let counter = counter.clone();
        thread::spawn(move || {
            // 休眠一小段时间，确保读锁已经被获取
            thread::sleep(std::time::Duration::from_secs(1));

            // 尝试获取写锁
            let mut num = counter.write().unwrap();
            *num += 1;
            println!("Writer : Incremented counter to {}", *num);
        })
    };

    // 创建一个线程请求读锁
    let read_handle_2 = {
        let counter = counter.clone();
        thread::spawn(move || {
            // 休眠一小段时间，确保写锁已经被获取
            thread::sleep(std::time::Duration::from_secs(2));

            // 尝试获取读锁
            let num = counter.read().unwrap();
            println!("Reader#2: {}", *num);
        })
    };

    // 等待读取线程和写入线程完成
    read_handle.join().unwrap();
    write_handle.join().unwrap();
    read_handle_2.join().unwrap();
}

pub fn rwmutex_exp_dead_lock() {
    // 创建一个可共享的可变整数，使用RwLock包装
    let counter = Arc::new(RwLock::new(444));
    // 创建一个线程持有读锁，尝试获取写锁
    let read_and_write_handle = {
        let counter = Arc::clone(&counter);
        thread::spawn(move || {
            // 获取读锁
            let num = counter.read().unwrap();
            println!("Reader {:?}: {}", thread::current().id(), *num);

            // 尝试获取写锁，这会导致死锁
            let mut num = counter.write().unwrap();
            *num += 1;
            println!(
                "Reader {:?}: Incremented counter to {}",
                thread::current().id(),
                *num
            );
        })
    };

    // 创建一个线程持有写锁，尝试获取读锁
    let write_and_read_handle = {
        let counter = Arc::clone(&counter);
        thread::spawn(move || {
            // 获取写锁
            let mut num = counter.write().unwrap();
            *num += 1;
            println!(
                "Writer {:?}: Incremented counter to {}",
                thread::current().id(),
                *num
            );

            // 尝试获取读锁，这会导致死锁
            let num = counter.read().unwrap();
            println!("Writer {:?}: {}", thread::current().id(), *num);
        })
    };
    let (s, r) = bounded(0);
    // 等待线程完成
    thread::spawn(move || {
        read_and_write_handle.join().unwrap();
        write_and_read_handle.join().unwrap();
        let _ = s.send(());
    });
    //3秒超时
    if let Ok(_msg) = r.recv_timeout(Duration::from_secs(3)) {
        println!("Task end!");
    } else {
        println!("Time out");
    };
}
