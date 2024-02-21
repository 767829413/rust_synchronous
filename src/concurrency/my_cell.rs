use std::borrow::Borrow;
use std::cell::Cell;
use std::cell::OnceCell;
use std::cell::RefCell;
use std::ops::Deref;



pub fn cell_exp() {
    let x = Cell::new(42);
    let y = &x;

    x.set(10); // 可以修改

    println!("y: {:?}", y.get()); // 输出 y: 10
}

pub fn ref_cell_exp() {
    let x = RefCell::new(42);

    {
        let y = x.borrow();
        // 在这个作用域内，只能获得不可变引用
        println!("y: {:?}", *y.borrow());
    }

    {
        let mut z = x.borrow_mut();
        // 在这个作用域内，可以获得可变引用
        *z = 10;
    }
    println!("x: {:?}", x.borrow().deref());
}

pub fn once_cell_exp() {
    let cell = OnceCell::new();
    assert!(cell.get().is_none()); // true

    let value: &String = cell.get_or_init(|| "Hello, World!".to_string());
    println!("left: {} right: {}", value, "Hello, World!");
    assert!(cell.get().is_some());
}
