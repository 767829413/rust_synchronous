# 线程池

## 介绍

`线程池是一种并发编程的设计模式:`

- 由一组预先创建的线程组成,用于执行多个任务.

- 线程池的主要作用是在任务到达时,重用已创建的线程,避免频繁地创建和销毁线程,从而提高系统的性能和资源利用率.

- 线程池通常用于需要处理大量短期任务或并发请求的应用程序.

`线程池的优势包括：`

1. 减少线程创建和销毁的开销：线程的创建和销毁是一项昂贵的操作,线程池通过重用线程减少了这些开销,提高了系统的响应速度和效率.

2. 控制并发度：线程池可以限制同时执行的线程数量,从而有效控制系统的并发度,避免资源耗尽和过度竞争.

3. 任务调度和负载均衡：线程池使用任务队列和调度算法来管理和分配任务,确保任务按照合理的方式分配给可用的线程,实现负载均衡和最优的资源利用.

## rust关于线程池的相关介绍

通过一些介绍展示使用 rust 对线程池的相关crate

```rust
use crate::base::util;
use fast_threadpool::ThreadPoolConfig;
use rayon::ThreadPoolBuilder;
use rusty_pool::ThreadPool;
use std::{
    sync::{
        atomic::{AtomicI32, AtomicUsize, Ordering},
        mpsc::channel,
        Arc, Barrier,
    },
    thread,
    time::Duration,
};
```

### rayon 线程池

```text
Rayon 是 Rust 中的一个并行计算库,它可以让你更容易地编写并行代码,以充分利用多核处理器.
Rayon 提供了一种简单的 API,允许你将迭代操作并行化,从而加速处理大规模数据集的能力.
除了这些核心功能外,它还提供构建线程池的能力.

rayon::ThreadPoolBuilder 是 Rayon 库中的一个结构体,用于自定义和配置 Rayon 线程池的行为.线程池是 Rayon 的核心部分,它管理并行任务的执行.通过使用 ThreadPoolBuilder,你可以根据你的需求定制 Rayon 线程池的行为,以便更好地适应你的并行计算任务.在创建线程池之后,你可以使用 Rayon 提供的方法来并行执行任务,利用多核处理器的性能优势.
```

`与直接 spawn thread 相比,使用 rayon 的线程池有以下优点:`

1. 线程可重用,避免频繁创建/销毁线程的开销

2. 线程数可配置,一般根据 CPU 核心数设置

3. 避免大量线程造成资源竞争问题

```rust
// ThreadPoolBuilder 是以设计模式中的构建者模式设计的,以下是一些 ThreadPoolBuilder 的主要方法：
pub fn rayon_build_example() {
    // new() 方法：创建一个新的 ThreadPoolBuilder 实例
    let builder = ThreadPoolBuilder::new();

    // num_threads() 方法：设置线程池的线程数量.
    // 可以通过这个方法指定线程池中的线程数,以控制并行度.
    // 默认情况下,Rayon 会根据 CPU 内核数量自动设置线程数.
    let builder = builder.num_threads(1); // 设置线程池有 4 个线程

    // thread_name() 方法：为线程池中的线程设置一个名称
    // 可以帮助在调试时更容易识别线程
    let builder = builder.thread_name(|i| format!("worker-{}", i));
    // build() 方法：通过 build 方法来创建线程池.
    // 这个方法会将之前的配置应用于线程池并返回一个 rayon::ThreadPool 实例.
    let pool = builder.build().unwrap(); // 使用 unwrap() 来处理潜在的错误

    // build_global方法 通过build_global方法创建一个全局的线程池.
    // 不推荐主动调用这个方法初始化全局的线程池,使用默认的配置就好,记得全局的线程池只会初始化一次.
    // let pool = builder.build_global().unwrap();

    // 其他方法：ThreadPoolBuilder 还提供了其他一些方法,用于配置线程池的行为,如:
    // stack_size() 用于设置线程栈的大小.
    // 还提供了一些回调函数的设置:
    // start_handler() 用于设置线程启动时的回调函数等.
    // spawn_handler实现定制化的函数来产生线程.
    // panic_handler提供对 panic 处理的回调函数.
    // exit_handler提供线程退出时的回调.
    let size: usize = 10;
    let n = pool.install(|| fib(size)); // pool.install() 在线程池中运行 fib
    println!("fib({}) = {}", size, n);
}

fn fib(n: usize) -> usize {
    if n == 0 || n == 1 {
        return n;
    }
    // rayon::join 用于并行执行两个函数并等待它们的结果.
    // 使得可以同时执行两个独立的任务,然后等待它们都完成,以便将它们的结果合并到一起.
    let (a, b) = rayon::join(|| fib(n - 1), || fib(n - 2));
    a + b
}

// 使用build_scoped的代码
pub fn rayon_threadpool2() {
    // 这一行代码使用了 scoped_tls 库的宏 scoped_thread_local! 来创建一个静态的线程本地存储变量 POOL_DATA
    // 其类型是 Vec<i32>.这意味着每个线程都可以拥有自己的 POOL_DATA 值,而这些值在不同线程之间是相互独立的
    scoped_tls::scoped_thread_local!(static POOL_DATA: Vec<i32>);

    // 创建了一个 Vec<i32> 类型的变量 pool_data,其中包含了整数 1、2 和 3.
    let pool_data = vec![1, 2, 3];
    // 用来检查在线程本地存储中是否已经设置了 POOL_DATA.在此初始阶段
    // 我们还没有为它的任何线程分配值,因此应该返回 false
    // 我们尚未分配任何 TLS 数据.
    assert!(!POOL_DATA.is_set());

    // 开始构建一个 Rayon 线程池
    rayon::ThreadPoolBuilder::new()
        .build_scoped(
            // 用于定义每个线程在启动时要执行的操作.
            // 它将 pool_data 的引用设置为 POOL_DATA 的线程本地存储值,并在一个新的线程中运行 thread.run()
            // 这个闭包的目的是为每个线程设置线程本地存储数据
            // 在 TLS 中为每个线程借用 "pool_data".
            |thread| POOL_DATA.set(&pool_data, || thread.run()),
            // 定义了线程池启动后要执行的操作.
            // 它使用 pool.install 方法来确保在线程池中的每个线程中都能够访问到线程本地存储的值
            // 并且执行了一个断言来验证 POOL_DATA 在这个线程的线程本地存储中已经被设置
            // 执行一些需要 TLS 数据的工作.
            |pool| {
                pool.install(|| {
                    if POOL_DATA.is_set() {
                        // 使用POOL_DATA设置的值
                        POOL_DATA.with(|d| {
                            println!("thread local data: {:?}", d);
                        })
                    }
                })
            },
        )
        .unwrap();

    // 在线程池的作用域结束后,这一行代码用来释放 pool_data 变量.
    // 这是因为线程本地存储中的值是按线程管理的,所以在这个作用域结束后,需要手动释放 pool_data,
    // 以确保它不再被任何线程访问
    // 一旦我们返回,`pool_data` 就不再被借用.
    drop(pool_data);
}
```

### threadpool 库

```text
threadpool 是一个 Rust 库,用于创建和管理线程池,使并行化任务变得更加容易.线程池是一种管理线程的机制,它可以在应用程序中重用线程,以减少线程创建和销毁的开销,并允许您有效地管理并行任务.
```

`关于 threadpool 库的一些基本介绍：`

1. 创建线程池：
    - threadpool 允许您轻松创建线程池,可以指定线程池的大小（即同时运行的线程数量）.这可以确保您不会创建过多的线程,从而避免不必要的开销.

2. 提交任务：
    - 一旦创建了线程池,您可以将任务提交给线程池进行执行.这可以是任何实现了 FnOnce() 特质的闭包,通常用于表示您想要并行执行的工作单元.

3. 任务调度：
    - 线程池会自动将任务分发给可用线程,并在任务完成后回收线程,以便其他任务可以使用.这种任务调度可以减少线程创建和销毁的开销,并更好地利用系统资源.

4. 等待任务完成：
    - 可以等待线程池中所有任务完成,以确保在继续执行后续代码之前,所有任务都已完成.这对于需要等待并行任务的结果的情况非常有用.

5. 错误处理：
    - threadpool 提供了一些错误处理机制,以便您可以检测和处理任务执行期间可能发生的错误.

```rust
// 使用 threadpool 库创建一个线程池并提交任务
// 创建了一个包含 4 个线程的线程池,并向线程池提交了 8 个任务,每个任务计算一个数字的两倍并将结果发送到通道.
// 最后等待所有任务完成并打印结果
pub fn threadpool_example() {
    // 创建一个线程池,其中包含 4 个线程
    let pool = threadpool::ThreadPool::new(4);

    // 创建一个通道,用于接收任务的结果
    let (sender, receiver) = channel();

    // 提交一些任务给线程池
    for i in 0..8 {
        let sender = sender.clone();
        pool.execute(move || {
            let result = i * 2;
            sender.send(result).expect("发送失败");
        });
    }

    // 等待所有任务完成,并接收它们的结果
    for _ in 0..8 {
        let result = receiver.recv().expect("接收失败");
        println!("任务结果: {}", result);
    }
}

// threadpool + barrier 的例子
// 并发执行多个任务,并且使用 barrier 等待所有的任务完成.
// 注意任务数一定不能大于 worker 的数量,否则会导致死锁
pub fn threadpool_barrier_example() {
    // create at least as many workers as jobs or you will deadlock yourself
    let n_workers = 42;
    let n_jobs = 23;
    let pool = threadpool::ThreadPool::new(n_workers);
    let an_atomic = Arc::new(AtomicUsize::new(0));

    assert!(n_jobs <= n_workers, "too many jobs, will deadlock");

    // 创建一个barrier,等待所有的任务完成
    let barrier = Arc::new(Barrier::new(n_jobs + 1));
    for _ in 0..n_jobs {
        let barrier = barrier.clone();
        let an_atomic = an_atomic.clone();

        pool.execute(move || {
            // 执行一个很重的任务
            an_atomic.fetch_add(1, Ordering::Relaxed);

            // 等待其他线程完成
            barrier.wait();
        });
    }

    // 等待线程完成
    barrier.wait();
    assert_eq!(an_atomic.load(Ordering::SeqCst), /* n_jobs = */ 23);
}
```

### rusty_pool 库

`基于 crossbeam 多生产者多消费者通道实现的自适应线程池.它具有以下特点:`

- 核心线程池和最大线程池两种大小
- 核心线程持续存活,额外线程有空闲回收机制
- 支持等待任务结果和异步任务
- 首次提交任务时才创建线程,避免资源占用
- 当核心线程池满了时才会创建额外线程
- 提供了 JoinHandle 来等待任务结果
- 如果任务 panic,JoinHandle 会收到一个取消错误
- 开启 asyncfeature 时可以作为 futures executor 使用
  - spawn 和 try_spawn 来提交 future,会自动 polling
  - 否则可以通过 complete 直接阻塞执行 future

**总之,该线程池实现了自动扩缩容、空闲回收、异步任务支持等功能.**

```text
其自适应控制和异步任务的支持使其可以很好地应对突发大流量,而平时也可以节省资源.
从实现来看,作者运用了 crossbeam 通道等 Rust 并发编程地道的方式,代码质量很高.
所以这是一个非常先进实用的线程池实现,值得深入学习借鉴.可以成为我们编写弹性伸缩的 Rust 并发程序的很好选择
```

```rust
pub fn rusty_pool_example1() {
    // 创建 rusty_pool 线程池,默认配置
    let pool = rusty_pool::ThreadPool::default();

    // 循环提交 10 个打印任务到线程池
    for i in 1..10 {
        pool.execute(move || {
            println!("Hello from a rusty_pool!---work-{}", i);
        });
    }
    // 在主线程中调用 join,等待线程池内所有任务完成
    pool.join();
}

async fn some_async_fn(i: i32, j: i32) -> i32 {
    i + j
}
async fn other_async_fn(i: i32, j: i32) -> i32 {
    i - j
}

// 与之前的 threadpool 类似,rusty_pool 也提供了一个方便的线程池抽象,使用起来更简单些.
// 通过 complete 和 spawn 的结合,可以灵活地在线程池中同步或异步地执行 Future 任务.
// rusty_pool 通过内置的 async 运行时,很好地支持了 Future based 的异步编程.
// 可以利用这种方式来实现复杂的异步业务,而不需要自己管理线程和 Future
pub fn rusty_pool_example2() {
    // 创建默认的 rusty_pool 线程池
    let pool = ThreadPool::default();

    // 使用 pool.complete 来同步执行一个 async 块
    // 可以获取 async 块的返回值
    // complete 会阻塞直到整个 async 块完成
    let handle = pool.complete(async {
        // 在 async 块中可以使用 await 运行异步函数
        let a = some_async_fn(4, 6).await; // 10
        let b = some_async_fn(a, 3).await; // 13
        let c = other_async_fn(b, a).await; // 3
        some_async_fn(c, 5).await // 8
    });
    // 检验异步任务的结果
    println!("except:{} handle result: {}", 8, handle.await_complete());

    let count = Arc::new(AtomicI32::new(0));
    let clone = count.clone();
    // 使用 pool.spawn 来异步执行 async 块
    // spawn 会立即返回一个 JoinHandle
    // async 块会在线程池中异步执行
    pool.spawn(async move {
        let a = some_async_fn(3, 6).await; // 9
        let b = other_async_fn(a, 4).await; // 5
        let c = some_async_fn(b, 7).await; // 12
        clone.fetch_add(c, Ordering::SeqCst); // 这里通过 Atomics 变量来保存结果
    });
    // 在主线程中调用 join,等待异步任务完成
    pool.join();
    // 检验异步任务的结果
    println!(
        "except:{} count result: {}",
        12,
        count.load(Ordering::SeqCst)
    );
}

// 等待超时以及关闭线程池的例子
pub fn rusty_pool_example3() {
    println!("rusty_pool_example3 start-----------------");
    let pool = ThreadPool::default();
    for _ in 0..4 {
        pool.execute(|| {
            let start = util::now_timestamp_millis();
            thread::sleep(Duration::from_secs(1));
            println!(
                "rusty_pool_example3 fn1 end: {}",
                util::now_timestamp_millis() - start
            );
        })
    }

    // 等待所有线程变得空闲,即所有任务都完成,包括此线程调用join（）后由其他线程添加的任务,或者等待超时
    pool.join_timeout(Duration::from_secs(4));

    let count = Arc::new(AtomicI32::new(0));
    for _ in 0..3 {
        let clone = count.clone();
        pool.execute(move || {
            let start = util::now_timestamp_millis();
            thread::sleep(Duration::from_secs(1));
            clone.fetch_add(1, Ordering::SeqCst);
            println!(
                "rusty_pool_example3 fn2 end: {}",
                util::now_timestamp_millis() - start
            );
        });
    }
    // 关闭并删除此“ ThreadPool”的唯一实例（无克隆）,导致通道被中断,从而导致所有worker在完成当前工作后退出
    pool.shutdown_join();
    println!(
        "rusty_pool_example3 except: {} count result: {}",
        3,
        count.load(Ordering::SeqCst)
    );
}
```

### fast_threadpool 库

```text
这个线程池实现经过优化以获取最小化延迟.特别是,保证你在执行你的任务之前不会产生线程生成的成本.
新线程仅在工作线程的 "闲置时间"（例如,在返回作业结果后）期间生成.

唯一可能导致延迟的情况是 "可用" 工作线程不足.为了最小化这种情况的发生概率,这个线程池会不断保持一定数量的可用工作线程（可配置）.

这个实现允许你以异步方式等待任务的执行结果,因此你可以将其用作替代异步运行时的 spawn_blocking 函数.
```

```rust
pub fn fast_threadpool_example() {
    // 使用 default 配置创建线程池
    let threadpool =
        fast_threadpool::ThreadPool::start(ThreadPoolConfig::default(), ()).into_sync_handler(); // 将线程池转换为 sync_handler,用于同步提交任务
    let res = threadpool.execute(|_| 2 + 2); // 提交一个简单的计算任务到线程池
    if let Ok(result) = res {
        //主线程中收集结果并验证
        println!("fast_threadpool_example result:{}", result);
    } else {
        println!("fast_threadpool_example error");
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let threadpool = fast_threadpool::ThreadPool::start(ThreadPoolConfig::default(), ())
            .into_async_handler();
        let res = threadpool.execute(|_| 200 + 200).await.unwrap();
        println!("fast_threadpool_example result2:{}", res);
    });
}
```

### scoped_threadpool 库

在 Rust 多线程编程中,scoped 是一个特定的概念,指的是一种限定作用域的线程.

`scoped 线程的主要特征是:`

1. 线程的生命周期限定在一个代码块中,离开作用域自动停止
2. 线程可以直接访问外部状态而无需 channel 或 mutex
3. 借用检查器自动确保线程安全

一个典型的 scoped 线程池用法如下:

```rust
pool.scoped(|scope| {
  scope.execute(|| {
    // 可以直接访问外部状态
  });
}); // 作用域结束时,线程被Join
```

`scoped 线程的优点是:`

1. 代码简洁,无需手动同步线程
2. 作用域控制自动管理线程 lifetime
3. 借用检查确保安全

`scoped 线程适用于:`

1. 需要访问共享状态的短任务
2. 难以手动管理线程 lifetime 的场景
3. 对代码安全性要求高的场景

`scoped 线程池的主要特点:`

1. 线程可以直接访问外部状态,不需要 channel 或 mutex
2. 外部状态的借用检查自动进行
3. 线程池作用域结束时,自动等待所有线程完成

`相比全局线程池,scoped 线程池的优势在于:`

1. 代码更简洁,无需手动同步外部状态
2. 借用检查确保线程安全
3. 作用域控制自动管理线程 lifetime

`接下来可以扩展介绍:`

1. 在线程间共享不同类型的状态
2. scoped 线程池的配置选项

与其他线程池的比较使用案例,如并行计算等

**总之,scoped 线程在 Rust 中提供了一种更安全便捷的多线程模式,值得我们在多线程编程中考虑使用.**

```rust
// 使用 scoped_threadpool 库创建一个 scoped 线程池
pub fn scoped_threadpool() {
    // 首先创建一个 scoped 线程池,指定使用 4 个线程
    let mut pool = scoped_threadpool::Pool::new(4);
    // 定义一个向量 vec 作为外部共享状态
    let mut vec = vec![0, 1, 2, 3, 4, 5, 6, 7];

    // 在 pool.scoped 中启动线程,在闭包中可以访问外部状态 vec
    pool.scoped(|s| {
        // 每个线程读取 vec 的一个元素,并在线程内修改它
        for e in &mut vec {
            // 所有线程执行完成后,vec 的元素全部+1
            s.execute(move || {
                *e += 1;
            });
        }
    });
    println!(
        "scoped_threadpool excpect: {:?},actual {:?}",
        vec![1, 2, 3, 4, 5, 6, 7, 8],
        vec
    );
}
```

### scheduled_thread_pool 库

scheduled-thread-pool 是一个 Rust 库,它提供了一个支持任务调度的线程池实现.

`主要功能和用法:`

1. 支持定时执行任务,无需自己实现调度器
2. 提供一次性和重复调度两种方式
3. 基于线程池模型,避免线程重复创建销毁
4. 任务可随时取消

`scheduled 线程池的主要功能:`

1. 可以调度任务在未来的某时间点执行
2. 提供一次性调度和定期调度两种方式
3. 采用工作线程池模型,避免线程重复创建销毁

`相比普通线程池,scheduled 线程池的优势在于:`

1. 可以将任务延迟或定期执行,无需自己实现定时器
2. 调度功能内置线程池,无需自己管理线程
3. 可以直接使用调度语义,代码更简洁

```rust
// 使用 scheduled_thread_pool crate 创建一个可调度的线程池
pub fn scheduled_thread_pool() {
    // 设置一个channel,用于接收线程池任务执行结果
    let (sender, receiver) = channel();
    // 创建一个包含 4 个线程的 scheduled 线程池
    let pool = scheduled_thread_pool::ScheduledThreadPool::new(4);
    // 使用 pool.execute_after 在 1 秒后调度一个任务
    let handle = pool.execute_after(Duration::from_millis(1000), move || {
        // 任务中打印消息并向 channel 发送完成信号
        println!("Hello from a scheduled thread!");
        sender.send("done").unwrap();
    });

    let _ = handle;
    // 主线程在 channel 中接收信号,阻塞等待任务完成
    println!("scheduled_thread_pool result:{}", receiver.recv().unwrap());
}

```
