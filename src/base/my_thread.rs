use affinity;
use crossbeam::thread::scope;
use go_spawn::{go, join};
use send_wrapper::SendWrapper;
use std::cell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use thread_control::*;
use thread_priority::*;

pub fn start_one_thread() {
    // 忽略thread::spawn 返回的 JoinHandle 值，那么这个新建的线程被称之为 detached
    let handle = thread::spawn(|| {
        println!(
            "start_one_thread:Hello from a thread! {:?}",
            thread::current().id()
        );
    });
    // 主程序退出的时候，新开的线程也会强制退出,通过 join 等待这些线程完成
    handle.join().unwrap();
}

// 创建线程，并返回线程的执行结果
pub fn start_one_thread_result() {
    let handle = thread::spawn(|| {
        println!("start_one_thread_result:Hello from a thread!");
        200
    });
    // join() 返回的是 Result 类型，如果线程 panicked了，那么它会返 Err ,否则它会返回 Ok(_) ,调用者还可以得到线程最后的返回值
    match handle.join() {
        Ok(v) => println!("start_one_thread_result:thread result: {}", v),
        Err(e) => println!("error: {:?}", e),
    }
}

// 创建两个线程
pub fn start_two_threads() {
    let handle1 = thread::spawn(|| {
        println!("start_two_threads:Hello from a thread1!");
    });

    let handle2 = thread::spawn(|| {
        println!("start_two_threads:Hello from a thread2!");
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

// 创建多个线程,使用一个Vector保存线程的handle
pub fn start_n_threads(n: i128) {
    let handles: Vec<_> = (0..n)
        .map(|i| {
            thread::spawn(move || {
                println!("start_n_threads:Hello from a thread{}!", i);
            })
        })
        .collect();

    // handles.into_iter().for_each(|h| h.join().unwrap());

    for handle in handles {
        handle.join().unwrap();
    }
}

pub fn start_one_thread_by_builder() {
    let builder = thread::Builder::new()
        .name("foo".into()) // 设置线程的名称
        .stack_size(32 * 1024); // 设置线程的栈大小

    // spawn 开启一个线程，同时还提供了 spawn_scoped 开启 scoped thread
    let handler = builder
        .spawn(|| {
            println!("start_one_thread_by_builder:Hello from a thread!");
        })
        .unwrap();

    handler.join().unwrap();
}

pub fn current_thread() {
    let current_thread = thread::current();
    println!(
        "current thread: {:?},{:?}",
        current_thread.id(),
        current_thread.name()
    );

    let builder = thread::Builder::new()
        .name("foo".into())
        .stack_size(32 * 1024);

    let handler = builder
        .spawn(|| {
            let current_thread = thread::current();
            println!(
                "child thread: {:?},{:?}",
                current_thread.id(),
                current_thread.name()
            );
        })
        .unwrap();

    handler.join().unwrap();
}

pub fn cpu_info() {
    let count = thread::available_parallelism().unwrap().get();
    println!("cpu_info:current cpu number: {}", count);
    // affinity(不支持 MacOS) crate 可以提供当前的 CPU 核数
    let cores: Vec<usize> = (0..affinity::get_core_num()).step_by(2).collect();
    println!(
        "cpu_info:affinity::get_core_num:current cores : {:?}",
        &cores
    );
    // 推荐使用 num_cpus 获取 CPU 的核数（逻辑核）
    let num = num_cpus::get();
    println!("cpu_info:num_cpus::get:current cores: {}", num);
}

pub fn thread_info() {
    let count = thread::available_parallelism().unwrap().get();
    println!("thread_info:available_parallelism: {}", count);

    if let Some(count) = num_threads::num_threads() {
        println!("thread_info:num_threads: {}", count);
    } else {
        println!("thread_info:num_threads: not supported");
    }

    // let count = thread_amount::thread_amount();
    // if !count.is_none() {
    //     println!("thread_amount: {}", count.unwrap());
    // }

    let count = num_cpus::get();
    println!("thread_info:num_cpus: {}", count);
}

pub fn thread_park() {
    let parked_thread = thread::Builder::new()
        .spawn(|| {
            println!("child Parking thread");
            thread::park();
            println!("child Thread unparked");
        })
        .unwrap();

    thread::sleep(Duration::from_millis(10));

    println!("curent Unpark the thread");
    parked_thread.thread().unpark();
    parked_thread.join().unwrap();
}

pub fn start_thread_with_sleep() {
    let handle1 = thread::spawn(|| {
        thread::sleep(Duration::from_millis(2000));
        println!("Hello from a thread3!");
    });

    let handle2 = thread::spawn(|| {
        thread::sleep(Duration::from_millis(1000));
        println!("Hello from a thread4!");
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

pub fn start_thread_with_yield_now() {
    let handle1 = thread::spawn(|| {
        thread::yield_now();
        println!("yield_now!");
    });

    let handle2 = thread::spawn(|| {
        thread::yield_now();
        println!("yield_now in another thread!");
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

pub fn thread_park_sleep() {
    let handle1 = || {
        let handle = thread::spawn(|| {
            thread::park();
            println!("Hello from a park thread!");
        });

        thread::sleep(Duration::from_millis(1000));

        handle.thread().unpark();

        handle.join().unwrap();
    };

    let handle2 = || {
        let handle = thread::spawn(|| {
            thread::sleep(Duration::from_millis(1000));
            thread::park();
            println!("Hello from a park thread in case of unpark first!");
        });

        handle.thread().unpark();

        handle.join().unwrap();
    };

    // 预先调用一股脑的 unpark 多次，然后再一股脑的调用 park 行不行,答案是不行。
    // 因为一个线程只有一个令牌，这个令牌或者存在或者只有一个，多次调用 unpark 也是针对一个令牌进行的的操作，上面的代码会导致新建的那个线程一直处于 parked 状态
    let handle3 = || {
        let handle = thread::spawn(|| {
            thread::sleep(Duration::from_millis(1000));
            thread::park();
            // thread::park();
            // thread::park();
            println!("Hello from a park thread in case of unpark first!");
        });
        handle.thread().unpark();
        // handle.thread().unpark();
        // handle.thread().unpark();
        handle.join().unwrap();
    };
    handle1();
    handle2();
    handle3();
}

pub fn start_scoped_threads() {
    let mut a = vec![1, 2, 3];
    let mut x = 0;

    thread::scope(|s| {
        s.spawn(|| {
            println!("hello from the first scoped thread");
            dbg!(&a);
        });
        s.spawn(|| {
            println!("hello from the second scoped thread");
            x += a[0] + a[2];
        });
        println!("hello from the main thread");
    });

    // After the scope, we can modify and access our variables again:
    a.push(4);
    println!("x: {} a:{:?}", x, a);
    assert_eq!(x, a.len());
}

pub fn start_threads_with_threadlocal() {
    // 定义了一个 Thread_local key: COUNTER。
    thread_local!(static COUNTER: cell::RefCell<u32> = cell::RefCell::new(1));

    COUNTER.with(|c| {
        *c.borrow_mut() = 2;
    });

    // 在外部线程和两个子线程中使用with 修改了COUNTER
    // 但是修改COUNTER只会影响本线程
    let handle1 = thread::spawn(move || {
        COUNTER.with(|c| {
            *c.borrow_mut() = 3;
        });

        COUNTER.with(|c| {
            println!("Hello from a handle1, c={}!", *c.borrow());
        });
    });

    let handle2 = thread::spawn(move || {
        COUNTER.with(|c| {
            *c.borrow_mut() = 4;
        });

        COUNTER.with(|c| {
            println!("Hello from a handle2, c={}!", *c.borrow());
        });
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    // 最后外部线程输出的COUNTER 的值是 2
    // 尽管两个子线程修改了 COUNTER 的值为 3 和 4
    COUNTER.with(|c| {
        println!("Hello from main, c={}!", *c.borrow());
    });
}

pub fn start_one_thread_with_move() {
    let mut x = 100;

    let handle = thread::spawn(move || {
        x += 1;
        println!("Hello from a thread with move, x={}!", x);
    });

    handle.join().unwrap();

    let handle = thread::spawn(move || {
        x += 1;
        println!("Hello from a thread with move again, x={}!", x);
    });
    handle.join().unwrap();

    let handle = thread::spawn(|| {
        println!("Hello from a thread without move");
    });
    handle.join().unwrap();
    println!("this is main, x={}!", x);
}

pub fn _start_one_thread_with_move2() {
    let x = vec![1, 2, 3];

    let handle = thread::spawn(move || {
        println!("Hello from a thread with move, x={:?}!", x);
    });

    handle.join().unwrap();

    // x 的所有权已经转移给第一个子线程了
    // let handle = thread::spawn(move|| {
    //     println!("Hello from a thread with move again, x={:?}!", x);
    // });
    // handle.join().unwrap();

    let handle = thread::spawn(|| {
        println!("Hello from a thread without move");
    });
    handle.join().unwrap();
}

pub fn control_thread() {
    let (flag, control) = make_pair();
    let handle = thread::spawn(move || {
        while flag.alive() {
            thread::sleep(Duration::from_millis(100));
            println!("I'm alive!");
        }
        println!("I'm out!");
    });

    thread::sleep(Duration::from_millis(100));
    assert_eq!(control.is_done(), false);
    control.stop(); // Also you can `control.interrupt()` it
    handle.join().unwrap();

    assert_eq!(control.is_interrupted(), false);
    assert_eq!(control.is_done(), true);

    println!("This thread is stopped")
}

pub fn start_thread_with_priority() {
    let handle1 = thread::spawn(|| {
        assert!(set_current_thread_priority(ThreadPriority::Min).is_ok());
        println!("Hello from a thread5!");
    });

    let handle2 = thread::spawn(|| {
        assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());
        println!("Hello from a thread6!");
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    assert!(
        // 设置一个特定值
        set_current_thread_priority(ThreadPriority::Crossplatform(0.try_into().unwrap())).is_ok()
    );
    // assert!(
    //     // 设置特定平台的优先级值：
    //     set_current_thread_priority(ThreadPriority::Os(WinAPIThreadPriority::Lowest.into()))
    //         .is_ok()
    // );
}

// 提供了一个 ThreadBuilder,类似标准库的 ThreadBuilder,只不过增加设置优先级的能力
// thread_priority::ThreadBuilderExt; 扩展标准库的ThreadBuilder支持设置优先级。
pub fn thread_builder() {
    let thread1 = ThreadBuilder::default()
        .name("MyThread")
        .priority(ThreadPriority::Max)
        .spawn(|result| {
            println!("Set priority result: {:?}", result);
            assert!(result.is_ok());
        })
        .unwrap();

    let thread2 = ThreadBuilder::default()
        .name("MyThread")
        .priority(ThreadPriority::Max)
        .spawn_careless(|| {
            println!("We don't care about the priority result.");
            assert!(std::thread::current().get_priority().is_ok());
            println!(
                "This thread's native id is: {:?}",
                std::thread::current().get_native_id()
            );
        })
        .unwrap();

    thread1.join().unwrap();
    thread2.join().unwrap();
}

#[cfg(not(target_os = "macos"))]
pub fn use_affinity() {
    // Select every second core
    let cores: Vec<usize> = (0..affinity::get_core_num()).step_by(2).collect();
    println!("Binding thread to cores : {:?}", &cores);

    affinity::set_thread_affinity(&cores).unwrap();
    println!(
        "Current thread affinity : {:?}",
        affinity::get_thread_affinity().unwrap()
    );
}

pub fn panic_example() {
    println!("Hello, world!");
    let h = std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        panic!("boom");
    });
    let r = h.join();
    match r {
        Ok(r) => println!("All is well! {:?}", r),
        Err(e) => println!("Got an error! {:?}", e),
    }
    println!("Exiting main!")
}

// 被捕获，外部的 handle 是检查不到这个 panic
// 通过 scope 生成的 scope thread，任何一个线程 panic,如果未被捕获，那么 scope 就会返回这个错误
pub fn panic_caught_example() {
    println!("Hello, panic_caught_example !");
    let h = std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let result = std::panic::catch_unwind(|| {
            panic!("boom");
        });
        println!("panic caught, result = {}", result.is_err()); // true
    });

    let r = h.join();
    match r {
        Ok(r) => println!("All is well! {:?}", r), // here
        Err(e) => println!("Got an error! {:?}", e),
    }

    println!("Exiting main!")
}

pub fn crossbeam_scope() {
    let mut a = vec![1, 2, 3];
    let mut x = 0;

    scope(|s| {
        s.spawn(|_| {
            println!("hello from the first crossbeam scoped thread");
            dbg!(&a);
        });
        s.spawn(|s| {
            println!("hello from the second crossbeam scoped thread");
            x += a[0] + a[2];
            s.spawn(|_| {
                println!("hello son son son thread");
            });
        });
        println!("hello from the main thread");
    })
    .unwrap();

    // After the scope, we can modify and access our variables again:
    a.push(4);
    assert_eq!(x, a.len());
}

pub fn rayon_scope() {
    let mut a = vec![1, 2, 3];
    let mut x = 0;

    rayon::scope(|s| {
        s.spawn(|_| {
            println!("hello from the first rayon scoped thread");
            dbg!(&a);
        });
        s.spawn(|s| {
            println!("hello from the second rayon scoped thread");
            x += a[0] + a[2];
            s.spawn(|_| {
                println!("sssssssssssssss");
            });
        });
        println!("hello from the main thread");
    });

    // fifo 的 scope thread。
    rayon::scope_fifo(|s| {
        s.spawn_fifo(|s| {
            // task s.1
            s.spawn_fifo(|_| {
                // task s.1.1
                rayon::scope_fifo(|t| {
                    t.spawn_fifo(|_| ()); // task t.1
                    t.spawn_fifo(|_| ()); // task t.2
                });
            });
        });
        s.spawn_fifo(|_| { // task s.2
        });
        // point mid
    }); // point end

    // After the scope, we can modify and access our variables again:
    a.push(4);
    assert_eq!(x, a.len());
}

pub fn send_wrapper() {
    let wrapped_value = SendWrapper::new(Rc::new(10000));
    let (sender, receiver) = channel();

    let _t = thread::spawn(move || {
        sender.send(wrapped_value).unwrap();
    });

    let wrapped_value = receiver.recv().unwrap();
    let value = wrapped_value.deref();
    println!("received from the main thread: {}", value);
}

pub fn go_thread() {
    let counter = Arc::new(AtomicI64::new(0));
    let counter_cloned = counter.clone();

    // Spawn a thread that captures values by move.
    go! {
        for _ in 0..100 {
            counter_cloned.fetch_add(1, Ordering::SeqCst);
        }
    }

    // Join the most recent thread spawned by `go_spawn` that has not yet been joined.
    assert!(join!().is_ok());
    assert_eq!(counter.load(Ordering::SeqCst), 100);
}
