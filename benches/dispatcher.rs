use criterion::{criterion_group, criterion_main, Criterion};
use lieutenant::{command, CommandDispatcher};

fn criterion_benchmark(c: &mut Criterion) {
    struct State;
    #[command(usage = "/command_1")]
    fn command_1(_: &mut State) {
        // thread::sleep(time::Duration::from_millis(1));
    }

    let mut dispatcher = CommandDispatcher::new();
    dispatcher.register(command_1).unwrap();

    c.bench_function("dispatcher with a single command being dispatched", |b| b.iter(|| dispatcher.dispatch(&mut State, "command_1")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);