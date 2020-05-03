use proc_macro_error::proc_macro_error;

mod command;
mod provider;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn command(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    command::command(args, input)
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn provider(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    provider::provider(args, input)
}
