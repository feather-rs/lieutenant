use lieutenant::{command, CommandDispatcher, Context};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
enum Error {
    #[error("{0}")]
    Custom(String),
}

#[test]
fn basic_command() {
    #[derive(Debug, PartialEq, Eq)]
    struct State(i32);

    impl Context for State {
        type Error = Error;
        type Ok = ();
    }

    #[command(usage = "test <x>")]
    fn test(ctx: &mut State, x: i32) -> Result<(), Error> {
        *ctx = State(x);
        Ok(())
    };

    let dispatcher = CommandDispatcher::default().with(test);

    let mut nodes = Vec::new();
    let mut errors = Vec::new();

    let mut x = State(0);
    assert!(
        smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut x, "test 27")).is_ok()
    );
    assert_eq!(x, State(27));
}

#[test]
fn basic_command_parralel() {
    use futures::future;
    use futures::join;
    use smol::Timer;
    use std::thread;
    use std::time::{Duration, Instant};

    for _ in 0..2 {
        // A pending future is one that simply yields forever.
        thread::spawn(|| smol::run(future::pending::<()>()));
    }

    #[derive(Debug, PartialEq, Eq)]
    struct State(i32);

    impl Context for State {
        type Error = Error;
        type Ok = ();
    }

    #[command(usage = "test <x>")]
    fn test(ctx: &mut State, x: i32) -> Result<(), Error> {
        *ctx = State(x);
        Timer::after(Duration::from_secs(1)).await;

        Ok(())
    };

    let dispatcher = CommandDispatcher::default().with(test);

    let mut nodes_a = Vec::new();
    let mut errors_a = Vec::new();

    let mut nodes_b = Vec::new();
    let mut errors_b = Vec::new();

    let mut a = State(0);
    let mut b = State(0);

    let call_a = dispatcher.dispatch(&mut nodes_a, &mut errors_a, &mut a, "test 27");
    let call_b = dispatcher.dispatch(&mut nodes_b, &mut errors_b, &mut b, "test 27");

    let now = Instant::now();

    assert_eq!(
        smol::block_on(async { join!(call_a, call_b) }),
        (Ok(()), Ok(()))
    );

    assert_eq!(now.elapsed().as_secs(), 1);
}

#[test]
fn error_handling() {
    #[derive(Debug, PartialEq, Eq)]
    struct State;

    impl Context for State {
        type Error = Error;
        type Ok = ();
    }

    #[command(usage = "test <x>")]
    fn test(ctx: &mut State, x: i32) -> Result<(), Error> {
        if x == 0 {
            Ok(())
        } else {
            Err(Error::Custom("Not zero".into()))
        }
    };

    let dispatcher = CommandDispatcher::default().with(test);

    let mut nodes = Vec::new();
    let mut errors = Vec::new();

    assert_eq!(
        smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, "test 0")),
        Ok(())
    );
    assert_eq!(
        smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut State, "test 5")),
        Err(&vec![Error::Custom("Not zero".into())])
    );
}

#[test]
fn multiple_args() {
    struct State {
        x: i32,
        y: String,
    }

    impl Context for State {
        type Error = Error;
        type Ok = ();
    }

    #[command(usage = "test14 <new_x> <new_y> extra_literal")]
    fn test14(state: &mut State, new_x: i32, new_y: String) -> Result<(), Error> {
        state.x = new_x;
        state.y = new_y;
        Ok(())
    }

    let mut dispatcher = CommandDispatcher::default();
    dispatcher.register(test14).unwrap();

    let mut nodes = Vec::new();
    let mut errors = Vec::new();

    let mut state = State {
        x: 690854,
        y: String::from("wrong"),
    };
    assert!(smol::block_on(dispatcher.dispatch(
        &mut nodes,
        &mut errors,
        &mut state,
        "test14 66 string extra_literal"
    ))
    .is_ok());

    assert_eq!(state.x, 66);
    assert_eq!(state.y.as_str(), "string");
}

#[test]
fn multiple_commands() {
    struct State {
        x: i32,
        y: String,
    }

    impl Context for State {
        type Error = Error;
        type Ok = ();
    }

    #[command(usage = "cmd1 <new_x> extra_lit")]
    fn cmd1(state: &mut State, new_x: i32) -> Result<(), Error> {
        state.x = new_x;
        Ok(())
    }

    #[command(usage = "cmd2 <new_y>")]
    fn cmd2(state: &mut State, new_y: String) -> Result<(), Error> {
        state.y = new_y;
        Ok(())
    }

    let dispatcher = CommandDispatcher::default().with(cmd1).with(cmd2);

    let mut nodes = Vec::new();
    let mut errors = Vec::new();

    let mut state = State {
        x: 32,
        y: String::from("incorrect"),
    };

    assert!(
        smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut state, "cmd1 10"))
            .is_err()
    ); // misssing extra_lit

    assert!(smol::block_on(dispatcher.dispatch(
        &mut nodes,
        &mut errors,
        &mut state,
        "cmd1 10 extra_lit"
    ))
    .is_ok());
    assert_eq!(state.x, 10);

    assert!(smol::block_on(dispatcher.dispatch(
        &mut nodes,
        &mut errors,
        &mut state,
        "invalid command 22"
    ))
    .is_err());

    assert!(smol::block_on(dispatcher.dispatch(
        &mut nodes,
        &mut errors,
        &mut state,
        "cmd2 new_string"
    ))
    .is_ok());
    assert_eq!(state.y.as_str(), "new_string");
}

#[test]
fn command_macro() {
    struct State {
        x: i32,
        player: String,
    }

    impl Context for State {
        type Error = Error;
        type Ok = ();
    }

    #[command(usage = "test <x>")]
    fn test(state: &mut State, x: i32) -> Result<(), Error> {
        state.x = x;
        Ok(())
    }

    #[command(usage = "foo <player>")]
    fn foo_a_player(state: &mut State, player: String) -> Result<(), Error> {
        state.player.push_str(&player);
        Ok(())
    }

    #[command(usage = "bar <player> <x>")]
    fn foo_a_player_then_bar_an_x(state: &mut State, x: i32, player: String) -> Result<(), Error> {
        state.player.push_str(&player);
        state.x = x + 1;
        Ok(())
    }

    let dispatcher = CommandDispatcher::default()
        .with(test)
        .with(foo_a_player)
        .with(foo_a_player_then_bar_an_x);

    let mut errors = Vec::new();
    let mut nodes = Vec::new();

    let mut state = State {
        x: 0,
        player: String::new(),
    };
    assert!(smol::block_on(dispatcher.dispatch(
        &mut nodes,
        &mut errors,
        &mut state,
        "false command"
    ))
    .is_err());

    assert!(
        smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut state, "test 25")).is_ok()
    );
    assert_eq!(state.x, 25);

    assert!(smol::block_on(dispatcher.dispatch(
        &mut nodes,
        &mut errors,
        &mut state,
        "foo twenty-six"
    ))
    .is_ok());
    assert_eq!(state.player.as_str(), "twenty-six");

    assert!(
        smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut state, "test")).is_err()
    );

    assert!(smol::block_on(dispatcher.dispatch(
        &mut nodes,
        &mut errors,
        &mut state,
        "test not-a-number"
    ))
    .is_err());

    assert!(
        smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut state, "bar")).is_err()
    );

    assert!(
        smol::block_on(dispatcher.dispatch(&mut nodes, &mut errors, &mut state, "bar player"))
            .is_err()
    );

    assert!(smol::block_on(dispatcher.dispatch(
        &mut nodes,
        &mut errors,
        &mut state,
        "bar player four"
    ))
    .is_err());

    assert!(smol::block_on(dispatcher.dispatch(
        &mut nodes,
        &mut errors,
        &mut state,
        "bar PLAYER 28"
    ))
    .is_ok());

    assert_eq!(state.x, 29);
    assert_eq!(state.player.as_str(), "twenty-sixPLAYER");
}

#[test]
fn help_command() {
    // use std::borrow::Cow;
    // use std::rc::Rc;
    // struct State {
    //     dispatcher: Rc<CommandDispatcher<Self>>,
    //     usages: Vec<Cow<'static, str>>,
    //     descriptions: Vec<Cow<'static, str>>,
    // }

    // impl Context for State {
    //     type Error = Error;
    //     type Ok = ();
    // }

    // let mut dispatcher = CommandDispatcher::default();

    // #[command(
    //     usage = "help <page>",
    //     description = "Shows the descriptions and usages of all commands."
    // )]
    // fn help(state: &mut State, page: u32) -> Result<(), Error> {
    //     state.usages = state
    //         .dispatcher
    //         .commands()
    //         .skip(page as usize * 10)
    //         .take(10)
    //         .map(|meta| meta.arguments.iter().map(|_| "").collect())
    //         .collect();
    //     state.descriptions = state
    //         .dispatcher
    //         .commands()
    //         .skip(page as usize * 10)
    //         .take(10)
    //         .filter_map(|meta| meta.description.clone())
    //         .collect();
    //     Ok(())
    // }

    // dispatcher.register(help).unwrap();

    // let dispatcher = Rc::new(dispatcher);

    // let mut nodes = Vec::new();
    // let mut errors = Vec::new();

    // let mut ctx = State {
    //     dispatcher: Rc::clone(&dispatcher),
    //     usages: vec![],
    //     descriptions: vec![],
    // };

    // assert!(dispatcher
    //     .dispatch(&mut nodes, &mut errors, &mut ctx, "help 0")
    //     .is_ok());
    // assert_eq!(ctx.usages, vec!["/help <page>"]);
    // assert_eq!(
    //     ctx.descriptions,
    //     vec!["Shows the descriptions and usages of all commands."]
    // );

    // assert!(dispatcher
    //     .dispatch(&mut nodes, &mut errors, &mut ctx, "help 1")
    //     .is_ok());
    // assert!(ctx.usages.is_empty());
    // assert!(ctx.descriptions.is_empty());
}
