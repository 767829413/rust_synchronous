# Rust并发数据安全操作

## 介绍

`Rust` 在并发编程方面有一些强大的原语, 可以写出安全且高效的并发代码. 

最显著的原语之一是 `ownership system`, 允许在没有锁的情况下管理内存访问. 此外, `Rust` 还提供了一些并发编程的工具和标准库,比如线程、线程池、消息通讯(`mpsc` 等)、原子操作等, 这些工具和库大部分有时间再说.

这次主要介绍的是保证在线程间共享的一些方式和库以及一些和`Rust`容器类相关的并发原语,比如 `Cow、beef::Cow、Box、 Cell、RefCell、OnceCell、LazyCell、LazyLock` 和 `Rc`. 因为基于它们的作用, 主要是对普通数据进行包装, 帮助我我们扩展高级功能

## Cow (clone on write, copy on write)

`Cow` 是 `clone on write` 或者 `copy on write` 的缩写。

这是一种优化内存和提高性能的技术,通常应用在资源共享的场景。

其基本思想是,当有多个调用者(`callers`)同时请求相同的资源时,都会共享同一份资源,直到有调用者试图修改资源内容时,系统才会真正复制一份副本出来给该调用者,而其他调用者仍然使用原来的资源。

`Rust` 中的 `String` 和 `Vec` 等类型就利用了 `Cow`。例如:

```rust
pub fn cow_exp_string() {
    let s1 = String::from("hello");
    println!("s1={}",  s1);
    let mut s2 = s1; // s1和s2共享同一份内存

    s2.push_str(" world"); // s2会进行写操作,于是系统复制一份新的内存给s2

    println!("s2={}", s2);
}
```

这样可以避免大量未修改的字符串、向量等的重复分配和复制,提高内存利用率和性能。

1. `cow` 的优点是:
    * 内存利用率高,只有进行写时才复制
    * 读取性能高,多个调用者共享同一资源

2. `cow` 缺点是:
    * 写时需要复制,有一定性能损失
    * 实现较复杂
    * 需要根据实际场景权衡使用。

综合来说,对于存在大量相同或相似资源的共享情况,使用 `cow` 可以带来显著性能提升。

标准库中`std::borrow::Cow` 类型是一个智能指针，提供了写时克隆（`clone-on-write`）的功能：它可以封装并提供对借用数据的不可变访问，当需要进行修改或获取所有权时，它可以惰性地克隆数据。

`Cow` 实现了 `Deref`，这意味着你可以直接在其封装的数据上调用不可变方法。如果需要进行改变，则 `to_mut` 将获取到一个对拥有的值的可变引用，必要时进行克隆。

下面的代码将 `origin` 字符串包装成一个 `cow`, 你可以把它 `borrowed` 成一个 `&str`,其实也可以直接在cow调用`&str`方法，因为`Cow`实现了`Deref`，可以自动解引用，比如直接调用`len`和`into`：

```rust
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
```

这里使用`to_mut`得到一个可变引用，一旦 s 有修改，它会从原始数据中 clone 一份，在克隆的数据上进行修改。

所以如果你想在某些数据上实现`copy-on-write/clone-on-write`的功能，可以考虑使用`std::borrow::Cow`

更进一步，`beef`库提供了一个更快，更紧凑的`Cow`类型,它的使用方法和标准库的`Cow`使用方法类似：

```rust
use beef;
use std::mem::size_of;

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
```

这个例子的上半部分演示了生成`beef::Cow`的三种方法`Cow::borrowed、Cow::from、Cow::owned`，标准库`Cow`也有这三个方法

它们的区别是：

* `borrowed`: 借用已有资源
* `from`: 从已有资源复制创建 Owned
* `owned`: 自己提供资源内容

这个例子下半部分对比了标准库`Cow`和`beef::Cow`以及更紧凑的`beef::lean::Cow`所占内存的大小。

可以看到对于数据是`str`类型的 `Cow`，现在的标准库的`Cow`占三个 `WORD`, 和 `beef::Cow` 相当,而进一步压缩的 `beef::lean::Cow` 只占了两个 `Word`。

`cow-utils`针对字符串的 `Cow` 做了优化，性能更好。

## Box 

`Box<T>`，通常简称为`box`，提供了在 `Rust` 中最简单的堆分配形式。`Box` 为这个分配提供了所有权，并在超出作用域时释放其内容。`Box` 还确保它们不会分配超过 `isize::MAX` 字节的内存。

使用很简单，下面的例子就是把值`val`从栈上移动到堆上：

```rust
pub fn box_exp_display() {
    let val: u8 = 5;
    let _boxed: Box<u8> = Box::new(val);
}
```

通过解引用把值从堆上移动到栈上：

```rust
pub fn box_heap_to_stack() {
    let boxed: Box<u8> = Box::new(5);
    let _val: u8 = *boxed;
}
```

如果我们要定义一个递归的数据结构，比如链表，下面的方式是不行的，因为 `List` 的大小不固定，我们不知道该分配给它多少内存：

```rust
pub fn box_auto_data_size() {
    /*
    // 因为 List 的大小是动态的,下面会报错
    #[derive(Debug)]
    enum List<T> {
        Cons(T, List<T>),
        Nil,
    }
    */
    #[derive(Debug)]
    enum List<T> {
        Cons(T, Box<List<T>>),
        Nil,
    }

    let list: List<i32> = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
    println!("{list:?}");
}
```

在 `Rust` 中，常常用 `瘦指针` 来指代 `&T` 类型的引用，其中 `T` 是某个类型的名称。这种指针只包含指向数据的地址，并不包含其他信息。

相比之下，`胖指针（fat pointer）`通常包含指向数据的地址以及额外的元数据，比如长度信息或者类型信息。例如，`&str` 类型的引用就是一个胖指针，因为它包含了字符串的长度信息

目前 `Rust` 还提供一个实验性的类型`ThinBox`, 它就是一个`瘦指针`，不管内部元素的类型是啥：

```rust
#![feature(thin_box)]
use std::boxed::ThinBox;

pub fn thin_box_example() {
    let five = ThinBox::new(5);
    let thin_slice = ThinBox::<[i32]>::new_unsize([1, 2, 3, 4]);
    use std::mem::{size_of, size_of_val};
    let size_of_ptr = size_of::<*const ()>();
    assert_eq!(size_of_ptr, size_of_val(&five));
    assert_eq!(size_of_ptr, size_of_val(&thin_slice));
}
```

## Cell、RefCell、OnceCell、LazyCell 和 LazyLock

`Cell`和`RefCell`是 `Rust` 中用于内部可变性(`interior mutability`)的两个重要类型。

`Cell`和`RefCell`都是可共享的可变容器。

可共享的可变容器的存在是为了以受控的方式允许可变性，即使存在别名引用。

`Cell`和 `RefCell`都允许在单线程环境下以这种方式进行。然而，无论是 `Cell`还是 `RefCell`都不是线程安全的（它们没有实现 `Sync`）。

### Cell

`Cell<T>`允许在不违反借用规则的前提下,修改其包含的值:

* `Cell`中的值不再拥有所有权,只能通过`get`和`set`方法访问。
* `set`方法可以在不获取可变引用的情况下修改`Cell`的值。
* 适用于简单的单值容器，如整数或字符。

下面这个例子创建了一个 `Cell`, 赋值给变量`x`,注意 `x` 是不可变的，但是我们能够通过`set`方法修改它的值，并且即使存在对 `x` 的引用 `y` 时也可以修改它的值：

```rust
use std::cell::Cell;

pub fn cell_exp() {
    let x = Cell::new(42);
    let y = &x;

    x.set(10); // 可以修改

    println!("y: {:?}", y.get()); // 输出 y: 10
}
```
