mod base;
use base::{my_async_await as maa, my_thread as mt, my_thread_pool as mtp};
use std::thread;

#[tokio::main]
async fn main() {
    // threads_base_example();
    // thread_pool_example();
    // 在运行时中异步执行任务
    tokio::spawn(async { println!("do work") });
    async_await_example().await;
}

// fn main() {
//     maa::tokio_async();
// }

// thread
#[allow(unused)]
fn threads_base_example() {
    mt::start_one_thread();
    mt::start_one_thread_result();
    mt::start_two_threads();
    mt::start_n_threads(12);
    mt::start_one_thread_by_builder();
    let current_thread = thread::current();
    println!(
        "current main thread: {:?},{:?}",
        current_thread.id(),
        current_thread.name()
    );
    mt::current_thread();
    mt::thread_park();
    mt::cpu_info();
    mt::thread_info();
    mt::start_thread_with_sleep();
    mt::start_thread_with_yield_now();
    mt::thread_park_sleep();
    mt::start_scoped_threads();
    mt::start_threads_with_threadlocal();
    mt::start_one_thread_with_move();
    mt::control_thread();
    mt::start_thread_with_priority();
    mt::thread_builder();
    mt::use_affinity();
    mt::panic_example();
    mt::panic_caught_example();
    mt::crossbeam_scope();
    mt::rayon_scope();
    mt::send_wrapper();
    mt::go_thread();
}

// thread pool
#[allow(unused)]
fn thread_pool_example() {
    mtp::rayon_build_example();
    mtp::rayon_threadpool2();
    mtp::threadpool_example();
    mtp::threadpool_barrier_example();
    mtp::rusty_pool_example1();
    mtp::rusty_pool_example2();
    mtp::rusty_pool_example3();
    mtp::fast_threadpool_example();
    mtp::scoped_threadpool();
    mtp::scheduled_thread_pool();
}

// async/await
#[allow(unused)]
async fn async_await_example() {
    maa::order_test().await;
    maa::return_test().await;
    maa::get_two_sites();
    maa::get_two_sites_async().await;
    maa::futures_async();
    maa::futures_lite_example();
    maa::async_std_example();
    maa::smol_async_example();
    maa::futures_select_example().await;
    maa::futures_join_example().await;
    maa::futures_try_join_example().await;
    maa::smol_zip();
}
