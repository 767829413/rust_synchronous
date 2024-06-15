mod base;
mod concurrency;
use base::{my_async_await as maa, my_thread as mt, my_thread_pool as mtp};
use concurrency::{
    my_atomic, my_barrier, my_box, my_cell, my_cow, my_mpsc, my_mutex, my_once, my_rc, my_set,
};
use std::thread;

#[cfg(target_os = "linux")]
mod mping;

fn main() {
    use std::env;
    env::set_var("RUST_LOG", "trace");
    env_logger::init(); // 初始化 env_logger

    let number_option: Option<u32> = Some(42);
    let string_option: Option<String> = number_option.map(|num| num.to_string());

    log::trace!("{:?}", string_option); // 使用日志宏记录信息

    let none_option: Option<u32> = None;
    let empty_string_option: Option<String> = none_option.map(|num| num.to_string());

    log::trace!("{:?}", empty_string_option); // 使用日志宏记录信息
}

// test some things
#[allow(unused)]
fn test() {
    /*
    _ = my_mping();
    // 创建了一个 Tokio 运行时rt
    let rt = tokio::runtime::Runtime::new().unwrap();
    let task = async {
        async_await_example().await
    };
    rt.block_on(task);
    maa::tokio_async();
     */
    my_concurrency()
}

// concurrency
#[allow(unused)]
fn my_concurrency() {
    my_cow::cow_exp_string();
    my_cow::cow_container_string();
    my_cow::beef_cow();
    my_box::box_stack_to_heap();
    my_box::box_heap_to_stack();
    my_box::box_auto_data_size();
    my_cell::cell_exp();
    my_cell::ref_cell_exp();
    my_cell::once_cell_exp();
    my_rc::rc_exp();
    my_rc::rc_refcell_example();
    my_rc::arc_exp();
    my_rc::arc_exp_mutex();
    my_mutex::mutex_lock_exp();
    my_mutex::mutex_try_lock_exp();
    my_mutex::mutex_poisoning_exp();
    my_mutex::mutex_fast_release_scop_exp();
    my_mutex::mutex_fast_release_drop_exp();
    my_mutex::rwmutex_exp();
    my_mutex::rwmutex_exp_write_wait();
    my_mutex::rwmutex_exp_read_wait();
    my_mutex::rwmutex_exp_dead_lock();
    my_once::once_exp();
    my_once::once_exp_get_config();
    my_once::once_cell_exp();
    my_barrier::barrier_exp();
    my_barrier::barrier_loop();
    my_mpsc::mpsc_exp();
    my_mpsc::mpsc_producer();
    my_mpsc::mpsc_sync();
    my_mpsc::mpsc_receiver_error();
    my_atomic::atomic_exp();
    my_atomic::atomic_ordering_relaxed();
    my_atomic::atomic_ordering_seqcst();
    my_atomic::atomic_ordering_acquire();
    my_atomic::atomic_ordering_release();
    my_atomic::atomic_ordering_acqrel();
    my_set::vec_exp();
    my_set::hash_map_exp();
    my_set::dash_map_exp();
    my_set::cuckoofilter_exp();
    my_set::evmap_exp();
    my_set::arc_swap_exp();
}

// mping
#[allow(unused)]
#[cfg(target_os = "linux")]
fn my_mping() -> Result<(), anyhow::Error> {
    return mping::exec::run();
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
