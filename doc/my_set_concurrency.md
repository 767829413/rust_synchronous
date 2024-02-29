# Rust并发下的集合类型

## 介绍

集合类型是我们编程中常用的数据类型, `Rust` 中提供了一些集合类型, 比如`Vec<T>、HashMap<K, V>、HashSet<T>、VecDeque<T>、LinkedList<T>、BTreeMap<K, V>、BTreeSet<T>`等, 它们的特点如下：

* `Vec`: 
  * 这是一种可变大小的数组,允许在头部或尾部高效地添加和删除元素. 
  * 它类似于 `C++`的 `vector` 或 `Java` 的 `ArrayList`. 

* `HashMap<K,V>`: 
  * 这是一个哈希映射,允许通过键快速查找值. 
  * 它类似于 `C++`的 `unordered_map` 或 `Java` 的 `HashMap`. 

* `HashSet`: 
  * 这是一个基于哈希的集,可以快速判断一个值是否在集合中. 
  * 它类似于 `C++`的 `unordered_set` 或 `Java` 的 `HashSet`. 

* `VecDeque`: 
  * 这是一个双端队列,允许在头部或尾部高效地添加和删除元素. 
  * 它类似于 `C++`的 `deque` 或 `Java` 的 `ArrayDeque`. 

* `LinkedList`: 
  * 这是一个链表数据结构,允许在头部或尾部快速添加和删除元素. 

* `BTreeMap<K,V>`: 
  * 这是一个有序的映射,可以通过键快速查找,同时保持元素的排序. 
  * 它使用 B 树作为底层数据结构. 

* `BTreeSet`: 
  * 这是一个有序的集合,元素会自动排序. 
  * 它使用 B 树作为底层数据结构. 

这些类型都不是线程安全的, 没有办法在线程中共享使用, 但是可以使用前面介绍的并发原语, 对这些类型进行包装, 使之成为线程安全的. 

## 线程安全的 Vec

要实现线程安全的 `Vec`, 可以使用 `Arc`（原子引用计数）和 `Mutex`（互斥锁）的组合. 

`Arc` 允许多个线程共享拥有相同数据的所有权, 而 `Mutex` 用于在访问数据时进行同步, 确保只有一个线程能够修改数据. 

以下是一个简单的例子, 演示如何创建线程安全的 `Vec`：

```rust
pub fn vec_exp() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    // 使用 Arc 和 Mutex 包装 Vec
    let shared_vec = Arc::new(Mutex::new(Vec::new()));

    // 创建一些线程, 共同向 Vec 中添加元素
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

    // 获取 Vec, 并输出结果
    let final_vec = shared_vec.lock().unwrap();
    println!("Final Vec: {:?}", *final_vec);
}
```

在上面例子中, `shared_vec` 是一个 `Mutex` 包装的 `Arc`, 使得多个线程能够共享对 `Vec` 的所有权. 

每个线程在修改 `Vec` 之前需要先获取锁, 确保同一时刻只有一个线程能够修改数据. 

## 线程安全的 HashMap

要实现线程安全的 `HashMap`, 可以使用 `Arc`（原子引用计数）和 `Mutex`（互斥锁）的组合, 或者使用 `RwLock`（读写锁）来提供更细粒度的并发控制. 

以下是使用 `Arc` 和 `Mutex` 的简单示例：

```rust
pub fn hash_map_exp() {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::thread;

    // 使用 Arc 和 Mutex 包装 HashMap
    let shared_map = Arc::new(Mutex::new(HashMap::new()));

    // 创建一些线程, 共同向 HashMap 中添加键值对
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

    // 获取 HashMap, 并输出结果
    let final_map = shared_map.lock().unwrap();
    println!("Final HashMap: {:?}", *final_map);
}
```

处理的套路都是一样的, 就是使用`Arc<Mutex<T>>`实现. 使用 `Arc<Mutex<T>>` 组合是一种常见的方式来实现线程安全的集合类型, 但不是唯一的选择. 

这种组合的基本思想是使用 `Arc`（原子引用计数）来实现多线程间的所有权共享, 而 `Mutex` 则提供了互斥锁, 确保在任何时刻只有一个线程能够修改数据. 

后面的几种集合类型都可以这么去实现. 

有些场景下你可能使用`Arc<RwLock<T>>`更合适, 允许多个线程同时读取数据, 但只有一个线程能够写入数据. 适用于读操作频繁、写操作较少的场景. 

## dashmap

`DashMap`是极快的 `Rust` 并发 `map`, 这是一个 `Rust` 中并发关联 `array/hashmap` 的实现. 

`DashMap`试图实现一个类似于`std::collections::HashMap`的简单易用的 `API`,并做了一些细微的改变来处理并发. 

`DashMap`的目标是变得非常简单易用,并可直接替代 `RwLock<HashMap<K, V>>`. 为实现这些目标,所有方法采用`&self`而不是`&mut self`. 

这允许将一个 `DashMap` 放入一个 `Arc`中,并在线程之间共享它,同时仍然能够修改它. 

`DashMap`非常注重性能,并旨在尽可能快, 下面是一个简单例子:

```rust
pub fn dash_map_exp() {
    use std::sync::Arc;
    use dashmap::DashMap;

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
```

基于`DashMap`, 它还实现了`DashSet`

## lockfree

`lockfree`库, 提供了丰富的线程安全的集合类, 但是很久没有维护了, 就不过多说了.

* `Per-Object Thread-Local Storage`
* `Channels (SPSC, MPSC, SPMC, MPMC)`
* `Map`
* `Set`
* `Stack`
* `Queue`

## cuckoofilter

`Cuckoo Filter` 是一种基于哈希的数据结构, 用于实现高效的近似集合成员检查. 

设计灵感来自于 `Cuckoo Hashing`, 但在性能和内存占用方面有更好的表现. `Cuckoo Filter` 主要用于解决布隆过滤器的一些问题, 如高内存占用和不支持删除操作. 

下面是一个使用 `cuckoofilter` 库的例子:

```rust
pub fn cuckoofilter_exp() {
    use cuckoofilter::CuckooFilter;

    let value: &str = "hello world";

    // 创建 cuckoo filter, 默认最大容量为 1000000 项
    let mut cf = CuckooFilter::new();

    // 向过滤器添加数据
    cf.add(value).unwrap();

    // 查询数据是否在过滤器中
    let success = cf.contains(value);
    println!("contains: {}", success);

    // 测试并添加到过滤器（如果数据不存在, 则添加）
    let success = cf.test_and_add(value).unwrap();
    println!("test_and_add: {}", success);

    // 从过滤器中删除数据
    let success = cf.delete(value);
    println!("delete: {}", success);
}
```

## evmap

`evmap（eventual map）`是一个 `Rust` 库, 提供了一个并发的、基于事件的映射（`Map`）实现. 

它允许多个线程并发地读取和写入映射, 同时支持观察者模式, 允许在映射的变化上注册事件监听器. 

以下是 evmap 的一些关键特性和概念：

* 并发读写： 
  * `evmap` 允许多个线程并发地读取和写入映射, 而不需要使用锁. 
  * 这是通过将映射分为多个片段来实现的, 每个片段都可以独立地读取和写入. 

* 事件触发： 
  * `evmap` 允许在映射的变化上注册事件监听器. 
  * 当映射发生变化时, 注册的监听器会被触发, 从而允许用户执行自定义的逻辑. 

* 键和值： 
  * 映射中的键和值可以是任意类型, 只要它们实现了 `Clone` 和 `Eq trait`. 
  * 这样允许用户使用自定义类型作为键和值. 

* 异步触发事件： 
  * `evmap` 支持异步的事件触发. 
  * 这使得在事件发生时执行一些异步任务成为可能. 
    
以下是一个简单的示例, 演示了如何使用 `evmap`：

```rust
pub fn evmap_exp() {
    use std::sync::{Arc, Mutex};

    let (book_reviews_r, book_reviews_w) = evmap::new();

    // 启动一些写入程序. 
    // 由于 evmap 不支持并发写入, 我们需要用 mutex 来保护写句柄. 
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
```

## arc-swap

`arc-swap` 是一个 `Rust` 库, 提供了基于 `Arc` 和 `Atomic` 的数据结构, 用于在多线程之间原子地交换数据. 

它的设计目的是提供一种高效的方式来实现线程间的共享数据更新, 避免锁的开销.  

可以把它看成`Atomic<Arc<T>>`或者`RwLock<Arc<T>>`. 

在许多情况下,可能需要一些数据结构,这些数据结构经常被读取而很少更新. 

一些例子可能是服务的配置,路由表,每几分钟更新一次的某些数据的快照等. 

在所有这些情况下,需要:

* 快速、频繁并从多个线程并发读取数据结构的当前值. 

* 在更长的时间内使用数据结构的相同版本
  * 查询应该由数据的一致版本回答,数据包应该由旧版本或新版本的路由表路由,而不是由组合路由. 

* 在不中断处理的情况下执行更新. 

第一个想法是使用 `RwLock<T>` 并在整个处理时间内保持读锁. 但是更新会暂停所有处理直到完成.  

更好的选择是使用 `RwLock<Arc<T>>`. 然后可以获取锁,克隆`Arc` 并解锁. 这会受到 `CPU` 级别的争用(锁和 `Arc` 的引用计数)的影响,从而相对较慢. 

根据实现的不同,稳定的 `reader` 流入可能会阻塞更新任意长的时间. 

可以使用 `ArcSwap` 替代,它解决了上述问题,在竞争和非竞争场景下,性能特征都优于 `RwLock`. 

下面是一个 `arc-swap` 的例子:

```rust
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
```

在这个例子中, `ArcSwap` 包含一个整数, 并通过原子的 `store` 操作交换了新的 `Arc`. 

这使得多个线程可以安全地共享和更新 `Arc`. 
