

pub fn box_stack_to_heap() {
    let val: u8 = 5;
    let _boxed: Box<u8> = Box::new(val);
}

pub fn box_heap_to_stack() {
    let boxed: Box<u8> = Box::new(5);
    let _val: u8 = *boxed;
}

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


