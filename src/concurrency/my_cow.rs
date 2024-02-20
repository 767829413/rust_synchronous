use std::borrow::Cow;
use beef;
use std::mem::size_of;

pub fn cow_exp_string() {
    let s1 = String::from("hello");
    println!("s1={}", s1);
    let mut s2 = s1; // s1和s2共享同一份内存

    s2.push_str(" world"); // s2会进行写操作,于是系统复制一份新的内存给s2

    println!("s2={}", s2);
}

pub fn cow_container_string() {
    let origin = "hello world";
    let cow = Cow::from(origin);
    println!("left: {} right: {}", cow, "hello world");

    // Cow can be borrowed as a str
    let s: &str = &cow;
    println!("left: {} right: {}", s, "hello world");

    println!("left: {} right: {}", s.len(), cow.len());

    // Cow can be converted to a String
    let s: String = cow.into();
    println!("left: {} right: {}", s, "HELLO WORLD");

    let mut cow1 = Cow::from(origin);
    // Cow can be borrowed as a mut str
    let s: &mut str = cow1.to_mut();
    s.make_ascii_uppercase();
    assert_eq!(s, "HELLO WORLD");
    assert_eq!(origin, "hello world");
}

pub fn beef_cow() {
    let borrowed: beef::Cow<str> = beef::Cow::borrowed("Hello");
    let owned: beef::Cow<str> = beef::Cow::owned(String::from("World"));
    let _ = beef::Cow::from("Hello");

    assert_eq!(format!("{} {}!", borrowed, owned), "Hello World!",);

    const WORD: usize = size_of::<usize>();

    assert_eq!(size_of::<std::borrow::Cow<str>>(), 3 * WORD);
    assert_eq!(size_of::<beef::Cow<str>>(), 3 * WORD);
    assert_eq!(size_of::<beef::lean::Cow<str>>(), 2 * WORD);
}
