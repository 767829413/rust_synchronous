# async/await

## 介绍

异步编程是一种并发编程模型，通过在任务执行期间不阻塞线程的方式，提高系统的并发能力和响应性。相比于传统的同步编程，异步编程可以更好地处理 I/O 密集型任务和并发请求，提高系统的吞吐量和性能。

`异步编程具有以下优势：`

- 提高系统的并发能力和响应速度

- 减少线程等待时间，提高资源利用率

- 可以处理大量的并发请求或任务

- 支持高效的事件驱动编程风格

`异步编程广泛应用于以下场景：`

- 网络编程：处理大量的并发网络请求

- I/O 密集型任务：如文件操作、数据库访问等

- 用户界面和图形渲染：保持用户界面的流畅响应

- 并行计算：加速复杂计算任务的执行

`通过一些介绍展示使用 rust 对 async/await 的相关crate`

```rust
use futures::{channel::mpsc, executor, executor::ThreadPool, stream::StreamExt};
use std::{future::Future, thread};
use tokio::{
    join,
    // runtime::{Builder, Runtime},
    time::{sleep, Duration},
};
```

## rust 中的异步编程模型

### 指导参考

1. [Rust官方的异步编程书](https://rust-lang.github.io/async-book/)

2. [中文版Rust异步编程指南](https://github.com/rustlang-cn/async-book)

### 模型介绍

由于并发编程在现代社会非常重要，因此每个主流语言都对自己的并发模型进行过权衡取舍和精心设计，Rust 语言也不例外。

下面介绍不同并发模型的特性: 都是不存在绝对的优缺点一说,毕竟抛开场景说优缺点那是耍流氓

1. `OS 线程`
    - 也许特点:
        - 简单，也无需改变任何编程模型(业务/代码逻辑)
        - 非常适合作为语言的原生并发模型，Rust 就选择了原生支持线程级的并发编程
    - 可能缺点:
        - 线程间的同步将变得更加困难
        - 线程间的上下文切换损耗较大
        - 使用线程池在一定程度上可以提升性能，但是对于 IO 密集的场景来说，线程池还是不够看

2. `事件驱动(Event driven)`
    - 也许特点:
        - 事件驱动常常跟回调( Callback )一起使用
        - 性能相当好
    - 可能缺点:
        - 存在回调地狱的风险, 参考 JS 曾经就存在回调地狱
        - 非线性的控制流和结果处理导致了数据流向和错误传播变得难以掌控
        - 代码可维护性和可读性的大幅降低

3. `协程(Coroutines)`
    - 也许特点:
        - 比如Go语言的协程设计,很棒的并发模型
        - 协程跟线程类似，无需改变编程模型
        - 和 async 类似，可以支持大量的任务并发运行
    - 可能缺点:
        - 协程抽象层次过高，导致用户无法接触到底层的细节
        - 系统编程语言和自定义异步运行存在难点

4. `actor 模型`
    - 也许特点:
        - erlang 的杀手锏之一
        - 将所有并发计算分割成一个一个单元，这些单元被称为 actor
        - 单元之间通过消息传递的方式进行通信和数据传递
        - 贴近分布式系统的设计理念
        - actor 模型和现实比较贴近，实现起来相对容易
    - 可能缺点:
        - 一旦遇到流控制、失败重试等场景时，可能会变得不太好用

5. `async/await`
    - 也许特点:
        - 性能高，支持底层编程
        - 和线程和协程那样无需过多的改变
    - 可能缺点:
        - 内部实现机制过于复杂
        - 理解和使用起来也没有线程和协程简单

### rust的异步编程模型和异步运行时

`rust 终选择了同时提供了多线程编程和 async 编程:`

1. 通过标准库实现，无需那么高的并发时，例如需要并行计算时，可以选择它，优点是线程内的代码执行效率更高、实现更直观更简单

2. 通过语言特性 + 标准库 + 三方库的方式实现，在你需要高并发、异步  I/O  时，选择它就对了

异步运行时是 rust 中支持异步编程的运行时环境，负责管理异步任务的执行和调度。

异步运行时主要提供:

1. 任务队列、线程池和事件循环等基础设施

2. 支持异步任务的并发执行和事件驱动的编程模型

**Rust 没有内置异步调用所必须的运行时**

`主要的 Rust 异步运行时包括:`

1. Tokio 
    - Rust 异步运行时的首选,拥有强大的性能和生态系统
    - Tokio 提供异步 TCP/UDP 套接字、线程池、定时器等功能

2. async-std 
    - 较新但功能完善的运行时,提供与 Tokio 类似的异步抽象
    - 代码较简洁,易于上手

3. smol 
    - 一个轻量级的运行时
    - 侧重 simplicity(简单性)、ergonomics(易用性)和小巧

4. futures/futures-lite/futures-async-await
    - futures
        - 一个完整的异步运行时库
        - 提供了丰富的异步编程工具，包括异步任务、异步通道、定时器等等
        - 它是 Rust 生态系统中广泛使用的异步库之一
    - futures-lite
        - 一个轻量级的异步运行时库
        - 专注于提供最基本的异步编程功能,目标是提供简单、易用的异步接口
        - 尽量减少对其他依赖库的依赖，使得它可以在更多的环境中使用
    - futures-async-await
        - 一个过渡性的库，用于在 rust 1.39 版本之前使用 async/await 语法进行异步编程
        - 在Rust 1.39 版本开始，不再需要使用该库，而是直接使用原生的 async/await 语法

5. bytedance/monoio
    - 字节跳动开源的 Rust 异步运行时库
    - 基于 async/await 编程模型
    - 提供高性能和可维护性的异步编程能力

### rust 异步编程模型的一些关键组件和概念

#### 异步函数和异步块：使用 async 关键字定义的异步函数和异步代码块

```rust
// `foo()`返回一个`Future<Output = u8>`,
// 当调用`foo().await`时，该`Future`将被运行，当调用结束后我们将获取到一个`u8`值
async fn foo() -> u8 { 5 }

fn bar() -> impl Future<Output = u8> {
    // 下面的`async`语句块返回`Future<Output = u8>`
    async {
        let x: u8 = foo().await;
        x + 5
    }
}
```

**async 语句块和 async fn 最大的区别就是前者无法显式的声明返回值，在大多数时候这都不是问题，但是当配合 ? 一起使用时，问题就有所不同:**

```rust
async fn foo() -> Result<u8, String> {
    Ok(1)
}
async fn bar() -> Result<u8, String> {
    Ok(1)
}
pub fn main() {
    let fut = async {
        foo().await?;
        bar().await?;
        Ok(())
    };
}
```

以上代码编译后会报错:

```bash
error[E0282]: type annotations needed
  --> src/main.rs:16:9
   |
16 |         Ok(())
   |         ^^ cannot infer type of the type parameter `E` declared on the enum `Result`
   |
help: consider specifying the generic arguments
   |
16 |         Ok::<(), E>(())
   |           +++++++++

For more information about this error, try `rustc --explain E0282`.
error: could not compile `synchronous` (bin "synchronous") due to previous error
```

原因在于编译器无法推断出  Result<T, E>中的 E 的类型，可以使用  ::< ... >  的方式来增加类型注释

```rust
let fut = async {
    foo().await?;
    bar().await?;
    // Ok(())
    Ok::<(), String>(()) // 在这一行进行显式的类型注释
};
```

#### await 关键字：在异步函数内部使用 await 关键字等待异步操作完成

```text
async/.await是 Rust 语法的一部分，它在遇到阻塞操作时( 例如 IO )会让出当前线程的所有权而不是阻塞当前线程，这样就允许当前线程继续去执行其它代码，最终实现并发。

async 是懒惰的，直到被执行器 poll 或者 .await 后才会开始运行，其中后者是最常用的运行 Future 的方法。 
当 .await 被调用时，它会尝试运行 Future 直到完成，但是若该 Future 进入阻塞，那就会让出当前线程的控制权。当 Future 后面准备再一次被运行时(例如从  socket  中读取到了数据)，执行器会得到通知，并再次运行该  Future ，如此循环，直到完成。
```

#### Future Trait：表示异步任务的 Future Trait，提供异步任务的执行和状态管理

```rust
pub trait Future {
    type Output;

    // Required method
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

### async/await 语法和用法

async 和 await 是 Rust 中用于异步编程的关键字。async 用于定义异步函数，表示函数体中包含异步代码。await 用于等待异步操作完成，并返回异步操作的结果。

1. 异步函数使用 async 关键字定义，并返回实现了 Future Trait 的类型。异步函数可以在其他异步函数中使用 await 关键字等待异步操作完成。调用异步函数时，会返回一个实现了 Future Trait 的对象，可以通过调用 .await 方法等待结果。

2. 异步块是一种在异步函数内部创建的临时异步上下文，可以使用 async 关键字创建。异步闭包是一种将异步代码封装在闭包中的方式，可以使用 async 关键字创建。异步块和异步闭包允许在同步上下文中使用 await 关键字等待异步操作。

```text
异步函数的返回类型通常是实现了 Future Trait 的类型。
Future Trait 表示一个异步任务，提供异步任务的执行和状态管理。
Rust 标准库和第三方库中提供了许多实现了 Future Trait 的类型，用于表示各种异步操作
```

`下面这个例子是一个传统的并发下载网页的例子：`

```rust
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
```

如果是在一个小项目中简单的去下载文件，这么写没有任何问题，但是一旦下载文件的并发请求多起来，那一个下载任务占用一个线程的模式就太重了，会很容易成为程序的瓶颈。

`好在，我们可以使用async的方式来解决：`

```rust
async fn download_async(str: &str) {
    println!("download_async start {} downloading...", str);
}

async fn get_two_sites_async() {
    // 创建两个不同的`future`，你可以把`future`理解为未来某个时刻会被执行的计划任务
    // 当两个`future`被同时执行后，它们将并发的去下载目标页面
    let future_one = download_async("https://www.foo.com");
    let future_two = download_async("https://www.bar.com");

    // 同时运行两个`future`，直至完成
    join!(future_one, future_two);
}

pub fn start_get_two_sites() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        get_two_sites_async().await;
    });

    println!("start_get_two_sites All done");
}
```

### Tokio

Tokio 是 Rust 异步编程最重要的运行时库,提供了异步 IO、异步任务调度、同步原语等功能。

`Tokio 的主要组件包括:`

- tokio - 核心运行时,提供任务调度,IO 资源等。
- tokio::net - 异步 TCP、UDP 的实现。
- tokio::sync - 互斥量、信号量等并发原语。
- tokio::time - 时间相关工具。
- tokio::fs - 异步文件 IO。

**Tokio 库包含了很多的功能，包括异步网络编程、并发原语等**

可以如下定义 main 函数，它自动支持运行时的启动：

main 函数前必须加 async 关键字，并且加 #[tokio::main] 属性，那么这个 main 就会在异步运行时运行

```rust
#[tokio::main]
async fn main() {
  // 在运行时中异步执行任务
  tokio::spawn(async {
    // do work
  });

  // 等待任务完成
  other_task.await;
}
```

还可以使用显示创建运行时的方法:

**PS: 不要运行在上面例子里: Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) attempted to block the current thread while the thread is being used to drive asynchronous tasks.**

```rust
fn main() {
    tokio_async();
}

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
```

`总结一下这个例子展示的要点:`

- 在 Tokio 运行时中用block_on执行异步任务
- 用spawn在运行时中异步执行任务
- 用spawn_blocking在线程池中执行阻塞任务
- 可以awaitJoinHandle 来等待异步任务结束

**Tokio 运行时提供了执行和调度异步任务所需的全部功能。通过正确地组合block_on、spawn和spawn_blocking,可以发挥 Tokio 的强大能力,实现各种异步场景。**

### futures

futures 库 futures 是 Rust 异步编程的基础抽象库,为编写异步代码提供了核心的 trait 和类型。

`主要提供了以下功能:`

- Future trait 
  - 表示一个异步计算的抽象,可以  .await  获取其结果。
- Stream trait 
  - 表示一个异步的数据流,可以通过  .await  迭代获取其元素。
- Sink trait 
  - 代表一个可以异步接收数据的目标。
- Executor 
  - 执行 futures 的运行时环境。
- Utilities 
  - 一些组合、创建 futures 的函数

下面这个例子展示了如何使用 futures 和线程池进行异步编程:

```rust
pub fn futures_async() {
    // 创建一个线程池pool
    let pool = ThreadPool::new().expect("futures_async Failed to build pool");
    // 创建一个无边界的通道tx和rx用来在任务间传递数据
    let (tx, rx) = mpsc::unbounded::<i32>();

    // 定义一个异步任务fut_values,里面首先用spawn_ok在线程池中异步执行一个任务
    // 这个任务会通过通道发送 0-99 的数字
    let fut_values = async {
        let fut_tx_result = async move {
            (0..100).for_each(|v| {
                tx.unbounded_send(v).expect("futures_async Failed to send");
            })
        };
        pool.spawn_ok(fut_tx_result);

        // 然后通过rx用map创建一个 Stream,它会将收到的数字乘 2
        let fut_values = rx.map(|v| v * 2).collect();

        fut_values.await
    };
    // 用collect收集 Stream 的结果到一个 Vec
    // block_on在主线程中执行这个异步任务并获取结果
    let values: Vec<i32> = executor::block_on(fut_values);

    println!("futures_async Values={:?}", values);
}
```

`上述代码展示了:`

- futures 和通道的组合使用 
  - 通过线程池并发地处理数据流。

- block_on运行 future 而不需要显式运行时也很方便

**futures 通过异步处理数据流,可以实现非阻塞并发程序,这在诸如网络服务端编程中很有用。与线程相比,futures 的抽象通常更轻量和高效。**

### futures_lite

这个库是 futures 的一个子集,它的编译速度快了一个数量级,修复了 futures API 中的一些小问题,补充了一些明显的空白,并移除了绝大部分不安全的代码。

**简而言之,这个库的目标是比 futures 更可易用,同时仍然与其完全兼容。**

从创建一个简单的 Future 开始。在 Rust 中，Future 是一种表示异步计算的 trait。以下是一个示例:

使用 futures-lite 中的 future::block_on 函数来运行异步函数 hello_async

```rust
use futures_lite::future;

async fn hello_async() {
    println!("Hello, async world!");
}

pub fn futures_lite_example() {
    future::block_on(hello_async());
}
```

### async_std

async-std 是一个为 Rust 提供异步标准库的库。它扩展了标准库，使得在异步上下文中进行文件 I/O、网络操作和任务管理等操作更加便捷。

它提供了你所习惯的所有接口,但以异步的形式,并且准备好用于 Rust 的async/await语法。

`主要特性如下:`

- 现代:  
  - 从零开始针对std::future和async/await构建,编译速度极快。

- 快速:  
  - 可靠的分配器和线程池设计提供了超高吞吐量和可预测的低延迟。

- 直观:  
  - 与标准库完全对等意味着只需要学习一次 API。

- 清晰:  
  - 详细的文档和可访问的指南意味着使用异步 Rust 没有沉重包袱。

下面是使用的一个简单例子:

```rust
use async_std::task;

// 定义一个异步函数 hi_async,其中只是简单打印一句话
async fn hi_async() {
    println!("Hi, async world!");
}

pub fn async_std_example() {
    // 使用 task::block_on 来执行这个异步函数
    // block_on 会阻塞当前线程,直到传入的 future 运行完成
    // 效果就是,尽管 hi_async 函数是异步的,但我们可以用同步的方式调用它,不需要手动处理 future
    task::block_on(hi_async());
}
```

**async/await 语法隐藏了 future 的细节,给异步编程带来了极大的便利。借助 async_std,我们可以非常轻松地使用 async/await 来编写异步 Rust 代码。**

### smol

smol 是一个超轻量级的异步运行时（async runtime）库，专为简化异步 Rust 代码的编写而设计。它提供了一个简洁而高效的方式来管理异步任务。

`主要特性如下:`

- 轻量级：
  - smol 的设计目标之一是轻量级，以便快速启动和低资源开销。

- 简洁 API： 
  - 提供简洁的 API，使得异步任务的创建、组合和运行变得直观和简单。

- 零配置： 
  - 无需复杂的配置，可以直接在现有的 Rust 项目中使用。

- 异步 I/O 操作： 
  - 支持异步文件 I/O、网络操作等，使得异步编程更加灵活。

下面这个例子演示了使用 smol 异步运行时执行异步代码块的例子：

```rust
use smol;

pub fn smol_async_example() {
    smol::block_on(async { println!("Hello from smol") });
}
```

### try_join、join、select 和 zip

在 Rust 中,有两个常见的宏可以用于同时等待多个 future:select 和 join。

select! 宏可以同时等待多个 future,并只处理最先完成的那个 future:

```rust
pub async fn futures_select_example() {
    let mut b = future::ready(6);
    let mut a = future::ready(4);

    let _ = select! {
        a_res = a => println!("a_res = {:?}", a_res),
        b_res = b => println!("b_res = {:?}", b_res),
    };
}
```

```text
join! 宏可以同时等待多个 future,并处理所有 future 的结果: join! 返回一个元组,包含所有 future 的结果。

这两个宏都需要 futures crate,使代码更加简洁。不使用宏的话,需要手动创建一个 Poll来组合多个 future。

所以 select 和 join 在处理多个 future 时非常方便。select 用于只处理最先完成的,join 可以同时处理所有 future。
```

```rust
use futures::future::{join, FutureExt};

let future1 = async { /* future 1 */ };
let future2 = async { /* future 2 */ };

let (res1, res2) = join!(future1, future2);
```

```text
try_join!宏也可以用于同时等待多个 future,它与 join!类似,但是有一点不同:

try_join!在任何一个 future 返回错误时,就会提前返回错误,而不会等待其他 future。

下面的例子因为 future2 返回了错误,所以 try_join!也会返回这个错误,不会等待 future1 完成。

这不同于 join!,join!会等待所有 future 完成。

所以 try_join!的用途是同时启动多个 future,但是遇到任何一个错误就立即返回,避免不必要的等待。这在需要并发但不能容忍任何失败的场景很有用。

而当需要等待所有 future 无论成功失败,获取所有结果的时候,再使用 join!。

所以 try_join!和 join!都可以组合多个 future,但错误处理策略不同。选择哪个要根据实际需要决定。
```

```rust
pub async fn futures_try_join_example() {
    let a = async { Ok::<i32, i32>(1) };
    let b = async { Err::<u64, i32>(2) };

    println!("futures_try_join_example {:?}", try_join!(a, b));
}
```

zip函数会 join 两个 future,并等待他们完成。而try_zip函数会 join 两个函数，但是会等待两个 future 都完成或者其中一个 Err 则返回：

```rust
pub fn smol_zip() {
    smol::block_on(async {
        use smol::future::{try_zip, zip, FutureExt};

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
```
