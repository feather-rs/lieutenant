use criterion::{criterion_group, criterion_main, Criterion};
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
    fn command_1(_: &mut State) -> Result<(), Error> {
        // thread::sleep(time::Duration::from_millis(1));
        Ok(())
    }

    let mut dispatcher = CommandDispatcher::default();
    dispatcher.register(command_1).unwrap();

    let mut nodes = Vec::new();
    let mut errors = Vec::new();

    c.bench_function("dispatcher with a single command being dispatched", |b| {
        b.iter(|| {
            assert!(smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, "command")).is_ok());
        })
    });
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

    c.bench_function("dispatcher with a single command being dispatched", |b| {
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
criterion_group!(multiple_commands_bench, multiple_commands);
criterion_main!(single_command_bench, multiple_commands_bench);
