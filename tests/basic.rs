use lieutenant::{command, provider, CommandDispatcher, Context, Provideable};
use std::convert::Infallible;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
enum Error {
    #[error("{0}")]
    Custom(String),
    #[error("failed to parse int")]
    ParseInt,
}

impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Self {
        Error::ParseInt
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Self {
        panic!("Cannot fail")
    }
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

    let mut x = State(0);
    assert!(dispatcher.dispatch(&mut x, "test 27").is_ok());
    assert_eq!(x, State(27));
}

/*#[test]
fn basic_command_parallel() {
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
    async fn test(ctx: &mut State, x: i32) -> Result<(), Error> {
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

    assert_eq!(smol::run(async { join!(call_a, call_b) }), (Ok(()), Ok(())));

    assert_eq!(now.elapsed().as_secs(), 1);
}*/

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

    assert_eq!(dispatcher.dispatch(&mut State, "test 0"), Ok(()));
    assert_eq!(
        dispatcher.dispatch(&mut State, "test 5"),
        Err(vec![Error::Custom("Not zero".into())])
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

    let mut state = State {
        x: 690_854,
        y: String::from("wrong"),
    };
    assert!(dispatcher
        .dispatch(&mut state, "test14 66 string extra_literal")
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

    let mut state = State {
        x: 32,
        y: String::from("incorrect"),
    };

    assert!(dispatcher.dispatch(&mut state, "cmd1 10").is_err()); // misssing extra_lit

    assert!(dispatcher.dispatch(&mut state, "cmd1 10 extra_lit").is_ok());
    assert_eq!(state.x, 10);

    assert!(dispatcher
        .dispatch(&mut state, "invalid command 22")
        .is_err());

    assert!(dispatcher.dispatch(&mut state, "cmd2 new_string").is_ok());
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

    let mut state = State {
        x: 0,
        player: String::new(),
    };
    assert!(dispatcher.dispatch(&mut state, "false command").is_err());

    assert!(dispatcher.dispatch(&mut state, "test 25").is_ok());
    assert_eq!(state.x, 25);

    assert!(dispatcher.dispatch(&mut state, "foo twenty-six").is_ok());
    assert_eq!(state.player.as_str(), "twenty-six");

    assert!(dispatcher.dispatch(&mut state, "test").is_err());

    assert!(dispatcher
        .dispatch(&mut state, "test not-a-number")
        .is_err());

    assert!(dispatcher.dispatch(&mut state, "bar").is_err());

    assert!(dispatcher.dispatch(&mut state, "bar player").is_err());

    assert!(dispatcher.dispatch(&mut state, "bar player four").is_err());

    assert!(dispatcher.dispatch(&mut state, "bar PLAYER 28").is_ok());

    assert_eq!(state.x, 29);
    assert_eq!(state.player.as_str(), "twenty-sixPLAYER");
}

#[test]
fn aliasing() {
    struct State {
        x: u32,
    }

    impl Context for State {
        type Error = Error;
        type Ok = ();
    }
    #[command(usage = "test|t <x> lit2|lit3")]
    fn command(state: &mut State, x: u32) -> Result<(), Error> {
        state.x = x;
        Ok(())
    }

    let dispatcher = CommandDispatcher::new().with(command);

    let mut state = State { x: 0 };

    assert!(dispatcher.dispatch(&mut state, "test 10 lit2").is_ok());
    assert_eq!(state.x, 10);

    assert!(dispatcher.dispatch(&mut state, "t 15 lit3").is_ok());
    assert_eq!(state.x, 15);

    for wrong in ["test 1 lit", "test 1", "t 2", "t 2 lit", "t string lit2"]
        .iter()
        .copied()
    {
        assert!(dispatcher.dispatch(&mut state, wrong).is_err());
    }
}

#[test]
fn providers() {
    struct State {
        x: u32,
    }

    impl Context for State {
        type Error = Infallible;
        type Ok = ();
    }

    struct Provided(u32);
    impl Provideable<State> for Provided {
        type Provider = provide_u32;
    }

    #[provider]
    fn provide_u32(ctx: &State) -> Provided {
        Provided(ctx.x)
    }

    #[command(usage = "test")]
    fn cmd(ctx: &mut State, provided: Provided) -> Result<(), Infallible> {
        ctx.x += provided.0;
        Ok(())
    }

    let mut state = State { x: 15 };
    let dispatcher = CommandDispatcher::new().with(cmd);

    assert!(dispatcher.dispatch(&mut state, "test").is_ok());
    assert_eq!(state.x, 30);
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
