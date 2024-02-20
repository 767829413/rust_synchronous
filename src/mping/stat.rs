use std::cmp::Ordering;
use std::{
    collections::BinaryHeap,
    collections::HashMap,
    sync::{Mutex, RwLock},
};

// Buckets 用于存储所有未处理的 Bucket
#[derive(Default)]
pub struct Buckets {
    // buckets 是一个优先级队列，队列最前面的是 key 最小的 Bucket
    pub buckets: Mutex<BinaryHeap<Bucket>>,
    // 利用 HashMap 通过 key 快速查找 Bucket
    pub map: Mutex<HashMap<u128, Bucket>>,
}

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

// Bucket 用于存储所有目标一秒钟内的 ping 结果.
#[derive(Default)]
pub struct Bucket {
    // key 是时间戳，以秒为单位
    pub key: u128,
    // 值是 Bucket 中所有目标 ping 结果.
    pub value: RwLock<HashMap<String, Result>>,
}

impl Clone for Bucket {
    fn clone(&self) -> Self {
        let v = self.value.read().unwrap().clone();
        return Bucket {
            key: self.key,
            value: RwLock::new(v),
        };
    }
}

impl Ord for Bucket {
    fn cmp(&self, other: &Self) -> Ordering {
        // 根据 key 进行比较
        self.key.cmp(&other.key).reverse()
    }
}

impl PartialOrd for Bucket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Bucket {}

impl PartialEq for Bucket {
    fn eq(&self, other: &Self) -> bool {
        return self.key == other.key;
    }
}

impl Bucket {
    // 创建一个 Bucket
    fn new_bucket(key: u128) -> Bucket {
        return Bucket {
            key: key,
            value: RwLock::new(HashMap::new()),
        };
    }

    // 往 Bucket 添加一个 ping 结果
    pub fn add(&self, result: Result) {
        let mut map = self.value.write().unwrap();
        let key = format!("{}-{}", result.target, result.seq);
        map.insert(key, result);
    }

    // 往 Bucket 添加一个 ping 回复
    pub fn add_reply(&self, mut reply: Result) {
        let mut map = self.value.write().unwrap();

        let key = format!("{}-{}", reply.target, reply.seq);
        if let Some(req) = map.get(&key) {
            reply.txts = req.txts;
            reply.calc_latency();
        }
        map.insert(key, reply.clone());
    }

    // 发送后, 更新 ping 结果的 txts（软件/硬件时间戳）
    pub fn update_txts(&self, target: String, seq: u16, txts: u128) {
        let mut map = self.value.write().unwrap();

        let key = format!("{}-{}", target, seq);
        if let Some(result) = map.get_mut(&key) {
            result.txts = txts;
        }
    }

    // 获取所有目标的 ping 结果
    pub fn values(&self) -> Vec<Result> {
        let map = self.value.read().unwrap();
        return map.values().cloned().collect();
    }
}

// Result 用于存储一个目标的一次 ping 结果.
// 结果由目标和序列号标识.
#[derive(Default, Clone, Debug)]
pub struct Result {
    // 发送 ping 请求的时间戳.
    pub txts: u128,
    // 收到 ping 回复时的时间戳.
    pub rxts: u128,
    // ping 请求的序列号.
    pub seq: u16,
    // ping 结果的目标.
    pub target: String,
    // ping 结果的延迟.
    pub latency: u128,
    // 如果收到了 ping 回复，则 received 为 true.
    pub received: bool,
    // 如果收到 ping 回复但数据已损坏，则 bitflip 为真.
    pub bitflip: bool,
}

impl Result {
    // 创建一个 ping 结果
    #[allow(unused)]
    fn new_result(txts: u128, target: &str, seq: u16) -> Result {
        Result {
            txts,
            target: target.to_string(),
            seq,
            ..Default::default()
        }
    }
    // 计算 ping 结果的延迟.
    pub fn calc_latency(&mut self) {
        self.latency = self.rxts - self.txts;
    }
}

// TargetResult 用于存储一个目标的 ping 统计结果
#[derive(Default, Clone, Debug)]
pub struct TargetResult {
    // ping 结果的目标.
    pub target: String,
    // ping 结果的丢失率
    pub loss_rate: f64,
    // ping 结果的平均延迟
    pub latency: u128,
    // ping 结果的损失计数
    pub loss: u32,
    // ping 结果的接收计数
    pub received: u32,
    // ping 结果的 bitflip 计数
    pub bitflip_count: u32,
}
