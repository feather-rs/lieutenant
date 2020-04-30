use criterion::{criterion_group, criterion_main, Criterion};
use lieutenant::{command, CommandDispatcher, Context};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("{0}")]
    Custom(String),
}

fn criterion_benchmark(c: &mut Criterion) {
    struct State;
    impl Context for State {
        type Error = Error;
        type Ok = ();
    }
    #[command(usage = "command_1")]
    fn command_1(_: &mut State) -> Result<(), Error> {
        // thread::sleep(time::Duration::from_millis(1));
        Ok(())
    }

    let mut dispatcher = CommandDispatcher::default();
    dispatcher.register(command_1).unwrap();

    c.bench_function("dispatcher with a single command being dispatched", |b| b.iter(|| dispatcher.dispatch(&mut State, "command_1")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);