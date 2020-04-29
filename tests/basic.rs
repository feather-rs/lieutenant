use lieutenant::{command, CommandBuilder, CommandDispatcher};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

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

static X: AtomicI32 = AtomicI32::new(0);
static PLAYER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

#[command(usage = "/test <x>")]
fn test(x: i32) {
    X.store(x, Ordering::SeqCst);
}

#[command(usage = "/foo <player>")]
fn foo_a_player(player: String) {
    PLAYER.lock().unwrap().push_str(&player);
}

#[command(usage = "/bar <player> <x>")]
fn foo_a_player_then_bar_an_x(player: String, x: i32) {
    X.store(x + 1, Ordering::SeqCst);
    PLAYER.lock().unwrap().push_str(&player);
}

#[test]
fn command_macro() {
    let dispatcher = CommandDispatcher::new()
        .with(test())
        .with(foo_a_player())
        .with(foo_a_player_then_bar_an_x());

    assert!(!dispatcher.dispatch("false command"));

    assert!(dispatcher.dispatch("test 25"));
    assert_eq!(X.load(Ordering::SeqCst), 25);

    assert!(dispatcher.dispatch("foo twenty-six"));
    assert_eq!(PLAYER.lock().unwrap().as_str(), "twenty-six");

    assert!(!dispatcher.dispatch("test"));
    assert!(!dispatcher.dispatch("test not-a-number"));

    assert!(!dispatcher.dispatch("bar"));
    assert!(!dispatcher.dispatch("bar player"));
    assert!(!dispatcher.dispatch("bar player four"));
    assert!(dispatcher.dispatch("bar PLAYER 28"));

    assert_eq!(X.load(Ordering::SeqCst), 29);
    assert_eq!(PLAYER.lock().unwrap().as_str(), "twenty-sixPLAYER");
}
