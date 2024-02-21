use std::{cell::{RefCell, RefMut}, collections::HashMap, rc::Rc};

pub fn rc_exp() {
    let data = Rc::new(42);

    let _reference1 = Rc::clone(&data);
    let _reference2 = Rc::clone(&data);
    // data 的引用计数现在为 3
    // 当 reference1 和 reference2 被丢弃时，引用计数减少
}

pub fn rc_refcell_example() {
    let shared_map: Rc<RefCell<_>> = Rc::new(RefCell::new(HashMap::new()));
    {
        let mut map: RefMut<_> = shared_map.borrow_mut();
        map.insert("africa", 92388);
        map.insert("kyoto", 11837);
        map.insert("piccadilly", 11826);
        map.insert("marbles", 38);
    }

    let total: i32 = shared_map.borrow().values().sum();
    println!("{total}");
}
