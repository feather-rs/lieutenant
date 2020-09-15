use lieutenant::{
    dispatcher::ParserKind,
    dispatcher::{InputConsumer, Node, NodeKind},
    CommandDispatcher,
};

#[test]
fn msg_command() {
    let literal_argument = Node {
        consumer: InputConsumer::SingleWord,
        kind: NodeKind::Literal("msg".to_owned()),
        execute: None,
        children: Vec::new(),
    };
    let mut dispatcher = CommandDispatcher::new();
    let literal = dispatcher.add_node(None, literal_argument);

    let player_argument = Node {
        consumer: InputConsumer::SingleWord,
        kind: NodeKind::Argument {
            parser: ParserKind::String,
        },
        execute: None,
        children: Vec::new(),
    };
    let player = dispatcher.add_node(Some(literal), player_argument);

    let message_argument = Node {
        consumer: InputConsumer::Greedy,
        kind: NodeKind::Argument {
            parser: ParserKind::String,
        },
        execute: Some(msg),
        children: Vec::new(),
    };
    dispatcher.add_node(Some(player), message_argument);

    fn msg(ctx: &mut String, input: &str) -> lieutenant::Result<()> {
        ctx.push_str(input);
        Ok(())
    }

    let command =
        "msg target_player this is a long message \" with quotes \" but this is a greedy string...";
    let mut context = String::new();
    dispatcher.execute(command, &mut context).unwrap();

    assert_eq!(context, command);
}
