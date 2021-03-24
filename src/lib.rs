pub mod argument;
pub mod command;
mod generic;
pub mod parser;
pub mod regex;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
mod tests {

    use crate::command::builder::{literal, CommandBuilder};
    use crate::command::Command;
    // //use crate::command::CommandBuilder;
    // use crate::{
    //     //command::{literal, Command},
    //     AddToDispatcher, Dispatcher,
    // };

    #[test]
    fn simple() {
        // (Gamestate, Extract) -> Res    Extract -> (Gamestate -> Res)
        let command = literal("/").space().arg::<u32>();
        let x = command.on_call(|x| {
            move |game_state, _foo| {
                println!("hi {} the gamestate was {}", x, game_state);
                42
            }
        });

        let _r = x.call((0, "test"), "/ 100 ").unwrap();
    }
}
