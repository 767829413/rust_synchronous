use async_std::task;
use futures::{
    channel::mpsc,
    executor,
    executor::ThreadPool,
    future::{self},
    select,
    stream::StreamExt,
    try_join,
};
use futures_lite::future as fl_future;
use smol::future::{try_zip, zip, FutureExt};
use std::{future::Future as std_Future, thread};
use tokio::{
    join,
    time::{sleep, Duration},
};

async fn my_async_function() {
    println!("Starting async function");
    sleep(Duration::from_secs(1)).await;
    println!("Async function completed");
}

pub async fn order_test() {
    println!("Before calling async function");
    my_async_function().await;
    println!("After calling async function");
}

// `foo()`返回一个`Future<Output = u8>`,
// 当调用`foo().await`时，该`Future`将被运行，当调用结束后我们将获取到一个`u8`值
async fn foo1() -> u8 {
    5
}

fn bar1() -> impl std_Future<Output = u8> {
    // 下面的`async`语句块返回`Future<Output = u8>`
    async {
        let x: u8 = foo1().await;
        x + 5
    }
}

async fn foo2() -> Result<u8, String> {
    Ok(1)
}
async fn bar2() -> Result<u8, String> {
    Ok(1)
}

pub async fn return_test() {
    let fut = async {
        foo2().await?;
        bar2().await?;
        let x = bar1().await;
        // Ok(())
        Ok::<u8, String>(x) // 在这一行进行显式的类型注释
    };
    let res = fut.await.unwrap();
    println!("return_test res: {:?}", res);
}

fn download(str: &str) {
    println!("download start {} downloading...", str);
}

pub fn get_two_sites() {
    // 创建两个新线程执行任务
    let thread_one = thread::spawn(|| download("https://course.rs"));
    let thread_two = thread::spawn(|| download("https://fancy.rs"));

    // 等待两个线程的完成
    thread_one.join().expect("thread one panicked");
    thread_two.join().expect("thread two panicked");

    println!("get_two_sites All done!");
}

async fn download_async(str: &str) {
    println!("download_async start {} downloading...", str);
}

pub async fn get_two_sites_async() {
    // 创建两个不同的`future`，你可以把`future`理解为未来某个时刻会被执行的计划任务
    // 当两个`future`被同时执行后，它们将并发的去下载目标页面
    let future_one = download_async("https://www.foo.com");
    let future_two = download_async("https://www.bar.com");

    // 同时运行两个`future`，直至完成
    join!(future_one, future_two);

    println!("get_two_sites_async All done");
}

#[allow(unused)]
pub fn tokio_async() {
    // 创建了一个 Tokio 运行时rt
    let rt = tokio::runtime::Runtime::new().unwrap();

    // block_on方法在运行时上下文中执行一个异步任务
    // 这里简单地打印了一句话
    rt.block_on(async {
        println!("tokio_async Hello from tokio!");

        // 使用rt.spawn在运行时中异步执行另一个任务
        // 这个任务会在线程池中运行,而不会阻塞运行时
        // spawn返回一个JoinHandle,所以这里调用.await来等待任务结束
        rt.spawn(async {
            println!("tokio_async Hello from a tokio task!");
            println!("tokio_async in spawn")
        })
        .await
        .unwrap();
    });
    // 使用spawn_blocking在运行时中执行一个普通的阻塞任务
    // 这个任务会在线程池中运行,而不会阻塞运行时
    rt.spawn_blocking(|| println!("tokio_async in spawn_blocking"));
}

pub fn futures_async() {
    // 创建一个线程池pool
    let pool = ThreadPool::new().expect("futures_async Failed to build pool");
    // 创建一个无边界的通道tx和rx用来在任务间传递数据
    let (tx, rx) = mpsc::unbounded::<i32>();

    // 定义一个异步任务fut_values
    let fut_values = async {
        // 定义一个异步任务,这个任务会通过通道发送 0-99 的数字
        let fut_tx_result = async move {
            (0..100).for_each(|v| {
                tx.unbounded_send(v).expect("futures_async Failed to send");
            })
        };
        // 首先用spawn_ok在线程池中异步执行一个任务
        pool.spawn_ok(fut_tx_result);

        // 然后通过rx用map创建一个 Stream,它会将收到的数字乘 2
        // 用collect收集 Stream 的结果到一个 Vec
        let fut_values = rx.map(|v| v * 2).collect();

        fut_values.await
    };
    // block_on在主线程中执行这个异步任务并获取结果
    let values: Vec<i32> = executor::block_on(fut_values);

    println!("futures_async Values={:?}", values);
}

async fn hello_async() {
    println!("Hello, async world!");
}

pub fn futures_lite_example() {
    fl_future::block_on(hello_async());
}

async fn hi_async() {
    println!("Hi, async world!");
}

pub fn async_std_example() {
    task::block_on(hi_async());
}

pub fn smol_async_example() {
    smol::block_on(async { println!("Hello from smol") });
}

pub async fn futures_select_example() {
    let mut b = future::ready(6);
    let mut a = future::ready(4);

    let _ = select! {
        a_res = a => println!("a_res = {:?}", a_res),
        b_res = b => println!("b_res = {:?}", b_res),
    };
}

pub async fn futures_join_example() {
    let future1 = future::ready(6);
    let future2 = future::ready(4);

    let (res1, res2) = join!(future1, future2);

    println!("futures_join_example res1 = {:?}, res2 = {:?}", res1, res2);
}

pub async fn futures_try_join_example() {
    let a = async { Ok::<i32, i32>(1) };
    let b = async { Err::<u64, i32>(2) };

    println!("futures_try_join_example {:?}", try_join!(a, b));
}

pub fn smol_zip() {
    smol::block_on(async {
        let future1 = async { 1 };
        let future2 = async { 2 };

        let result = zip(future1, future2);
        println!("smol_zip: {:?}", result.await);

        let future1 = async { Ok::<i32, i32>(1) };
        let future2 = async { Err::<i32, i32>(2) };

        let result = try_zip(future1, future2).await;
        println!("smol_try_zip: {:?}", result);
    });
}
