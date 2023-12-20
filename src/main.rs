mod base;
use base::my_thread as mt;
use base::my_thread_pool as mtp;
use std::thread;


// use tokio::time::{sleep, Duration};



fn main() {
    // threads_base_example();
    thread_pool_example();
}

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

// async fn my_async_function() {
//     println!("Starting async function");
//     sleep(Duration::from_secs(1)).await;
//     println!("Async function completed");
// }

// #[tokio::main]
// async fn main() {
//     println!("Before calling async function");
//     my_async_function().await;
//     println!("After calling async function");
// }