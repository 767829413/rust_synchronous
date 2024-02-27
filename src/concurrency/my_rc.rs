use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use std::sync::{Arc, Mutex};
use std::thread;

pub fn rc_exp() {
    let data = Rc::new(42);

    let _reference1 = Rc::clone(&data);
    let _reference2 = Rc::clone(&data);
    // data 的引用计数现在为 3
    // 当 reference1 和 reference2 被丢弃时，引用计数减少
}

pub fn rc_refcell_example() {
    let shared_map: Rc<RefCell<_>> = Rc::new(RefCell::new(HashMap::new()));
    {
        let mut map: RefMut<_> = shared_map.borrow_mut();
        map.insert("africa", 92388);
        map.insert("kyoto", 11837);
        map.insert("piccadilly", 11826);
        map.insert("marbles", 38);
    }

    let total: i32 = shared_map.borrow().values().sum();
    println!("{total}");
}

pub fn arc_exp() {
    // 创建一个可共享的整数
    let data = Arc::new(46);

    // 创建两个线程，共享对data的引用
    let thread1 = {
        let data = Arc::clone(&data);
        thread::spawn(move || {
            // 在线程中使用data
            println!("Thread 1: {}", data);
        })
    };

    let thread2 = {
        let data = Arc::clone(&data);
        thread::spawn(move || {
            // 在另一个线程中使用data
            println!("Thread 2: {}", data);
        })
    };

    // 等待两个线程完成
    thread1.join().unwrap();
    thread2.join().unwrap();
}

pub fn arc_exp_mutex() {
    // 创建一个可共享的可变整数
    let counter = Arc::new(Mutex::new(0));

    // 创建多个线程来增加计数器的值
    let mut handles = vec![];

    for _ in 0..5 {
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
    println!("Final count: {}", *counter.lock().unwrap());
}
