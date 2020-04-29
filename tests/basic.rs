use lieutenant::{command, CommandDispatcher};

#[test]
fn basic_command() {
    #[command(usage = "/test <x>")]
    fn test(ctx: &mut i32, x: i32) {
        *ctx = x;
    };

    let dispatcher = CommandDispatcher::new().with(test);

    let mut x = 0;
    assert!(dispatcher.dispatch(&mut x, "test 27"));
    assert_eq!(x, 27);
}

#[test]
fn multiple_args() {
    struct State {
        x: i32,
        y: String,
    }

    #[command(usage = "/test14 <new_x> <new_y> extra_literal")]
    fn test14(state: &mut State, new_x: i32, new_y: String) {
        state.x = new_x;
        state.y = new_y;
    }

    let mut dispatcher = CommandDispatcher::new();
    dispatcher.register(test14).unwrap();

    let mut state = State {
        x: 690854,
        y: String::from("wrong"),
    };
    assert!(dispatcher.dispatch(&mut state, "test14 66 string extra_literal"));

    assert_eq!(state.x, 66);
    assert_eq!(state.y.as_str(), "string");
}

#[test]
fn multiple_commands() {
    struct State {
        x: i32,
        y: String,
    }

    #[command(usage = "/cmd1 <new_x> extra_lit")]
    fn cmd1(state: &mut State, new_x: i32) {
        state.x = new_x;
    }

    #[command(usage = "/cmd2 <new_y>")]
    fn cmd2(state: &mut State, new_y: String) {
        state.y = new_y;
    }

    let dispatcher = CommandDispatcher::new().with(cmd1).with(cmd2);

    let mut state = State {
        x: 32,
        y: String::from("incorrect"),
    };

    assert!(!dispatcher.dispatch(&mut state, "cmd1 10")); // misssing extra_lit
    assert!(dispatcher.dispatch(&mut state, "cmd1 10 extra_lit"));
    assert_eq!(state.x, 10);

    assert!(!dispatcher.dispatch(&mut state, "invalid command 22"));

    assert!(dispatcher.dispatch(&mut state, "cmd2 new_string"));
    assert_eq!(state.y.as_str(), "new_string");
}

#[test]
fn command_macro() {
    struct State {
        x: i32,
        player: String,
    }

    #[command(usage = "/test <x>")]
    fn test(state: &mut State, x: i32) {
        state.x = x;
    }

    #[command(usage = "/foo <player>")]
    fn foo_a_player(state: &mut State, player: String) {
        state.player.push_str(&player);
    }

    #[command(usage = "/bar <player> <x>")]
    fn foo_a_player_then_bar_an_x(state: &mut State, x: i32, player: String) {
        state.player.push_str(&player);
        state.x = x + 1;
    }

    let dispatcher = CommandDispatcher::new()
        .with(test)
        .with(foo_a_player)
        .with(foo_a_player_then_bar_an_x);

    let mut state = State {
        x: 0,
        player: String::new(),
    };
    assert!(!dispatcher.dispatch(&mut state, "false command"));

    assert!(dispatcher.dispatch(&mut state, "test 25"));
    assert_eq!(state.x, 25);

    assert!(dispatcher.dispatch(&mut state, "foo twenty-six"));
    assert_eq!(state.player.as_str(), "twenty-six");

    assert!(!dispatcher.dispatch(&mut state, "test"));
    assert!(!dispatcher.dispatch(&mut state, "test not-a-number"));

    assert!(!dispatcher.dispatch(&mut state, "bar"));
    assert!(!dispatcher.dispatch(&mut state, "bar player"));
    assert!(!dispatcher.dispatch(&mut state, "bar player four"));
    assert!(dispatcher.dispatch(&mut state, "bar PLAYER 28"));

    assert_eq!(state.x, 29);
    assert_eq!(state.player.as_str(), "twenty-sixPLAYER");
}

#[test]
fn help_command() {
    use std::borrow::Cow;
    use std::rc::Rc;
    struct State {
        pub dispatcher: Rc<CommandDispatcher<Self>>,
        pub usages: Vec<Cow<'static, str>>,
        pub descriptions: Vec<Cow<'static, str>>,
    }

    let mut dispatcher = CommandDispatcher::new();

    #[command(
        usage = "/help <page>",
        description = "Shows the descriptions and usages of all commands."
    )]
    fn help(state: &mut State, page: u32) {
        state.usages = state
            .dispatcher
            .command_meta()
            .skip(page as usize * 10)
            .take(10)
            .map(|meta| meta.usage.clone())
            .collect();
        state.descriptions = state
            .dispatcher
            .command_meta()
            .skip(page as usize * 10)
            .take(10)
            .filter_map(|meta| meta.description.clone())
            .collect();
    }

    dispatcher.register(help).unwrap();

    let dispatcher = Rc::new(dispatcher);

    let mut ctx = State {
        dispatcher: Rc::clone(&dispatcher),
        usages: vec![],
        descriptions: vec![],
    };

    assert!(dispatcher.dispatch(&mut ctx, "help 0"));
    assert_eq!(ctx.usages, vec!["/help <page>"]);
    assert_eq!(ctx.descriptions, vec!["Shows the descriptions and usages of all commands."]);

    assert!(dispatcher.dispatch(&mut ctx, "help 1"));
    assert!(ctx.usages.is_empty());
    assert!(ctx.descriptions.is_empty());
}
