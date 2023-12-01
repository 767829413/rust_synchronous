use affinity;
use std::cell;
use std::thread;
use std::time::Duration;

/*
Rust 标准库std::thread[3] crate 提供了线程相关的函数。正如上面所说，一个 Rust 程序执行的会启动一个进程，这个进程会包含一个或者多个线程，Rust 中的线程是纯操作的系统的线程，拥有自己的栈和状态。 线程之间的通讯可以通过 channel[4]，就像 Go 语言中的 channel 的那样，也可以通过一些同步原语[5])。
*/
// 创建线程
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

// Thread Builder
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

/*
因为线程是操作系统最小的调度和运算单元，所以一段代码的执行隶属于某个线程。
如何获得当前的线程呢？通过 thread::current()
它会返回一个Thread对象，你可以通过它获得线程的ID和name:
*/

// 获取当前线程信息
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

//  unpark 方法，唤醒被阻塞(parked)的线程
// park 和 unpark 用来阻塞和唤醒线程的方法，利用它们可以有效的利用 CPU,让暂时不满足条件的线程暂时不可执行
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

/*
并发能力是一种资源，一个机器能够提供并发的能力值，这个数值一般等价于计算机拥有的 CPU 数（逻辑的核数），但是在虚机和容器的环境下，程序可以使用的 CPU 核数可能受到限制。
可以通过 available_parallelism 获取当前的并发数：
*/

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
/*
如果想获得当前进程的线程数，比如在一些性能监控收集指标的时候，你可以使用num_threads crate, 实际测试num_threads 不支持 windows，所以你可以使用thread-amount代替。

(Rust 生态圈就是这样，有很多功能相同或者类似的 crate,你可能需要花费时间进行评估和比较,不像 Go 生态圈，优选标准库的包，如果没有，生态圈中一般会有一个或者几个高标准的大家公认的库可以使用。相对而言，Rust 生态圈就比较分裂,这一点在选择异步运行时或者网络库的时候感受相当明显。)
*/

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

/*
有时候我们需要将当前的业务暂停一段时间，可能是某些条件不满足.
比如实现 spinlock,或者是想定时的执行某些业务，如 cron 类的程序
这个时候我们可以调用 thread::sleep 至少保证当前线程sleep指定的时间
因为它会阻塞当前的线程，所以不要在异步的代码中调用它.
*/

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

/*
。如果时间设置为 0,不同的平台处理是不一样的，Unix 类的平台会立即返回，不会调用nanosleep 系统调用，而 Windows 平台总是会调用底层的 Sleep 系统调用。如果只是想让渡出时间片，你不用设置时间为 0，而是调用 yield_now 函数即可：
*/

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

/*
如果在休眠时间不确定的情况下，我们想让某个线程休眠，将来在某个事件发生之后，我们再主动的唤醒它，
可以使用 park 和 unpark 方法了。
你可以认为每个线程都有一个令牌( token ),最初该令牌不存在:

thread::park 将阻塞当前线程，直到线程的令牌可用。
此时它以原子操作的使用令牌。thread::park_timeout 执行相同的操作，但允许指定阻止线程的最长时间。
和 sleep 不同，它可以还未到超时的时候就被唤醒。

thread.upark 方法以原子方式使令牌可用（如果尚未可用）。
由于令牌初始不存在，unpark 会导致紧接着的 park 调用立即返回。

 park 函数的调用并不保证线程永远保持 parked 状态，调用者应该小心这种可能性
*/

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

/*
thread::scope 函数提供了创建scoped thread的可能性
scoped thread不同于上面我们创建的thread, 它可以借用scope外部的非 'static' 数据
使用 thread::scope 函数提供的Scope的参数，可以创建(spawn) scoped thread
创建出来的scoped thread如果没有手工调用 join ,在这个函数返回前会自动 join

pub fn _wrong_start_threads_without_scoped() {
    let mut a = vec![1, 2, 3];
    let mut x = 0;

    thread::spawn(move || {
        println!("hello from the first scoped thread");
        dbg!(&a);
    });
    // 线程外的 a 没有办法 move 到两个 thread 中，即使 move 到一个 thread,外部的线程也没有办法再使用它了。
    // 为了解决这个问题，我们可以使用 scoped thread:
    thread::spawn(move || {
        println!("hello from the second scoped thread");
        x += a[0] + a[2];
    });
    println!("hello from the main thread");

    // After the scope, we can modify and access our variables again:
    a.push(4);
    assert_eq!(x, a.len());
}

下面调用了 thread::scope 函数，并使用s参数启动了两个scoped thread, 它们使用了外部的变量 a 和 x
因为我们对 a 只是读，对 x 只有单线程的写，所以不用考虑并发问题。
thread::scope 返回后，两个线程已经执行完毕，所以外部的线程又可以访问变量了。
标准库的scope功能并没有进一步扩展，事实上我们可以看到，在新的scoped thread,我们是不是还可以启动新的scope线程?
这样实现类似 java 一样的 Fork-Join 父子线程。不过如果你有这个需求，可以通过第三方的库实现。
*/
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

/*
ThreadLocal 为 Rust 程序提供了 thread-local storage 的实现。
TLS(thread-local storage)可以存储数据到全局变量中，每个线程都有这个存储变量的副本，线程不会分享这个数据，副本是线程独有的，所以对它的访问不需要同步控制。
Java 中也有类似的数据结构，但是 Go 官方不建议实现 goroutine-local storage。

thread-local key 拥有它的值，并且在线程退出此值会被销毁。我们使用 thread_local! 宏创建 thread-local key,它可以包含 'static 的值。
它使用 with 访问函数去访问值。如果我们想修值，我们还需要结合 Cell 和 RefCell ,可以理解它们为不可变变量提供内部可修改性。

定义了一个 Thread_local key: COUNTER。
在外部线程和两个子线程中使用with 修改了COUNTER,但是修改COUNTER只会影响本线程。
可以看到最后外部线程输出的COUNTER 的值是 2， 尽管两个子线程修改了 COUNTER 的值为 3 和 4。
*/
pub fn start_threads_with_threadlocal() {
    thread_local!(static COUNTER: cell::RefCell<u32> = cell::RefCell::new(1));

    COUNTER.with(|c| {
        *c.borrow_mut() = 2;
    });

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

    COUNTER.with(|c| {
        println!("Hello from main, c={}!", *c.borrow());
    });
}

/*
在前面的例子中，可以看到有时候在调用 thread::spawn 的时候，有时候会使用 move ，有时候没有使用 move

使不使用 move 依赖相应的闭包是否要获取外部变量的所有权。
如果不获取外部变量的所有权，则可以不使用 move ,大部分情况下我们会使用外部变量，所以这里 move 更常见:
*/

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

/*
当在线程中引用变量 x 时，使用了 move ,当我们没引用变量，我们没使用 move 。

这里有一个问题， move 不是把 x 的所有权交给了第一个子线程了么，为什么第二个子线程依然可以move并使用 x 呢？

这是因为 x 变量是 i32 类型的，它实现了 Copy trait,实际move的时候实际复制它的的值，如果我们把 x 替换成一个未实现 Copy 的类型，类似的代码就无法编译了，因为 x 的所有权已经转移给第一个子线程了:
*/

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
