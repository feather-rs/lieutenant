//! Test cases for https://github.com/feather-rs/lieutenant/issues/14.

use lieutenant::{command, Context};

struct Ctx;
impl Context for Ctx {
    type Error = anyhow::Error;
    type Ok = (); 
}

#[command(usage = "a b c <arg>")]
fn cmd(ctx: &mut Ctx, _arg: u32) -> anyhow::Result<()> {
    Ok(())
}
