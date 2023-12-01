mod base;
use base::my_thread;
use std::thread;

fn main() {
    // thread base
    my_thread::start_one_thread();
    my_thread::start_one_thread_result();
    my_thread::start_two_threads();
    my_thread::start_n_threads(12);
    my_thread::start_one_thread_by_builder();
    let current_thread = thread::current();
    println!(
        "current main thread: {:?},{:?}",
        current_thread.id(),
        current_thread.name()
    );
    my_thread::current_thread();
    my_thread::thread_park();
    my_thread::cpu_info();
    my_thread::thread_info();
    my_thread::start_thread_with_sleep();
    my_thread::start_thread_with_yield_now();
    my_thread::thread_park_sleep();
    my_thread::start_scoped_threads();
    my_thread::start_threads_with_threadlocal();
    my_thread::start_one_thread_with_move();
}
