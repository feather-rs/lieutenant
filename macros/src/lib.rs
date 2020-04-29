use darling::ast::GenericParamExt;
use darling::FromMeta;
use proc_macro2::{Ident, Span};
use proc_macro_error::*;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, FnArg, ItemFn, Pat, PatType, Type};

#[derive(FromMeta)]
struct Args {
    usage: String,
}

struct Usage {
    arguments: Vec<Argument>,
    root_literal: String,
}

enum Argument {
    Parameter { name: String },
    OptionalParameter { name: String },
    Literal { value: String },
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn command(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    let args: Args = match Args::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => abort_call_site!("invalid parameters passed to #[command]: {}", e;
            help = "correct parameters: #[command(usage = \"/command <args...>\")]";
        ),
    };

    if let Some(first_generic) = input.sig.generics.params.iter().next() {
        let help = first_generic
            .as_type_param()
            .map(|type_param| format!("remove the parameter {}", type_param.ident));
        emit_error!(
            first_generic.span(), "command functions may not have generic parameters";

            help =? help;
        );
    }

    let usage = parse_usage(&args.usage);
    let parameters = collect_parameters(&usage, &input.sig.inputs.iter());

    let root_literal = &usage.root_literal;

    let mut builder_calls = vec![quote! {
        CommandBuilder::new(#root_literal)
    }];
    for argument in &usage.arguments {
        match argument {
            Argument::Parameter { .. } | Argument::OptionalParameter { .. } => {
                builder_calls.push(quote! { .arg() })
            }
            Argument::Literal { value } => builder_calls.push(quote! { .literal(#value) }),
        }
    }

    let mut params_tuple = vec![];
    let mut params_tuple_type = vec![];
    for parameter in parameters {
        let ty = &parameter.ty;
        let pat = &parameter.pat;
        params_tuple.push(quote! {
            #pat
        });
        params_tuple_type.push(quote! {
            #ty
        });
    }

    let command_ident = &input.sig.ident;
    let block = &input.block;
    let tokens = quote! {
        fn #command_ident() -> impl lieutenant::Command {
            #(#builder_calls)*
                .build(|(#(#params_tuple),*): (#(#params_tuple_type),*)| {
                    #block
                })
        }
    };
    tokens.into()
}

fn parse_usage(usage: &str) -> Usage {
    let mut arguments = vec![];
    let mut root_literal = None;

    for splitted in usage.split(" ") {
        let (first, middle) = splitted.split_at(1.min(splitted.len()));
        let (middle, last) = middle.split_at(middle.len() - 1);
        match (first, middle, last) {
            ("/", _, _) => root_literal = Some(splitted[1..].to_owned()),
            ("<", param, ">") => arguments.push(Argument::Parameter {
                name: param.to_owned(),
            }),
            ("[", param, "]") => arguments.push(Argument::OptionalParameter {
                name: param.to_owned(),
            }),
            (_, _, _) => arguments.push(Argument::Literal {
                value: splitted.to_owned(),
            }),
        }
    }

    let root_literal = match root_literal {
        Some(r) => r,
        None => abort_call_site!(
            "missing root command literal";

            help = "make sure your `usage` starts with the command name prefixed with a slash: `/command`"
        ),
    };

    Usage {
        arguments,
        root_literal,
    }
}

fn collect_parameters<'a>(
    usage: &Usage,
    inputs: &(impl Iterator<Item = &'a FnArg> + Clone),
) -> Vec<&'a PatType> {
    let mut parameters = vec![];
    for arg in &usage.arguments {
        match arg {
            Argument::Parameter { name } | Argument::OptionalParameter { name } => {
                collect_parameter(name, &mut parameters, arg, inputs);
            }
            Argument::Literal { .. } => (),
        }
    }

    parameters
}

fn collect_parameter<'a>(
    name: &str,
    parameters: &mut Vec<&'a PatType>,
    arg: &Argument,
    inputs: &(impl Iterator<Item = &'a FnArg> + Clone),
) {
    // check that there is a corresponding parameter to the function
    let arg_type = if let Some(arg_type) = find_corresponding_arg(name, inputs) {
        arg_type
    } else {
        emit_call_site_error!(
            "no corresponding function parameter for command parameter {}", name;

            help = "add a parameter to the function: `{}: <argument type>", name;
        );
        return;
    };
    validate_parameter(name, arg, arg_type);
    parameters.push(arg_type);
}

fn validate_parameter(name: &str, arg: &Argument, arg_type: &PatType) {
    // If not an optional parameter, ensure the type is not an option.
    // Otherwise, ensure it _is_ an Option.
    if let Argument::Parameter { .. } = arg {
        // not optional
        validate_argument_type(&arg_type.ty, name);
        if let Type::Path(path) = arg_type.ty.as_ref() {
            // verify that path is not an `Option`
            if path.path.is_ident(&Ident::new("Option", Span::call_site())) {
                emit_error!(
                    path.span(), "the parameter {} is defined as an `Option`, but the usage message indicates it is a required argument", name;

                    help = "change the usage instructions to make the argument optional: `<{}>`", name;
                );
            }
        };
    } else {
        // optional
    }
}

fn validate_argument_type(ty: &Type, name: &str) {
    match ty {
        Type::ImplTrait(span) => emit_error!(
            span.span(), "command function may not take `impl Trait`-style parameters";

            help = "change the type of the parameter {}", name;
        ),
        Type::Reference(reference) => {
            if reference.lifetime.clone().map(|l| l.ident.to_string()) != Some("static".to_owned())
            {
                emit_error!(
                    reference.span(), "command function may not take non-'static references as paramters";

                    hint = "use an owned value instead by removing the '&'";
                );
            }
        }
        _ => (),
    }
}

fn find_corresponding_arg<'a>(
    name: &str,
    args: &(impl Iterator<Item = &'a FnArg> + Clone),
) -> Option<&'a PatType> {
    args.clone()
        .find(|arg| {
            let ident = match arg {
                FnArg::Receiver(x) => {
                    emit_error!(x.span(), "command functions may not take `self` as a parameter";
                        help = "remove the `self` parameter";
                    );
                    return false;
                }
                FnArg::Typed(ty) => match ty.pat.as_ref() {
                    Pat::Ident(ident) => &ident.ident,
                    pat => {
                        emit_error!(pat.span(), "invalid command parameter pattern");
                        return false;
                    }
                },
            };

            possible_parameter_idents(name).contains(&ident.to_string())
        })
        .map(|arg| match arg {
            FnArg::Typed(ty) => ty,
            _ => unreachable!(),
        })
}

fn possible_parameter_idents(name: &str) -> Vec<String> {
    vec![name.to_owned(), format!("_{}", name)]
}
