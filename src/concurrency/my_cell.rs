use std::cell::Cell;

pub fn cell_exp() {
    let x = Cell::new(42);
    let y = &x;

    x.set(10); // 可以修改

    println!("y: {:?}", y.get()); // 输出 y: 10
}
