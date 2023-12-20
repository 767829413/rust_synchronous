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

/*
1. 创建线程

Rust 标准库std::thread[3] crate 提供了线程相关的函数。正如上面所说，一个 Rust 程序执行的会启动一个进程，这个进程会包含一个或者多个线程，Rust 中的线程是纯操作的系统的线程，拥有自己的栈和状态。 线程之间的通讯可以通过 channel[4]，就像 Go 语言中的 channel 的那样，也可以通过一些同步原语[5])。
*/
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

/*
2. Thread Builder

通过 Builder 你可以对线程的初始状态进行更多的控制，比如设置线程的名称、栈大大小等等。
提供了 spawn 开启一个线程，同时还提供了 spawn_scoped开启scoped thread ，
一个实验性的方法 spawn_unchecked ,提供更宽松的声明周期的绑定，调用者要确保引用的对象丢弃之前线程的 join 一定要被调用，
或者使用``static`声明周期，因为是实验性的方法，不过多介绍，一个简单的例子如下:
*/
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
3. 获取当前线程信息

因为线程是操作系统最小的调度和运算单元，所以一段代码的执行隶属于某个线程。
如何获得当前的线程呢？通过 thread::current()
它会返回一个Thread对象，你可以通过它获得线程的ID和name:
*/
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
4. 并发数和当前线程数

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
5. sleep 和 park

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
6. scoped thread

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
7. ThreadLocal

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
8. Move

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

/*
9. 控制新建的线程

从上面所有的例子中，貌似没有办法控制创建的子线程，只能傻傻等待它的执行或者忽略它的执行，并没有办法中途停止它，或者告诉它停止。Go 创建的 goroutine 也有类似的问题，但是 Go 提供了Context.WithCancel 和 channel ，父 goroutine 可以传递给子 goroutine 信号。
Rust 也可以实现类似的机制，我们可以使用以后讲到的 mpsc 或者 spsc 或者 oneshot 等类似的同步原语进行控制，也可以使用这个 crate:thread-control:

通过 make_pair 生成一对对象 flag,control ,就像破镜重圆的两块镜子心心相惜，或者更像处于纠缠态的两个量子，其中一个量子的变化另外一个量子立马感知。
这里 control 交给父进程进行控制，你可以调用 stop 方法触发信号，这个时候flag.alive()就会变为 false。如果子线程panickled,可以通过 control.is_interrupted() == true 来判断。
*/
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

/*
10. 设置线程优先级

通过 crate thread-priority可以设置线程的优先级。

因为 Rust 的线程都是纯的操作系统的优先级，现代的操作系统的线程都有优先级的概念，所以可以通过系统调用等方式设置优先级，唯一一点不好的就是各个操作系统的平台的优先级的数字和范围不一样。当前这个库支持以下的平台：Linux Android DragonFly FreeBSD OpenBSD NetBSD macOS Windows
*/

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

/*
11. 设置 affinity

可以将线程绑定在一个核上或者几个核上。
有个较老的 crate [core_affinity](core_affinity),但是它只能将线程绑定到一个核上，
如果要绑定到多个核上，可以使用 crate affinity 不支持 MacOS

这个例子我们把当前线程绑定到偶数的核上。

绑核是在极端情况提升性能的有效手段之一，
将某几个核只给我们的应用使用，可以让这些核专门提供给我们的业务服务，
既提供了 CPU 资源隔离，还提升了性能。尽量把线程绑定在同一个 NUMA 节点的核上。
*/
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

/*
12. Panic

Rust 中致命的逻辑错误会导致线程 panic, 出现 panic 是线程会执行栈回退，运行解构器以及释放拥有的资源等等。Rust 可以使用 catch_unwind 实现类似 try/catch 捕获 panic 的功能，或者 resume_unwind 继续执行。如果 panic 没有被捕获，那么线程就会退出，通过 JoinHandle 可以检查这个错误，如下面的代码：
*/
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

/*
13. crossbeam scoped thread

crossbeam 也提供了创建了scoped thread的功能，和标准库的 scope 功能类似，
但是它创建的scoped thread可以继续创建scoped thread

这里我们创建了两个子线程，子线程在 spawn 的时候，传递了一个 scope 值的，利用这个 scope 值
还可以在子线程中创建孙线程。
*/
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

/*
14. Rayon scoped thread

rayonscope in rayon - Rust (docs.rs)也提供了和 crossbeam 类似的机制，用来创建孙线程，子子孙孙线程：
*/
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

/*
15. send_wrapper

跨线程的变量必须实现 Send,否则不允许在跨线程使用，比如下面的代码：

pub fn wrong_send() {
    let counter = Rc::new(42);

    let (sender, receiver) = channel();

    let _t = thread::spawn(move || {
        sender.send(counter).unwrap();
    });

    let value = receiver.recv().unwrap();

    println!("received from the main thread: {}", value);
}

因为 Rc 没有实现 Send,所以它不能直接在线程间使用。
因为两个线程使用的 Rc 指向相同的引用计数值，它们同时更新这个引用计数，
并且没有使用原子操作，可能会导致意想不到的行为。
可以通过 Arc 类型替换 Rc 类型，
也可以使用一个第三方的库，send_wrapperhttps://crates.io/crates/send_wrapper,对它进行包装，以便实现Sender: Send .
*/
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

/*
16. Go 风格的启动线程

Go 开启新的 goroutine 的方法非常的简洁，通过 go func() {...}() 就启动了一个 goroutine，貌似同步的代码，却是异步的执行。

有一个第三方的库 go-spawn，可以提供 Go 类似的便利的方法:
*/
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
