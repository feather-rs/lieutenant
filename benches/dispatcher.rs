use criterion::{criterion_group, criterion_main, Criterion, black_box};
use lieutenant::{command, CommandDispatcher, Context};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {}

fn single_command(c: &mut Criterion) {
    struct State;
    impl Context for State {
        type Error = Error;
        type Ok = ();
    }
    #[command(usage = "command")]
    fn command(_: &mut State) -> Result<(), Error> {
        // thread::sleep(time::Duration::from_millis(1));
        Ok(())
    }

    let mut dispatcher = CommandDispatcher::default();
    dispatcher.register(command).unwrap();

    let mut nodes = Vec::new();
    let mut errors = Vec::new();

    c.bench_function("dispatch single command", |b| {
        b.iter(|| {
            assert!(smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, black_box("command"))).is_ok());
        })
    });
}

fn single_command_prallel(_c: &mut Criterion) {
    // use std::thread;
    // use thread_local::ThreadLocal;
    // use futures::future;
    // use std::cell::RefCell;
    // use std::sync::Arc;
    // use std::time::Duration;
    // use smol::Task;

    // for _ in 0..2 {
    //     // A pending future is one that simply yields forever.
    //     thread::spawn(|| smol::run(future::pending::<()>()));
    // }

    // #[derive(Clone, Copy)]
    // struct State;
    // impl Context for State {
    //     type Error = Error;
    //     type Ok = ();
    // }
    // #[command(usage = "command")]
    // fn command_1(_: &mut State) -> Result<(), Error> {
    //     smol::Timer::after(Duration::from_secs(1)).await;
    //     Ok(())
    // }

    // let mut dispatcher = CommandDispatcher::default();
    // dispatcher.register(command_1).unwrap();
    // let dispatcher = Arc::new(dispatcher);

    // let nodes = Arc::new(ThreadLocal::new());
    // let errors = Arc::new(ThreadLocal::new());

    // c.bench_function("paralel dispatching with a single command", |b| {
    //     b.iter(|| {
    //         let nodes = Arc::clone(&nodes);
    //         let errors = Arc::clone(&errors);
    //         let dispatcher = Arc::clone(&dispatcher);

    //         Task::spawn(async move {
    //             let nodes: &RefCell<Vec<_>> = nodes.get_or_default();
    //             let errors: &RefCell<Vec<_>> = errors.get_or_default();
    //             dispatcher.dispatch(&mut *nodes.borrow_mut(), &mut *errors.borrow_mut(), &mut State, "command").await;
    //         }).detach();
    //     })
    // });
}

fn multiple_commands(c: &mut Criterion) {
    struct State;
    impl Context for State {
        type Error = Error;
        type Ok = ();
    }
    #[command(usage = "command")]
    fn command_1(_state: &mut State) -> Result<(), Error> {
        // thread::sleep(time::Duration::from_millis(1));
        Ok(())
    }

    #[command(usage = "command <a>")]
    fn command_2(_state: &mut State, _a: i32) -> Result<(), Error> {
        // thread::sleep(time::Duration::from_millis(1));
        Ok(())
    }

    #[command(usage = "command <a> <b>")]
    fn command_3(_state: &mut State, _a: i32, _b: String) -> Result<(), Error> {
        // thread::sleep(time::Duration::from_millis(1));
        Ok(())
    }

    #[command(usage = "command <a> <b>")]
    fn command_4(_state: &mut State, _a: String, _b: String) -> Result<(), Error> {
        // thread::sleep(time::Duration::from_millis(1));
        Ok(())
    }

    #[command(usage = "command <a> <b> <c>")]
    fn command_5(_state: &mut State, _a: i32, _b: i32, _c: i32) -> Result<(), Error> {
        // thread::sleep(time::Duration::from_millis(1));
        Ok(())
    }

    let dispatcher = CommandDispatcher::default()
        .with(command_1)
        .with(command_2)
        .with(command_3)
        .with(command_4)
        .with(command_5)
        ;

    let mut nodes = Vec::new();
    let mut errors = Vec::new();

    c.bench_function("dispatch multiple commands", |b| {
        b.iter(|| {
            assert!(smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, "command")).is_ok());
            assert!(smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, "command 4")).is_ok());
            assert!(smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, "command 4 hello")).is_ok());
            assert!(smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, "command hello hello")).is_ok());
            assert!(smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, "command 4 4 4")).is_ok());
            assert!(smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, "command a a a")).is_err());
        })
    });
}

criterion_group!(single_command_bench, single_command);
criterion_group!(single_command_parallel_bench, single_command_prallel);
criterion_group!(multiple_commands_bench, multiple_commands);

criterion_main!(single_command_bench, single_command_parallel_bench, multiple_commands_bench);

