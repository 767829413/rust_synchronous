# Rust:Mutex内部实现

## 介绍

`Mutex` 是最常用的一种同步原语，它提供了互斥锁的功能,多线程可以互斥访问共享数据以及通过锁保护临界区。

`Rust` 标准库提供了 `Mutex` 的实现，接下来看看它是怎么实现的。

## Mutex 的定义

Mutex包含三个字段:

* 内部实现的锁(`sys::Mutex`)，根据不同的操作系统，可能选择不同的实现
* `poison`，用来标记锁是否被破坏，是否中毒了
* `data`，用来存储被保护的数据

```rust
pub struct Mutex<T: ?Sized> {
    inner: sys::Mutex,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(t: T) -> Mutex<T> {
        Mutex { inner: sys::Mutex::new(), poison: poison::Flag::new(), data: UnsafeCell::new(t) }
    }
}
```

另外一个关联的数据结构是`MutexGuard`, 它是`Mutex`的一个智能指针，用来管理锁的生命周期。

它实现了`Deref`和`Drop`，所以可以通过*来访问被保护的数据，当`MutexGuard`离开作用域时，会自动释放锁。

```rust
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
    poison: poison::Guard,
}
```

当请求锁时，调用内部的`sys::Mutex`上锁，并且返回一个`MutexGuard`，其实严格的说，返回一个`LockResult`，它是一个`Result`，当锁中毒时，返回`Err`，否则返回`Ok`，`Ok`中包含了`MutexGuard`。

```rust
pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
    unsafe {
        self.inner.lock();
        MutexGuard::new(self)
    }
}

pub type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;
```

`poison`如果不太了解，可以看看[Poisoning](https://nomicon.purewhite.io/poisoning.html#poisoning)，不进一步介绍了。

`MutexGuard`功能已经介绍了, 接下来看看它的实现:

```rust
impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}
```

`MutexGuard`的解引用返回被保护的数据，返回的是一个引用和可变引用。

当`MutexGuard`离开作用域时，会自动释放锁，它的实现如下,可以看到它调用了`self.lock.inner.unlock()`:

```rust
impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.unlock();
        }
    }
}
```

`try_lock`调用`inner`的`try_lock`：

```rust
pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
    unsafe {
        if self.inner.try_lock() {
            Ok(MutexGuard::new(self)?)
        } else {
            Err(TryLockError::WouldBlock)
        }
    }
}
```

目前 `rust` 还提供了一个不稳定的方法`unlock`，主动的立即释放锁,其实就是调用了`drop(guard)`:

```rust
pub fn unlock(guard: MutexGuard<'_, T>) {
        drop(guard);
}
```

那么看起来，锁的主要逻辑是`inner`实现的，下面可以看看`inner`的实现。

`inner`的类型是`sys::Mutex`,它位于[library/std/src/sys/mod.rs](https://github.com/rust-lang/rust/blob/8acf40bd546258a70953187c950ea56d1a8f5bf2/library/std/src/sys/mod.rs),根据操作系统的不同，有不同的实现，我们主要看 `linux(unix)`的实现。

`unix` 的实现位于[/library/std/src/sys/unix/locks](https://github.com/rust-lang/rust/blob/8acf40bd546258a70953187c950ea56d1a8f5bf2/library/std/src/sys/unix/locks/futex_mutex.rs), 根据具体的操作系统，也有两种实现方案，我们主要看 `linux` 的实现。

```rust
if #[cfg(any(
        target_os = "linux",
        target_os = "android",
        all(target_os = "emscripten", target_feature = "atomics"),
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "dragonfly",
    ))] {
        mod futex_mutex;
        mod futex_rwlock;
        mod futex_condvar;
        pub(crate) use futex_mutex::Mutex;
        pub(crate) use futex_rwlock::RwLock;
        pub(crate) use futex_condvar::Condvar;
    } ...
```

它有两种实现`futex_mutex`和`pthread_mutex.rs`, `Linux` 操作系统下，使用的是`futex_mutex`。

`futex_mutex`会使用 `Linux` 操作系统的`futex_wait`和`futex_wake`系统调用。 

它只包含一个`futex`字段，它是一个`AtomicU32`，用来表示锁的状态，它有三种状态:

* 0: 未加锁状态·
* 1: 加锁，且没有其它线程等待
* 2: 加锁，且有其它线程等待(竞争)

```rust
use crate::sync::atomic::{
    AtomicU32,
    Ordering::{Acquire, Relaxed, Release},
};
use crate::sys::futex::{futex_wait, futex_wake};

pub struct Mutex {
    futex: AtomicU32,
}
```

`new`创建一个未加锁的 `Muext`, 而`try_lock`会尝试将`futex`从 0 改为 1，如果成功，表示加锁成功，否则失败。 

注意这里使用了 `Acquire`和`Relaxed Ordering`，交换成功新值内存可见(`Acquire`)，失败无所谓了(`Relaxed`):

```rust
impl Mutex {
    #[inline]
    pub const fn new() -> Self {
        Self { futex: AtomicU32::new(0) }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        self.futex.compare_exchange(0, 1, Acquire, Relaxed).is_ok()
    }

    ...
}

接下来是重头戏`lock`:

```rust
impl Mutex {
    ...
    #[inline]
    pub fn lock(&self) {
        if self.futex.compare_exchange(0, 1, Acquire, Relaxed).is_err() {
            self.lock_contended(); // 如果第一次抢不到锁，进入锁竞争场景的处理
        }
    }

    #[cold]
    fn lock_contended(&self) {
        // 获取锁的状态
        let mut state = self.spin();

        // 如果未加速，去抢!
        if state == 0 {
            match self.futex.compare_exchange(0, 1, Acquire, Relaxed) {
                Ok(_) => return, // 抢成功，返回
                Err(s) => state = s, // 失败，把当前的状态赋值给state
            }
        }

        loop {
            // 运气来了，在多个人抢一个未加锁的锁时，你，命中天子抢到了锁，并且把锁状态从0改为2
            if state != 2 && self.futex.swap(2, Acquire) == 0 {
                // 成功把锁的状态从0改成了2，获取了锁，返回
                return;
            }

            // 等待锁状态变化，如果锁状态不是2，就不等了
            futex_wait(&self.futex, 2, None);

            // 别的线程释放了锁，spin检查锁的状态，再集中精力继续抢
            state = self.spin();
        }
    }

    fn spin(&self) -> u32 {
        let mut spin = 100;
        loop {
            // 获取当前锁的状态， 可能是0,1,2三种状态
            let state = self.futex.load(Relaxed);

            // 如果是未加锁，或者是有其它线程等待，则返回
            // 如果spin次数用完了，也返回
            if state != 1 || spin == 0 {
                return state;
            }

            crate::hint::spin_loop();
            spin -= 1;
        }
    }
    ...

}
```

总结一下，就是抢锁的线程通过 `spin`,避免上下文切换，能够提高性能。如果 `spin` 次数用完了，就进入等待状态，等待其它线程释放锁，然后再抢。

剩下一个方法就是解锁了。解锁比较简单，就是把锁的状态从 1 或者 2 改为 0，如果原来是 2，表示还有其它线程等待，就唤醒一个。

```rust
impl Mutex {
    ...
    pub unsafe fn unlock(&self) {
        if self.futex.swap(0, Release) == 2 {
            // 如果还有等待的线程，就唤醒一个
            self.wake();
        }
    }

    #[cold]
    fn wake(&self) {
        futex_wake(&self.futex);
    }
    ...
}
```

可以看到，`Rust` 的 `Mutex` 的实现还是比较简单的，它的核心是利用操作系统的`futex`相关方法，加一个`AtomicU32`标志来实现。