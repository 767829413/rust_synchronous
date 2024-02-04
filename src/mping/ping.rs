use core::result::Result::Ok;
use std::collections::BTreeMap;
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, SocketAddr};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        use libc::{
            c_int, c_void, cmsghdr, iovec, msghdr, recvmsg, setsockopt, timespec, timeval, MSG_DONTWAIT,
            MSG_ERRQUEUE, SOL_SOCKET, SO_TIMESTAMP, SO_TIMESTAMPING,
        };
        use libc::{
            SCM_TIMESTAMPING, SOF_TIMESTAMPING_OPT_CMSG, SOF_TIMESTAMPING_OPT_TSONLY,
            SOF_TIMESTAMPING_RAW_HARDWARE, SOF_TIMESTAMPING_RX_HARDWARE, SOF_TIMESTAMPING_RX_SOFTWARE,
            SOF_TIMESTAMPING_SOFTWARE, SOF_TIMESTAMPING_SYS_HARDWARE, SOF_TIMESTAMPING_TX_HARDWARE,
            SOF_TIMESTAMPING_TX_SOFTWARE,
        };
        use std::mem;
    } else {
        use libc::{ c_void, iovec, msghdr, recvmsg};
    }
}

use std::os::unix::io::AsRawFd;

use log::{error, info, warn};
use rand::Rng;
use rate_limit::SyncLimiter;
use ticker::Ticker;

use pnet_packet::icmp::{self, echo_reply, echo_request, IcmpTypes};
use pnet_packet::ipv4::Ipv4Packet;
use pnet_packet::Packet;
use socket2::{Domain, Protocol, Socket, Type};

use crate::mping::stat::{Buckets, Result, TargetResult};

/// Ping option struct for ping function.
/// ``` rust
/// use std::time::Duration;
/// use mping::PingOption;
///
/// let popt = PingOption {
///    timeout: Duration::from_secs(1),
///    ttl: 64,
///    tos: None,
///    ident: 1234,
///    len: 56,
///    rate: 100,
///    rate_for_all: false,
///    delay: 3,
///    count: None,
/// };
/// ```
#[derive(Default, Clone, Debug)]
pub struct PingOption {
    // 每次 ping 写入和读取的超时
    pub timeout: Duration,
    // 每个 ping 数据包的 TTL
    pub ttl: u32,
    // 每个 ping 数据包的 TOS
    pub tos: Option<u32>,
    // 每个 ICMP 数据包的 Id
    pub ident: u32,
    // 每个 ICMP 数据包的有效载荷长度
    pub len: usize,
    // 每个目标的 Ping 速率
    pub rate: u64,
    // 所有目标或单个目标的速率
    // 如果为 true，每个目标将以固定的速率发送数据包
    // 如果为假，每个目标将以各自的速率发送数据包
    pub rate_for_all: bool,
    // 输出 ping 结果的延迟
    pub delay: u64,
    // 每个目标的最大 ping 计数, None 时不检查
    pub count: Option<i64>,
}

/// Ping function.
///
/// # Arguments
///
/// - `addrs` 是一个 IP 地址 vector
/// - `popt` 是 PingOption struct
/// - `enable_print_stat` 是一个 bool 值，用于在日志中打印 ping 状态
/// - `tx`` 是发送 ping 结果的发件人, 如果 tx 为 None，ping 将不会发送结果
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

    // 创建了一个新的套接字，指定了 IPv4 地址族、原始类型（RAW）以及 ICMPv4 协议。如果创建失败，会通过 unwrap() 抛出错误
    let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).unwrap();
    // 设置套接字的 TTL（Time-To-Live）值为 popt.ttl，即生存时间。如果设置失败，会通过 unwrap() 抛出错误
    socket.set_ttl(popt.ttl).unwrap();
    // 设置套接字的写超时时间为 popt.timeout。如果设置失败，会通过 unwrap() 抛出错误
    socket.set_write_timeout(Some(popt.timeout)).unwrap();
    // 如果在命令行选项中指定了 TOS（Type of Service）值，则设置套接字的 TOS 值为指定的值。如果设置失败，会通过 unwrap() 抛出错误
    if let Some(tos_value) = popt.tos {
        socket.set_tos(tos_value).unwrap();
    }
    // 尝试克隆套接字，如果克隆失败，则打印错误信息并终止程序
    let socket2 = socket.try_clone().expect("Failed to clone socket");
    // 检查是否设置接收, 使用 is_some() 方法检查其是否有值，如果有值则返回 true，否则返回 false, 结果存储在 has_tx 变量中
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

fn random_bytes(len: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut vec = vec![0u8; len];
    rng.fill(&mut vec[..]);

    return vec;
}

#[cfg(target_os = "linux")]
fn get_timestamp(msghdr: &mut msghdr) -> Option<SystemTime> {
    let mut cmsg: *mut cmsghdr = unsafe { libc::CMSG_FIRSTHDR(msghdr) };

    while !cmsg.is_null() {
        if unsafe { (*cmsg).cmsg_level == SOL_SOCKET && (*cmsg).cmsg_type == SO_TIMESTAMP } {
            let tv: *mut timeval = unsafe { libc::CMSG_DATA(cmsg) } as *mut timeval;
            let timestamp = unsafe { *tv };
            return Some(
                SystemTime::UNIX_EPOCH
                    + Duration::new(timestamp.tv_sec as u64, timestamp.tv_usec as u32 * 1000),
            );
        }
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

        cmsg = unsafe { libc::CMSG_NXTHDR(msghdr, cmsg) };
    }

    None
}

fn send(
    socket: Socket,
    addrs: Vec<IpAddr>,
    popt: PingOption,
    send_buckets: Arc<Mutex<Buckets>>,
    rand_payload: Vec<u8>,
    pid: u16,
    has_tx: bool,
) -> anyhow::Result<()> {
    cfg_if! {
        if #[cfg(target_os = "linux")] {
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

    let zero_payload = vec![0; popt.len];
    let one_payload = vec![1; popt.len];
    let fivea_payload = vec![0x5A; popt.len];

    let payloads: [&[u8]; 4] = [&rand_payload, &zero_payload, &one_payload, &fivea_payload];

    let limiter = SyncLimiter::full(popt.rate, Duration::from_millis(1000));
    let mut seq = 1u16;
    let mut sent_count = 0;

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

    loop {
        if !popt.rate_for_all {
            limiter.take();
        }

        let payload = payloads[seq as usize % payloads.len()];
        for ip in &addrs {
            if popt.rate_for_all {
                limiter.take();
            }

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

            match socket.send_to(&buf, &dest.into()) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error in send: {:?}", e);
                    return Err(e.into());
                }
            }

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

        seq += 1;
        sent_count += 1;

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
// , addrs: Vec<IpAddr>, popt: PingOption, enable_print_stat: bool
fn read(
    socket2: Socket,
    popt: PingOption,
    read_buckets: Arc<Mutex<Buckets>>,
    pid: u16,
    read_rand_payload: Vec<u8>,
) -> anyhow::Result<()> {
    let zero_payload = vec![0; popt.len];
    let one_payload = vec![1; popt.len];
    let fivea_payload = vec![0x5A; popt.len];

    let payloads: [&[u8]; 4] = [
        &read_rand_payload,
        &zero_payload,
        &one_payload,
        &fivea_payload,
    ];

    socket2.set_read_timeout(Some(popt.timeout))?;
    let raw_fd = socket2.as_raw_fd();

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

        let ipv4_packet = Ipv4Packet::new(buf).unwrap();
        let icmp_packet = pnet_packet::icmp::IcmpPacket::new(ipv4_packet.payload()).unwrap();

        if icmp_packet.get_icmp_type() != IcmpTypes::EchoReply
            || icmp_packet.get_icmp_code() != echo_reply::IcmpCodes::NoCode
        {
            continue;
        }

        let echo_reply = match icmp::echo_reply::EchoReplyPacket::new(icmp_packet.packet()) {
            Some(echo_reply) => echo_reply,
            None => {
                continue;
            }
        };

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

        let buckets = read_buckets.lock().unwrap();
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

fn print_stat(
    buckets: Arc<Mutex<Buckets>>,
    popt: PingOption,
    enable_print_stat: bool,
    tx: Option<Sender<TargetResult>>,
) -> anyhow::Result<()> {
    let delay = Duration::from_secs(popt.delay).as_nanos(); // 5s
    let mut last_key = 0;

    let has_sender = tx.is_some();

    let ticker = Ticker::new(0.., Duration::from_secs(1));
    for _ in ticker {
        let buckets = buckets.lock().unwrap();
        let bucket = buckets.last();
        if bucket.is_none() {
            continue;
        }

        let bucket = bucket.unwrap();
        if bucket.key <= last_key {
            buckets.pop();
            continue;
        }

        if bucket.key
            <= SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                - delay
        {
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

                // output
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
