use std::sync::Once;

static mut GLOBAL_CONFIG: Option<String> = None;
static INIT_EXP: Once = Once::new();
static INIT_CONFIG: Once = Once::new();

pub fn once_exp() {
    // 通过 call_once 方法确保某个操作只执行一次
    INIT_EXP.call_once(|| {
        // 这里放置需要执行一次的初始化代码
        println!("Initialization code executed!");
    });

    // 之后再调用 call_once，初始化代码不会再次执行
    INIT_EXP.call_once(|| {
        println!("This won't be printed.");
    });
}

fn init_global_config() {
    println!("Init Config");
    unsafe {
        GLOBAL_CONFIG = Some("Initialized global configuration".to_string());
    }
}

fn get_global_config() -> &'static str {
    INIT_CONFIG.call_once(|| init_global_config());
    unsafe { GLOBAL_CONFIG.as_ref().unwrap() }
}

pub fn once_exp_get_config() {
    println!("start");
    println!("init: {}", get_global_config());
    println!("get: {}", get_global_config()); // 不会重新初始化，只会输出一次
}

pub fn once_cell_exp() {
    use once_cell::sync::OnceCell;

    static CELL: OnceCell<String> = OnceCell::new();
    assert!(CELL.get().is_none());

    std::thread::spawn(|| {
        let value: &String = CELL.get_or_init(|| "Hello, World!".to_string());
        assert_eq!(value, "Hello, World!");
    })
    .join()
    .unwrap();

    let value: Option<&String> = CELL.get();
    assert!(value.is_some());
    assert_eq!(value.unwrap().as_str(), "Hello, World!");
    println!("once_cell {:?}", value);
}
