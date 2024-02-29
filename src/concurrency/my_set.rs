pub fn vec_exp() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    // 使用 Arc 和 Mutex 包装 Vec
    let shared_vec = Arc::new(Mutex::new(Vec::new()));

    // 创建一些线程，共同向 Vec 中添加元素
    let mut handles = vec![];
    for i in 0..5 {
        let shared_vec = Arc::clone(&shared_vec);
        let handle = thread::spawn(move || {
            // 获取锁
            let mut vec = shared_vec.lock().unwrap();

            // 修改 Vec
            vec.push(i);
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 获取 Vec，并输出结果
    let final_vec = shared_vec.lock().unwrap();
    println!("Final Vec: {:?}", *final_vec);
}

pub fn hash_map_exp() {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::thread;

    // 使用 Arc 和 Mutex 包装 HashMap
    let shared_map = Arc::new(Mutex::new(HashMap::new()));

    // 创建一些线程，共同向 HashMap 中添加键值对
    let mut handles = vec![];
    for i in 0..5 {
        let shared_map = Arc::clone(&shared_map);
        let handle = thread::spawn(move || {
            // 获取锁
            let mut map = shared_map.lock().unwrap();

            // 修改 HashMap
            map.insert(i, i * i);
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 获取 HashMap，并输出结果
    let final_map = shared_map.lock().unwrap();
    println!("Final HashMap: {:?}", *final_map);
}

pub fn dash_map_exp() {
    use dashmap::DashMap;
    use std::sync::Arc;

    let map = Arc::new(DashMap::new());
    let mut handles = vec![];

    for i in 0..10 {
        let map = Arc::clone(&map);
        handles.push(std::thread::spawn(move || {
            map.insert(i, i);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("DashMap: {:?}", map);
}

pub fn cuckoofilter_exp() {
    use cuckoofilter::CuckooFilter;

    let value: &str = "hello world";

    // 创建 cuckoo filter，默认最大容量为 1000000 项
    let mut cf = CuckooFilter::new();

    // 向过滤器添加数据
    cf.add(value).unwrap();

    // 查询数据是否在过滤器中
    let success = cf.contains(value);
    println!("contains: {}", success);

    // 测试并添加到过滤器（如果数据不存在，则添加）
    let success = cf.test_and_add(value).unwrap();
    println!("test_and_add: {}", success);

    // 从过滤器中删除数据
    let success = cf.delete(value);
    println!("delete: {}", success);
}

pub fn evmap_exp() {
    use std::sync::{Arc, Mutex};

    let (book_reviews_r, book_reviews_w) = evmap::new();

    // 启动一些写入程序。
    // 由于 evmap 不支持并发写入，我们需要用 mutex 来保护写句柄。
    let w = Arc::new(Mutex::new(book_reviews_w));
    let writers: Vec<_> = (0..4)
        .map(|i| {
            let w = w.clone();
            std::thread::spawn(move || {
                let mut w = w.lock().unwrap();
                w.insert(i, true);
                w.refresh();
            })
        })
        .collect();

    // 最终我们会看到 eventually
    while book_reviews_r.len() < 4 {
        std::thread::yield_now();
    }

    // 所有线程最终都应完成写入
    for w in writers.into_iter() {
        assert!(w.join().is_ok());
    }
}

pub fn arc_swap_exp() {
    use arc_swap::ArcSwap;
    use std::sync::Arc;

    // 创建 ArcSwap 包含整数
    let data = ArcSwap::new(Arc::new(1));

    // 打印当前值
    println!("Initial Value: {}", data.load());

    // 原子地交换值
    data.store(Arc::new(2));

    // 打印新值
    println!("New Value: {}", data.load());
}
