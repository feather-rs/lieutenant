//! Test cases for https://github.com/feather-rs/lieutenant/issues/14.

use lieutenant::{command, CommandDispatcher, Context};

struct Ctx;
impl Context for Ctx {
    type Error = anyhow::Error;
    type Ok = ();
}

#[command(usage = "a b c <arg>")]
fn cmd(ctx: &mut Ctx, arg: u32) -> anyhow::Result<()> {
    assert_eq!(arg, 10);
    Ok(())
}

#[test]
fn parses() {
    let dispatcher = CommandDispatcher::new().with(cmd);

    dispatcher.dispatch(&mut Ctx, "a b c 10").unwrap();
}
