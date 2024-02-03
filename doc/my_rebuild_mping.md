# 使用rust重写一个ping工具

## 目的介绍

使用 go 实现了一个探测多目标的高频探测程序 [mping](https://github.com/767829413/advanced-go/tree/main/ping/mping), 支持硬件时间戳和软件时间戳，提供更精准的时延。

使用 rust 重写主要目的:

1. 体验 rust 开发工具的使用难度
2. 体验 rust 生态圈尤其是底层 socket 的编程的成熟度
3. 体验 rust 程序的性能
4. 积累一些 rust 底层网络编程的经验

Go 版本的 mping 功能如下:

* 支持对多目标 IP 的探测
* 支持对网段的探测
* 支持对域名的探测
* 支持高频探测，支持限流
* 支持设置 TOS、payload 长度等选项
* 支持最大探测数量
* 支持每秒打印统计数据
* 支持时延
* 支持比特跳变检查
* 支持软硬件时间戳

## 程序概设

回顾一下 mping 的实现拆解, 使用了新的 icmp 库实现, 这个实现分成了四个模块：

1. 主程序入口
    * 处理各种参数，比如网段信息转换成 IP 列表等，限流等

2. 发送 goroutine
    * 负责发送 ICMP 探测包

3. 接收 goroutine
    * 负责接收 ICMP echo reply 包
    
4. 统计打印 goroutine
    * 负责统计打印每一秒的丢包率和时延，这里使用了 bucket 数据结构进行统计分析

代码架构经过梳理还是能看的，结合 Go 成熟的 raw socket 的库，实现了一个高效的探测与压测集一身的 ping 工具。

在使用 rust 实现的时候也是采用这种架构，以下是是coding之前的一些概设:

1. 程序入口：
    * 问题思考:
        * 解析命令行参数使用包的选择?
            * structopt 库
            * clap 库
    * 选择 structopt 库
    * 理由
        * 对于 mping 这个工具来说，不需要强大的命令行交互能力
        * 能把命令行的参数解析成对应类型的变量
        * tructopt 的主要特性包括
            * structopt 利用 Rust 的属性宏来简化命令行参数解析
            * 允许直接在 struct 定义中添加属性,自动生成解析命令行参数所需的 clap 参数定义
            * 可以跳过手动定义 clap::Arg 的过程
            * 支持子命令,可以在 struct 中嵌套定义子命令
            * 支持各种参数类型,如布尔类型、字符串、整数等
            * 支持参数默认值、帮助信息等
            * 简单的错误处理
            * 持完整的 clap 功能,可以直接获取 clap::App 对象进行自定义
2. 使用线程代替 goroutine
    * 问题思考:
        * 要不要使用 async/await 异步方式?
    * 不需要使用 async/await
    * 理由
        * 通常情况下 rust 的并发单元是线程，所以这里发送、接收、统计模块分别启动三个线程来处理
        * 接收线程使用了主线程，而没有生成新的线程
        * 理论上这个程序是 I/O 密集型类型的程序，但是由于我们只需要一两个 socket + 三个线程就可以完成
        * 没有成百上千的 I/O 并发单元争抢 CPU,所以使用普通的线程就可以
        * 不要管异步运行时库的选择了

3. 网络库的选择
    * 问题思考:
        * 使用 标准库、tokio、socket2、nix、libc ?
    * 使用 socket2 库
    * 理由
        * 实现的 mping 不是一个普通的 TCP/UDP Client/Server 点对点的网络程序
        * 也不是 web 服务器这样的七层网络应用服务器
        * 要能收发 ICMP 包，并且支持单个 socket 发送和接收成百上千个目标地址的网络程序
        * 能设置 TOS、TTL 等网络选项
        * socket2 不是标准库，但是提供了更多的设置选项，更底层的控制
        * 写标准的 TCP/UDP，可能选择 tokio 较好
        * nix、libc 更底层，只是缺乏相关的文档和范例,容易踩坑
        * socket2 对软硬件时间戳的设置和解析可能存在问题,但是也能克服
4. 包的解析
    * 问题思考:
        * 使用什么库来实现解析或者自己造轮子?
    * [pnet](https://crates.io/crates/pnet)
    * 理由:
        * pnet 支持各种网络包的处理
        * 提供其它一些网络处理的功能
        * 子包 pnet_packet 能够处理发送和接收时候 IPv4、ICMP 的包

那么接下来过一下代码的实现，然后稍微比较一下 Rust 和 Go 版本 mping 工具不同点.

rust 版本的完整代码在 github 上: <https://github.com/767829413/rust_synchronous/tree/main/src/mping>

## 主程序入口

1. 使用structopt定义命令行参数的结构体Opt:

```rust
// rust_synchronous/src/mping/exec.rs
struct Opt {
    #[clap(
        short = 'w',
        long = "timeout",
        default_value = "1",
        help = "timeout in seconds"
    )]
    timeout: u64,

    #[clap(short = 't', long = "ttl", default_value = "64", help = "time to live")]
    ttl: u32,

    #[clap(short = 'z', long = "tos", help = "type of service")]
    tos: Option<u32>,

    #[clap(
        short = 's',
        long = "size",
        default_value = "64",
        help = "payload size"
    )]
    size: usize,

    #[clap(
        short = 'r',
        long = "rate",
        default_value = "100",
        help = "rate in packets/second"
    )]
    rate: u64,

    #[clap(
        short = 'd',
        long = "delay",
        default_value = "3",
        help = "delay in seconds"
    )]
    delay: u64,

    #[clap(short = 'c', long = "count", help = "max packet count")]
    count: Option<i64>,

    #[clap(
        value_delimiter = ',',
        required = true,
        name = "ip address",
        help = "one ip address or more, e.g. 127.0.0.1,8.8.8.8/24,bing.com"
    )]
    free: Vec<std::path::PathBuf>,
}
```

2. 解析多目标地址

需要把`8.8.8.8/30`, `8.8.4.4`, `github.com` 这种的目标解析成要探测的目标地址列表:

```rust
fn parse_ips(input: &str) -> Vec<IpAddr> {
    let mut ips = Vec::new();

    for s in input.split(',') {
        match s.parse::<IpNetwork>() {
            Ok(network) => {
                for ip in network.iter() {
                    ips.push(ip);
                }
            }
            Err(_) => {
                if let Ok(ip) = s.parse::<IpAddr>() {
                    ips.push(ip);
                } else if let Ok(addrs) = (s, 0).to_socket_addrs() {
                    for addr in addrs {
                        if let IpAddr::V4(ipv4) = addr.ip() {
                            ips.push(IpAddr::V4(ipv4));
                            break;
                        }
                    }
                }
            }
        }
    }

    return ips;
}
```

3. 定义一个执行函数 run() 给 main.rs 调用

```rust
pub fn run() -> Result<(), anyhow::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    let opt = Opt::parse();

    if opt.free.is_empty() {
        println!("Please input ip address");
        return Ok(());
    }

    let _ = opt.count;

    let addrs = opt.free.last().unwrap().to_string_lossy();
    let _ip_addrs = parse_ips(&addrs);

    let _timeout = Duration::from_secs(opt.timeout);
    let _pid = process::id() as u16;

    let popt = mping::ping::PingOption {
        timeout,
        ttl: opt.ttl,
        tos: opt.tos,
        ident: pid,
        len: opt.size,
        rate: opt.rate,
        rate_for_all: false,
        delay: opt.delay,
        count: opt.count,
    };
    // 实现发送、接收、定时统计的功能
    mping::ping::ping(ip_addrs, popt, true, None)?;

    Ok(())
}
```

## 发送逻辑
