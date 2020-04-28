use lieutenant::{CommandBuilder, CommandDispatcher};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// Using Rc until context inputs are supported

#[test]
fn basic_command() {
    let x = Rc::new(Cell::new(10));
    let x2 = Rc::clone(&x);

    let command = CommandBuilder::new("test")
        .arg::<i32>()
        .build(move |input| {
            x2.set(input);
        });

    let mut dispatcher = CommandDispatcher::new();
    dispatcher.register(command).unwrap();

    assert!(dispatcher.dispatch("test 27"));
    assert_eq!(x.get(), 27);
}

#[test]
fn multiple_args() {
    let x = Rc::new(Cell::new(42));
    let y = Rc::new(RefCell::new("wrong value".to_owned()));

    let x2 = Rc::clone(&x);
    let y2 = Rc::clone(&y);

    let command = CommandBuilder::new("test14")
        .arg::<i32>()
        .arg::<String>()
        .literal("extra_literal")
        .build(move |(new_x, new_y)| {
            x.set(new_x);
            *y.borrow_mut() = new_y;
        });

    let mut dispatcher = CommandDispatcher::new();
    dispatcher.register(command).unwrap();

    assert!(dispatcher.dispatch("test14 66 string extra_literal"));

    assert_eq!(x2.get(), 66);
    assert_eq!(y2.borrow().as_str(), "string");
}

#[test]
fn multiple_commands() {
    let x = Rc::new(Cell::new(42));
    let y = Rc::new(RefCell::new("wrong value".to_owned()));

    let x2 = Rc::clone(&x);
    let y2 = Rc::clone(&y);

    let cmd1 = CommandBuilder::new("cmd1")
        .arg::<i32>()
        .literal("extra_lit")
        .build(move |new_x| x.set(new_x));

    let cmd2 = CommandBuilder::new("cmd2")
        .arg::<String>()
        .build(move |new_y| *y.borrow_mut() = new_y);

    let mut dispatcher = CommandDispatcher::new();
    dispatcher.register(cmd1).unwrap();
    dispatcher.register(cmd2).unwrap();

    assert!(dispatcher.dispatch("cmd1 10 extra_lit"));
    assert_eq!(x2.get(), 10);

    assert!(!dispatcher.dispatch("invalid command 22"));

    assert!(dispatcher.dispatch("cmd2 new_string"));
    assert_eq!(y2.borrow().as_str(), "new_string");
}
