# 使用rust重写一个ping工具

## 目的介绍

使用 go 实现了一个探测多目标的高频探测程序 [mping](https://github.com/767829413/advanced-go/blob/main/doc/ch8-Implementing%20ping%20by%20icmp%20package/README.md), 支持硬件时间戳和软件时间戳, 提供更精准的时延. 

使用 rust 重写主要目的:

1. 体验 rust 开发工具的使用难度
2. 体验 rust 生态圈尤其是底层 socket 的编程的成熟度
3. 体验 rust 程序的性能
4. 积累一些 rust 底层网络编程的经验

Go 版本的 mping 功能如下:

* 支持对多目标 IP 的探测
* 支持对网段的探测
* 支持对域名的探测
* 支持高频探测, 支持限流
* 支持设置 TOS、payload 长度等选项
* 支持最大探测数量
* 支持每秒打印统计数据
* 支持时延
* 支持比特跳变检查
* 支持软硬件时间戳

## 程序概设

回顾一下 mping 的实现拆解, 使用了新的 icmp 库实现, 这个实现分成了四个模块：

1. 主程序入口
    * 处理各种参数, 比如网段信息转换成 IP 列表等, 限流等

2. 发送 goroutine
    * 负责发送 ICMP 探测包

3. 接收 goroutine
    * 负责接收 ICMP echo reply 包
    
4. 统计打印 goroutine
    * 负责统计打印每一秒的丢包率和时延, 这里使用了 bucket 数据结构进行统计分析

代码架构经过梳理还是能看的, 结合 Go 成熟的 raw socket 的库, 实现了一个高效的探测与压测集一身的 ping 工具. 

在使用 rust 实现的时候也是采用这种架构, 以下是是coding之前的一些概设:

1. 程序入口：
    * 问题思考:
        * 解析命令行参数使用包的选择?
            * structopt 库
            * clap 库
    * 选择 structopt 库
    * 理由
        * 对于 mping 这个工具来说, 不需要强大的命令行交互能力
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
        * 通常情况下 rust 的并发单元是线程, 所以这里发送、接收、统计模块分别启动三个线程来处理
        * 接收线程使用了主线程, 而没有生成新的线程
        * 理论上这个程序是 I/O 密集型类型的程序, 但是由于我们只需要一两个 socket + 三个线程就可以完成
        * 没有成百上千的 I/O 并发单元争抢 CPU,所以使用普通的线程就可以
        * 不要管异步运行时库的选择了

3. 网络库的选择
    * 问题思考:
        * 使用 标准库、tokio、socket2、nix、libc ?
    * 使用 socket2 库
    * 理由
        * 实现的 mping 不是一个普通的 TCP/UDP Client/Server 点对点的网络程序
        * 也不是 web 服务器这样的七层网络应用服务器
        * 要能收发 ICMP 包, 并且支持单个 socket 发送和接收成百上千个目标地址的网络程序
        * 能设置 TOS、TTL 等网络选项
        * socket2 不是标准库, 但是提供了更多的设置选项, 更底层的控制
        * 写标准的 TCP/UDP, 可能选择 tokio 较好
        * nix、libc 更底层, 只是缺乏相关的文档和范例,容易踩坑
        * socket2 对软硬件时间戳的设置和解析可能存在问题,但是也能克服
4. 包的解析
    * 问题思考:
        * 使用什么库来实现解析或者自己造轮子?
    * [pnet](https://crates.io/crates/pnet)
    * 理由:
        * pnet 支持各种网络包的处理
        * 提供其它一些网络处理的功能
        * 子包 pnet_packet 能够处理发送和接收时候 IPv4、ICMP 的包

那么接下来过一下代码的实现, 然后稍微比较一下 Rust 和 Go 版本 mping 工具不同点.

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

// 核心的逻辑入口函数 ping()
pub fn ping(
    addrs: Vec<IpAddr>,
    popt: PingOption,
    enable_print_stat: bool,
    tx: Option<Sender<TargetResult>>,
) -> anyhow::Result<()> {
    let pid = popt.ident as u16;

    let rand_payload = random_bytes(popt.len);
    let read_rand_payload = rand_payload.clone();

    // 构建存储 ping 结果的 Buckets
    let buckets = Arc::new(Mutex::new(Buckets::new_buckets()));
    // 发送
    let send_buckets = buckets.clone();
    // 接收
    let read_buckets = buckets.clone();
    // 状态打印
    let stat_buckets = buckets.clone();

    // 创建了一个新的套接字, 指定了 IPv4 地址族、原始类型（RAW）以及 ICMPv4 协议. 如果创建失败, 会通过 unwrap() 抛出错误
    let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).unwrap();
    // 设置套接字的 TTL（Time-To-Live）值为 popt.ttl, 即生存时间. 如果设置失败, 会通过 unwrap() 抛出错误
    socket.set_ttl(popt.ttl).unwrap();
    // 设置套接字的写超时时间为 popt.timeout. 如果设置失败, 会通过 unwrap() 抛出错误
    socket.set_write_timeout(Some(popt.timeout)).unwrap();
    // 如果在命令行选项中指定了 TOS（Type of Service）值, 则设置套接字的 TOS 值为指定的值. 如果设置失败, 会通过 unwrap() 抛出错误
    if let Some(tos_value) = popt.tos {
        socket.set_tos(tos_value).unwrap();
    }
    // 尝试克隆套接字, 如果克隆失败, 则打印错误信息并终止程序
    let socket2 = socket.try_clone().expect("Failed to clone socket");
    // 检查是否设置接收, 使用 is_some() 方法检查其是否有值, 如果有值则返回 true, 否则返回 false, 结果存储在 has_tx 变量中
    let has_tx = tx.is_some();

    // send
    let send_opt = popt.clone();
    thread::spawn(move || {
        send(
            socket,
            addrs,
            send_opt,
            send_buckets,
            rand_payload,
            pid,
            has_tx,
        )
    });

    // 打印
    let print_opt = popt.clone();
    thread::spawn(move || print_stat(stat_buckets, print_opt, enable_print_stat, tx.clone()));

    // read
    let read_opt = popt.clone();
    let _ = read(socket2, read_opt, read_buckets, pid, read_rand_payload);

    Ok(())
}

```

## 发送逻辑

主要就是不停的发包

```rust
fn send(
    socket: Socket,
    addrs: Vec<IpAddr>,
    popt: PingOption,
    send_buckets: Arc<Mutex<Buckets>>,
    rand_payload: Vec<u8>,
    pid: u16,
    has_tx: bool,
) -> anyhow::Result<()> {
    // 条件化的 Linux socket 设置
    cfg_if! {
        if #[cfg(target_os = "linux")] {
            // 在 Linux 环境下, 根据目标操作系统的不同设置了一些 socket 选项
            // 主要是关于时间戳的设置
            // 如果设置失败, 会将 support_tx_timestamping 置为 false
            let mut support_tx_timestamping = true;
            let raw_fd = socket.as_raw_fd();
            let enable = SOF_TIMESTAMPING_SOFTWARE
                | SOF_TIMESTAMPING_TX_SOFTWARE
                | SOF_TIMESTAMPING_SYS_HARDWARE
                | SOF_TIMESTAMPING_TX_HARDWARE
                | SOF_TIMESTAMPING_RAW_HARDWARE
                | SOF_TIMESTAMPING_OPT_CMSG
                | SOF_TIMESTAMPING_OPT_TSONLY;
            let ret = unsafe {
                setsockopt(
                    raw_fd,
                    SOL_SOCKET,
                    SO_TIMESTAMPING,
                    &enable as *const _ as *const c_void,
                    mem::size_of_val(&enable) as u32,
                )
            };

            if ret == -1 {
                warn!("Failed to set SO_TIMESTAMPING");
                support_tx_timestamping = false;
            }
        } else {
            let support_tx_timestamping = false;
        }
    }

    // Payload 初始化
    // 初始化了四种不同的 Payload 数据内容
    // 全零数据
    let zero_payload = vec![0; popt.len];
    // 全一数据
    let one_payload = vec![1; popt.len];
    // 全 0x5A 数据
    let fivea_payload = vec![0x5A; popt.len];
    // 随机数据 + 全零数据 + 全一数据 + 全 0x5A 数据组成 payloads
    let payloads: [&[u8]; 4] = [&rand_payload, &zero_payload, &one_payload, &fivea_payload];

    // SyncLimiter 的初始化
    // 使用 SyncLimiter 类型创建了一个速率限制器, 用于控制发送速率
    let limiter = SyncLimiter::full(popt.rate, Duration::from_millis(1000));
    let mut seq = 1u16;
    let mut sent_count = 0;

    // Linux 环境下的缓冲区初始化
    // 主要初始化了用于网络通信的缓冲区和相关的结构体, 如 iovec 和 msghdr
    cfg_if! {
        if #[cfg(target_os = "linux")] {
            let mut buf = [0; 2048];
            let mut control_buf = [0; 1024];
            let mut iovec = iovec {
                iov_base: buf.as_mut_ptr() as *mut c_void,
                iov_len: buf.len(),
            };

            let mut msghdr = msghdr {
                msg_name: std::ptr::null_mut(),
                msg_namelen: 0,
                msg_iov: &mut iovec,
                msg_iovlen: 1,
                msg_control: control_buf.as_mut_ptr() as *mut c_void,
                msg_controllen: control_buf.len(),
                msg_flags: 0,
            };
        }
    }

    // 发送循环
    loop {
        // 根据启动参数选择是否启用限速器
        if !popt.rate_for_all {
            limiter.take();
        }
        let payload = payloads[seq as usize % payloads.len()];
        // 遍历目标地址集合, 发送 ICMP Echo 请求
        for ip in &addrs {
            if popt.rate_for_all {
                limiter.take();
            }

            // 构造 ICMP Echo 请求包
            let mut buf = vec![0; 8 + payload.len()]; // 8 bytes of header, then payload
            let mut packet = echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
            packet.set_icmp_type(icmp::IcmpTypes::EchoRequest);
            packet.set_identifier(pid);
            packet.set_sequence_number(seq);

            let now = SystemTime::now();
            let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
            let timestamp = since_the_epoch.as_nanos();

            let ts_bytes = timestamp.to_be_bytes();
            let mut send_payload = vec![0; payload.len()];
            send_payload[..16].copy_from_slice(&ts_bytes[..16]);
            send_payload[16..].copy_from_slice(&payload[16..]);

            packet.set_payload(&send_payload);

            let icmp_packet = icmp::IcmpPacket::new(packet.packet()).unwrap();
            let checksum = icmp::checksum(&icmp_packet);
            packet.set_checksum(checksum);

            let dest = SocketAddr::new(*ip, 0);
            let key = timestamp / 1_000_000_000;
            let target = dest.ip().to_string();

            let data = send_buckets.lock().unwrap();
            data.add(
                key,
                Result {
                    txts: timestamp,
                    target: target.clone(),
                    seq,
                    latency: 0,
                    received: false,
                    bitflip: false,
                    ..Default::default()
                },
            );
            drop(data);

            // 发送 ICMP Echo 请求包
            match socket.send_to(&buf, &dest.into()) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error in send: {:?}", e);
                    return Err(e.into());
                }
            }

            // 如果支持时间戳, 接收并处理
            if support_tx_timestamping {
                cfg_if! {
                    if #[cfg(target_os = "linux")] {
                        unsafe {
                            let _ = recvmsg(raw_fd, &mut msghdr, MSG_ERRQUEUE | MSG_DONTWAIT);
                        }
                        if let Some(txts) = get_timestamp(&mut msghdr) {
                            let ts = txts.duration_since(UNIX_EPOCH).unwrap().as_nanos();
                            let data = send_buckets.lock().unwrap();
                            data.update_txts(key, target, seq, ts);
                            drop(data);
                        }
                    }
                }
            }
        }

        // 更新序列号和发送计数
        seq += 1;
        sent_count += 1;

         // 如果设置了发送次数限制, 达到次数后退出循环
        if popt.count.is_some() && sent_count >= popt.count.unwrap() {
            thread::sleep(Duration::from_secs(popt.delay));
            info!("reached {} and exit", sent_count);
            if has_tx {
                return Ok(());
            }
            std::process::exit(0);
        }
    }
}
```

## 接收逻辑

用于接收 ICMP Echo 回复消息, 并处理其中的信息,通俗讲就是在一个死循环中读,大部分逻辑和发送类似

```rust
fn read(
    socket2: Socket,
    popt: PingOption,
    read_buckets: Arc<Mutex<Buckets>>,
    pid: u16,
    read_rand_payload: Vec<u8>,
) -> anyhow::Result<()> {
    // 同 send 的 Payload 初始化
    let zero_payload = vec![0; popt.len];
    let one_payload = vec![1; popt.len];
    let fivea_payload = vec![0x5A; popt.len];

    let payloads: [&[u8]; 4] = [
        &read_rand_payload,
        &zero_payload,
        &one_payload,
        &fivea_payload,
    ];

    // 设置 socket 读取超时和时间戳选项
    socket2.set_read_timeout(Some(popt.timeout))?;
    let raw_fd = socket2.as_raw_fd();

    // Linux 环境下的缓冲区和结构体初始化
    // 同 send
    cfg_if! {
        if #[cfg(target_os = "linux")] {
            let enable = SOF_TIMESTAMPING_SOFTWARE
                | SOF_TIMESTAMPING_TX_SOFTWARE
                | SOF_TIMESTAMPING_RX_SOFTWARE
                | SOF_TIMESTAMPING_SYS_HARDWARE
                | SOF_TIMESTAMPING_TX_HARDWARE
                | SOF_TIMESTAMPING_RX_HARDWARE
                | SOF_TIMESTAMPING_RAW_HARDWARE
                | SOF_TIMESTAMPING_OPT_CMSG
                | SOF_TIMESTAMPING_OPT_TSONLY;
            let ret = unsafe {
                setsockopt(
                    raw_fd,
                    SOL_SOCKET,
                    SO_TIMESTAMPING,
                    &enable as *const _ as *const c_void,
                    mem::size_of_val(&enable) as u32,
                )
            };
            if ret == -1 {
                warn!("Failed to set read SO_TIMESTAMPING");
                let enable: c_int = 1;
                let ret = unsafe {
                    setsockopt(
                        raw_fd,
                        SOL_SOCKET,
                        SO_TIMESTAMP,
                        &enable as *const _ as *const c_void,
                        std::mem::size_of_val(&enable) as u32,
                    )
                };
                if ret == -1 {
                    warn!("Failed to set SO_TIMESTAMP");
                }
            }
        }
    }

    let mut buffer: [u8; 2048] = [0; 2048];
    let mut control_buf = [0; 1024];

    let mut iovec = iovec {
        iov_base: buffer.as_mut_ptr() as *mut c_void,
        iov_len: buffer.len(),
    };

    cfg_if! {
        if #[cfg(target_os = "linux")] {
            let mut msghdr = msghdr {
                msg_name: std::ptr::null_mut(),
                msg_namelen: 0,
                msg_iov: &mut iovec,
                msg_iovlen: 1,
                msg_control: control_buf.as_mut_ptr() as *mut c_void,
                msg_controllen: control_buf.len(),
                msg_flags: 0,
            };
        } else {
            let mut msghdr = msghdr {
                msg_name: std::ptr::null_mut(),
                msg_namelen: 0,
                msg_iov: &mut iovec,
                msg_iovlen: 1,
                msg_control: control_buf.as_mut_ptr() as *mut c_void,
                msg_controllen: control_buf.len() as u32,
                msg_flags: 0,
            };
        }
    }

    loop {
        let nbytes = unsafe { recvmsg(raw_fd, &mut msghdr, 0) };
        if nbytes == -1 {
            let err = Error::last_os_error();
            if err.kind() == ErrorKind::WouldBlock {
                continue;
            }

            error!("Failed torr receive message");
            return Err(Error::new(ErrorKind::Other, "Failed to receive message").into());
        }

        let buf = &buffer[..nbytes as usize];

        // 解析 ICMP Echo 回复消息
        let ipv4_packet = Ipv4Packet::new(buf).unwrap();
        let icmp_packet = pnet_packet::icmp::IcmpPacket::new(ipv4_packet.payload()).unwrap();

        // 判断 ICMP 报文类型和代码
        if icmp_packet.get_icmp_type() != IcmpTypes::EchoReply
            || icmp_packet.get_icmp_code() != echo_reply::IcmpCodes::NoCode
        {
            continue;
        }

        // 解析 Echo 回复消息
        let echo_reply = match icmp::echo_reply::EchoReplyPacket::new(icmp_packet.packet()) {
            Some(echo_reply) => echo_reply,
            None => {
                continue;
            }
        };

        // 根据 Echo 回复消息中的信息进行处理, 例如比较标识符、序列号等
        if echo_reply.get_identifier() != pid {
            continue;
        }

        let mut bitflip = false;
        if payloads[echo_reply.get_sequence_number() as usize % payloads.len()][16..]
            != echo_reply.payload()[16..]
        {
            warn!(
                "bitflip detected! seq={:?},",
                echo_reply.get_sequence_number()
            );
            bitflip = true;
        }

        let payload = echo_reply.payload();
        let ts_bytes = &payload[..16];
        let txts = u128::from_be_bytes(ts_bytes.try_into().unwrap());
        let dest_ip = ipv4_packet.get_source();

        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();

        cfg_if! {
            if #[cfg(target_os = "linux")] {
                let mut timestamp = since_the_epoch.as_nanos();
                if let Some(rxts) = get_timestamp(&mut msghdr) {
                    timestamp = rxts.duration_since(UNIX_EPOCH).unwrap().as_nanos();
                }
            } else {
                let timestamp = since_the_epoch.as_nanos();
            }
        }

        // 记录结果到数据结构中
        let buckets = read_buckets.lock().unwrap();
        // 将回复信息加入 bucket 结构
        buckets.add_reply(
            txts / 1_000_000_000,
            Result {
                txts,
                rxts: timestamp,
                target: dest_ip.to_string(),
                seq: echo_reply.get_sequence_number(),
                latency: 0,
                received: true,
                bitflip,
            },
        );
    }
}
```

## 统计逻辑

通过定义 Buckets ,代表当前还未处理的一批桶 Bucket

因为发送完数据还需要留出缓冲时间 delay 等待回包, 所以会保留最近一段时间 Bucket, 

让回包有足够的时间翻入到相应的 Bucket 中, 因为是按照秒进行统计的, 每个 Bucket 代表一秒内发送的包以及它相应的回包信息. 

只需每秒定时的把最久的 Bucket 摘取掉, 把它的统计数据输出出来就行

```rust
// Buckets 用于存储所有未处理的 Bucket
#[derive(Default)]
pub struct Buckets {
    // buckets 是一个优先级队列, 队列最前面的是 key 最小的 Bucket
    pub buckets: Mutex<BinaryHeap<Bucket>>,
    // 利用 HashMap 通过 key 快速查找 Bucket
    pub map: Mutex<HashMap<u128, Bucket>>,
}
```

Buckets 实现了一个最小堆, 这样就方便的弹出最久的那个 Bucket.  map 保存每一个 Bucket, 方便我们查找和更新, 它的主键是时间戳(以秒为单位). 

接下来它要实现 add、add_reply、pop、last 等方法:

```rust
impl Buckets {
    // 创建一个新的 Buckets
    pub fn new_buckets() -> Buckets {
        Buckets {
            buckets: Mutex::new(BinaryHeap::new()),
            map: Mutex::new(HashMap::new()),
        }
    }

    // 将 ping 结果添加到 Buckets 中, Bucket 中的 key 是以秒为单位的时间戳
    pub fn add(&self, key: u128, value: Result) {
        let mut map = self.map.lock().unwrap();
        map.entry(key).or_insert_with(|| {
            let bucket = Bucket::new_bucket(key);
            self.buckets.lock().unwrap().push(bucket.clone());
            bucket
        });

        let bucket = map.get(&key).unwrap();
        bucket.add(value);
    }

    // 将 ping 回复添加到 Buckets 中, Bucket 中的 key 是以秒为单位的时间戳
    pub fn add_reply(&self, key: u128, result: Result) {
        let mut map = self.map.lock().unwrap();

        map.entry(key).or_insert_with(|| {
            self.buckets.lock().unwrap().push(Bucket::new_bucket(key));
            Bucket::new_bucket(key)
        });

        let bucket = map.get(&key).unwrap();
        bucket.add_reply(result);
    }

    // 发送后更新 ping 结果的 txts（软件/硬件时间戳）
    // Bucket 中的 key 是以秒为单位的时间戳
    pub fn update_txts(&self, key: u128, target: String, seq: u16, txts: u128) {
        let map = self.map.lock().unwrap();

        if let Some(bucket) = map.get(&key) {
            bucket.update_txts(target, seq, txts);
        }
    }

    // 用最小的 key 弹出 bucket
    pub fn pop(&self) -> Option<Bucket> {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets.pop()?;
        let bucket = self.map.lock().unwrap().remove(&bucket.key).unwrap();
        Some(bucket)
    }

    // 用最小的 key 获取 bucket, bucket 还在堆和 map 中
    pub fn last(&self) -> Option<Bucket> {
        let buckets = self.buckets.lock().unwrap();
        buckets.peek().cloned()
    }
}
```

* add方法主要是发送模块使用
  * 在发送一个数据后, 根据发送时间戳, 放入到对应的 Bucket 中

* add_reply方法主要是接收模块使用
  * 在接收一个数据后, 更新它的时延, 并标记此 seq 已经接收到数据了. 

* pop和last方便统计的时候使用
  * 因为定时器需要定时的检查最后的 Bucket 存不存在, 应不应该进行统计. 

Bucket 代表一个 桶, 对应每一秒的统计数据

```rust
// Bucket 用于存储所有目标一秒钟内的 ping 结果.
#[derive(Default)]
pub struct Bucket {
    // key 是时间戳, 以秒为单位
    pub key: u128,
    // 值是 Bucket 中所有目标 ping 结果.
    pub value: RwLock<HashMap<String, Result>>,
}
```

key 使用时间戳(截短到秒),  value 值代表每一个 seq 对应的请求和响应数据. 

实现的方法就是和 Buckets 的方法对应, add 和 add_reply 用来增加和更新统计数据. 

以上 Buckets 和 Bucket 是数据结构, 定义好这个数据结构后, 就可以方便的统计了. 

## 数据输出

根据上面统计模块定义的数据格式进行获取输出即可

```rust
fn print_stat(
    buckets: Arc<Mutex<Buckets>>,
    popt: PingOption,
    enable_print_stat: bool,
    tx: Option<Sender<TargetResult>>,
) -> anyhow::Result<()> {
    // 统计打印的初始化和配置
    // 包括延迟、最后一个 key、是否有发送通道以及一个 Ticker, 用于每秒钟进行定期操作
    let delay = Duration::from_secs(popt.delay).as_nanos(); // 5s
    let mut last_key = 0;

    let has_sender = tx.is_some();

    let ticker = Ticker::new(0.., Duration::from_secs(1));

    //定期统计信息输出
    // 使用 Ticker 来定期执行循环体, 获取当前存储 bucket 的信息
    // 检查是否为空, 然后进行后续统计逻辑
    for _ in ticker {
        let buckets = buckets.lock().unwrap();
        let bucket = buckets.last();
        if bucket.is_none() {
            continue;
        }

        // bucket 信息的处理
        // 检查 bucket key, 并进行后续统计逻辑
        let bucket = bucket.unwrap();
        // 如果 bucket key 小于等于上一次处理的 key, 则弹出该存储桶, 继续下一个循环
        if bucket.key <= last_key {
            buckets.pop();
            continue;
        }

        // 然后检查 bucket 是否在指定的时间范围内, 如果是, 执行后续的统计逻辑
        if bucket.key
            <= SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                - delay
        {
            // 计算统计信息并输出
            if let Some(pop) = buckets.pop() {
                if pop.key < bucket.key {
                    continue;
                }

                last_key = pop.key;

                // cacl stat
                let mut target_results = BTreeMap::new();

                for r in pop.values() {
                    let target_result = target_results
                        .entry(r.target.clone())
                        .or_insert_with(TargetResult::default);

                    target_result.latency += r.latency;

                    if r.received {
                        target_result.received += 1;
                    } else {
                        target_result.loss += 1;
                    }

                    if r.bitflip {
                        target_result.bitflip_count += 1;
                    }
                }

                // 输出和通道发送
                for (target, tr) in &target_results {
                    let total = tr.received + tr.loss;
                    let loss_rate = if total == 0 {
                        0.0
                    } else {
                        (tr.loss as f64) / (total as f64)
                    };

                    if enable_print_stat {
                        if tr.received == 0 {
                            info!(
                                "{}: sent:{}, recv:{}, loss rate: {:.2}%, latency: {}ms",
                                target,
                                total,
                                tr.received,
                                loss_rate * 100.0,
                                0
                            )
                        } else {
                            info!(
                                "{}: sent:{}, recv:{},  loss rate: {:.2}%, latency: {:.2}ms",
                                target,
                                total,
                                tr.received,
                                loss_rate * 100.0,
                                Duration::from_nanos(tr.latency as u64 / (tr.received as u64))
                                    .as_secs_f64()
                                    * 1000.0
                            )
                        }
                    }

                    // 如果有发送接收, 将结果发送过去
                    if has_sender {
                        let mut tr = tr.clone();
                        tr.target = target.clone();
                        tr.loss_rate = loss_rate;
                        let _ = tx.as_ref().unwrap().send(tr);
                    }
                }
            }
        }
    }

    Ok(())
}
```

## 软硬件时间戳

### 概述

Go 版本的 mping 对于低版本的内核, 不支持 SO_TIMESTAMPING 的话, 会退化使用 SO_TIMESTAMPNS

mping-rs 会使用 SO_TIMESTAMP,这并没有啥特殊的设计, 两者都可以, 只是从 Out-Of-Bound 控制数据中读取的数据结构略有不同. 

### 实现逻辑

以接收数据的时候为例

#### 核心问题

* 如何把返回的数据包进入网卡后涉及的硬件时间戳获取出来

#### 问题说明

* 对于发送的时候, 包进入本地 socket 的缓存, 这时候应用层的调用就返回了
* 此时还没有进入内核的协议栈的处理
* 需要考虑获取协议栈设置的软件时间戳
* 需要考虑获取网卡驱动设置的硬件时间戳

#### 解决方案

1. 读取软硬件时间戳, 需要 setsockopt 把相应的参数设置上
2. 较新版本的 Linux 内核版本支持 SO_TIMESTAMPING 选项, 设置这个选项的时候需要设置一堆的 flags
3. 指定要不要把发送和接收的软硬件时间戳的 flag 加上, 把 CMSG 和 TSONLY flag 设置上
4. 最终是从控制信息中获取时间戳
    * 如果同时设置软硬件时间戳, 会尽量返回硬件时间戳
        * 如果机器不支持硬件时间戳, 那就返回软件时间戳
        * 如果机器软件时间戳也不支持, 那就没有时间戳可返回. 

以下逻辑就是设置这些参数, 先尝试设置 SO_TIMESTAMPING 选项,不成功的话再尝试设置 SO_TIMESTAMP 选项:

```rust
    cfg_if! {
        if #[cfg(target_os = "linux")] {
            let enable = SOF_TIMESTAMPING_SOFTWARE
                | SOF_TIMESTAMPING_TX_SOFTWARE
                | SOF_TIMESTAMPING_RX_SOFTWARE
                | SOF_TIMESTAMPING_SYS_HARDWARE
                | SOF_TIMESTAMPING_TX_HARDWARE
                | SOF_TIMESTAMPING_RX_HARDWARE
                | SOF_TIMESTAMPING_RAW_HARDWARE
                | SOF_TIMESTAMPING_OPT_CMSG
                | SOF_TIMESTAMPING_OPT_TSONLY;
            let ret = unsafe {
                setsockopt(
                    raw_fd,
                    SOL_SOCKET,
                    SO_TIMESTAMPING,
                    &enable as *const _ as *const c_void,
                    mem::size_of_val(&enable) as u32,
                )
            };
            if ret == -1 {
                warn!("Failed to set read SO_TIMESTAMPING");
                let enable: c_int = 1;
                let ret = unsafe {
                    setsockopt(
                        raw_fd,
                        SOL_SOCKET,
                        SO_TIMESTAMP,
                        &enable as *const _ as *const c_void,
                        std::mem::size_of_val(&enable) as u32,
                    )
                };
                if ret == -1 {
                    warn!("Failed to set SO_TIMESTAMP");
                }
            }
        }
    }
```

因为需要读取控制信息,  socket2 库提供的 Socket 不支持相应额度读取, 官方仓库中也有 issue 提到这个需求, 但是没人实现, 所以需要使用 libc 的 recvmsg 方法实现. Gopher 应该庆幸 Rob Pike 这些大师在设计 Go 标准库的经验和能力, Go 标准库早就支持方便的读取 OOB 控制数据了. 

而使用 libc 的系统调用读取 OOB 数据还比较麻烦, 首先我们要准备一些数据,最终组成一个 msghdr 变量, 这是recvmsg 系统调用所必须的：

```rust
    cfg_if! {
        if #[cfg(target_os = "linux")] {
            let mut msghdr = msghdr {
                msg_name: std::ptr::null_mut(),
                msg_namelen: 0,
                msg_iov: &mut iovec,
                msg_iovlen: 1,
                msg_control: control_buf.as_mut_ptr() as *mut c_void,
                msg_controllen: control_buf.len(),
                msg_flags: 0,
            };
        } else {
            let mut msghdr = msghdr {
                msg_name: std::ptr::null_mut(),
                msg_namelen: 0,
                msg_iov: &mut iovec,
                msg_iovlen: 1,
                msg_control: control_buf.as_mut_ptr() as *mut c_void,
                msg_controllen: control_buf.len() as u32,
                msg_flags: 0,
            };
        }
    }
```

然后改造一下读取的逻辑, 换成 recvmsg 系统调用:

```rust
    loop {
        let nbytes = unsafe { recvmsg(raw_fd, &mut msghdr, 0) };
        if nbytes == -1 {
            let err = Error::last_os_error();
            if err.kind() == ErrorKind::WouldBlock {
                continue;
            }

            error!("Failed torr receive message");
            return Err(Error::new(ErrorKind::Other, "Failed to receive message").into());
        }

        let buf = &buffer[..nbytes as usize];

        // 解析 ICMP Echo 回复消息
        let ipv4_packet = Ipv4Packet::new(buf).unwrap();
        let icmp_packet = pnet_packet::icmp::IcmpPacket::new(ipv4_packet.payload()).unwrap();
......
```

这里通过 msghdr 这个变量, 就可以得到控制信息 msg_control,长度是 msg_controllen. 

接下来处理控制信息, 从里面把时间戳的控制信息解析出来, 抽成了一个函数:

```rust
// 用于从 Linux socket 消息头中提取时间戳的函数
// 接受一个 msghdr, 返回一个 Option<SystemTime>
#[cfg(target_os = "linux")]
fn get_timestamp(msghdr: &mut msghdr) -> Option<SystemTime> {
    // 获取 CMSG 指针
    // 使用 libc::CMSG_FIRSTHDR 获取第一个 CMSG（控制消息）头的指针
    // 在后续的循环中, 将迭代 CMSG 消息头
    let mut cmsg: *mut cmsghdr = unsafe { libc::CMSG_FIRSTHDR(msghdr) };

    while !cmsg.is_null() {
        // 判断 CMSG 消息头的 level 和 type, 并提取时间戳
        // 分别处理 SO_TIMESTAMP 和 SCM_TIMESTAMPING 两种情况
        // 处理 SO_TIMESTAMP
        // 如果 CMSG 消息头的 level 是 SOL_SOCKET, type 是 SO_TIMESTAMP, 则提取 timeval 结构体, 将其转换为 SystemTime
        if unsafe { (*cmsg).cmsg_level == SOL_SOCKET && (*cmsg).cmsg_type == SO_TIMESTAMP } {
            let tv: *mut timeval = unsafe { libc::CMSG_DATA(cmsg) } as *mut timeval;
            let timestamp = unsafe { *tv };
            return Some(
                SystemTime::UNIX_EPOCH
                    + Duration::new(timestamp.tv_sec as u64, timestamp.tv_usec as u32 * 1000),
            );
        }

        // 处理 SCM_TIMESTAMPING
        // 如果 CMSG 消息头的 level 是 SOL_SOCKET, type 是 SCM_TIMESTAMPING, 则提取 [timespec; 3] 数组, 遍历其中的 timespec, 将其转换为 SystemTime
        if unsafe { (*cmsg).cmsg_level == SOL_SOCKET && (*cmsg).cmsg_type == SCM_TIMESTAMPING } {
            let tv: *mut [timespec; 3] = unsafe { libc::CMSG_DATA(cmsg) } as *mut [timespec; 3];
            let timestamps = unsafe { *tv };

            for timestamp in &timestamps {
                if timestamp.tv_sec != 0 || timestamp.tv_nsec != 0 {
                    let seconds = Duration::from_secs(timestamp.tv_sec as u64);
                    let nanoseconds = Duration::from_nanos(timestamp.tv_nsec as u64);
                    if let Some(duration) = seconds.checked_add(nanoseconds) {
                        return Some(SystemTime::UNIX_EPOCH + duration);
                    }
                }
            }
        }

        // 循环迭代 CMSG 消息头
        // 使用 libc::CMSG_NXTHDR 获取下一个 CMSG 消息头的指针, 用于下一次循环迭代
        cmsg = unsafe { libc::CMSG_NXTHDR(msghdr, cmsg) };
    }

    None
}
```

因为控制信息中可能包含多条信息, 需要遍历找出需要的控制信息. 

1. 对于设置了 SO_TIMESTAMPING 的场景

    * 通过 `(*cmsg).cmsg_level == SOL_SOCKET && (*cmsg).cmsg_type == SCM_TIMESTAMPING` 把控制信息筛选到

    * 它的信息是包含三个 timespec 的数据, 一般信息会放在第一个元素中, 但是也可能放入第三个或者第二个, 依次遍历这三个元素, 找到第一个非零的元素即可. 

2. 对于设置了 SO_TIMESTAMP 的场景

    * 通过`(*cmsg).cmsg_level == SOL_SOCKET && (*cmsg).cmsg_type == SO_TIMESTAMP` 筛选出来

    * 它的值是一个类型为timeval的值, 包含秒数和微秒数. 

这样就获取了软硬件的时间戳

## 最后

可以在 Cargo.toml 中添加上下面的信息进行优化，剔除符号表等信息:

```toml
[profile.release]
lto = true
strip = true
opt-level = "z"
codegen-units = 1
```

至于性能就随便看看了

![go_mping](https://pic.imgdb.cn/item/65c05c369f345e8d031a427d.png)

![rust_mping](https://pic.imgdb.cn/item/65c05c379f345e8d031a4314.png)