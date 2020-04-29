use darling::ast::GenericParamExt;
use darling::FromMeta;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::*;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, Block, FnArg, ItemFn, Pat, PatType, Type};

#[derive(Debug, FromMeta)]
struct Args {
    usage: String,
    #[darling(default)]
    description: Option<String>
}

#[derive(Debug)]
struct Usage {
    arguments: Vec<Argument>,
}

#[derive(Debug)]
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

    let ctx_type = detect_context_type(&parameters, input.sig.inputs.iter().next());

    let command_ident = &input.sig.ident;

    let impl_header = if let Some((ctx_type, _)) = ctx_type {
        quote! {
            impl lieutenant::Command<#ctx_type> for #command_ident
        }
    } else {
        quote! {
            impl <C> lieutenant::Command<C> for #command_ident
        }
    };

    let ctx_actual_type = if let Some((ty, _)) = ctx_type {
        quote! { #ty }
    } else {
        quote! { C }
    };

    let command_spec = generate_command_spec(&usage, args.description, &parameters, ctx_type, &input.block);
    let visibility = &input.vis;

    let tokens = quote! {
        #[allow(non_camel_case_types)]
        #visibility struct #command_ident;

        #impl_header {
            fn build(self) -> lieutenant::CommandSpec<#ctx_actual_type> {
                #command_spec
            }
        }
    };
    tokens.into()
}

fn parse_usage(usage: &str) -> Usage {
    let mut arguments = vec![];

    for splitted in usage.split(" ") {
        let (first, middle) = splitted.split_at(1.min(splitted.len()));
        let (middle, last) = middle.split_at(middle.len() - 1);
        match (first, middle, last) {
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

    Usage {
        arguments,
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

fn detect_context_type<'a>(
    parameter_types: &[&PatType],
    first_arg: Option<&'a FnArg>,
) -> Option<(&'a Type, &'a Pat)> {
    first_arg
        .map(|first_arg| {
            let first_arg = match first_arg {
                FnArg::Typed(arg) => arg,
                _ => unreachable!(),
            };

            // check if any parameter types are this first argument
            if parameter_types
                .iter()
                .any(|param| param.pat == first_arg.pat)
            {
                None
            } else {
                Some((first_arg.ty.as_ref(), first_arg.pat.as_ref()))
            }
        })
        .flatten()
        .map(|(ty, pat)| {
            let ty = match ty {
                Type::Reference(reference) => &reference.elem,
                x => abort!(x.span(), "context input must be a reference";

                    help = "change the type of the first function parameter to be a mutable reference";
                ),
            };

            (ty.as_ref(), pat)
        })
}

fn generate_command_spec(
    usage: &Usage,
    description: Option<String>,
    parameters: &[&PatType],
    ctx_type: Option<(&Type, &Pat)>,
    block: &Block,
) -> TokenStream {
    // let mut statements = vec![];

    let ctx_param = match ctx_type {
        Some((t, _)) => quote! { #t },
        None => quote! { C },
    };

    let mut arguments = vec![];

    let mut i = 0;
    for argument in &usage.arguments {
        let argument = match argument {
            Argument::Parameter { name } | Argument::OptionalParameter { name } => {
                let argument_type = parameters[i];

                let ty = &argument_type.ty;
                i += 1;

                quote! {
                    lieutenant::Argument::Parser {
                        name: #name.into(),
                        checker: Box::new(<<#ty as lieutenant::ArgumentKind<#ctx_param>>::Checker
                            as lieutenant::ArgumentChecker<#ctx_param>>::default())
                    }
                }
            }
            Argument::Literal { value } => {
                quote! {
                    lieutenant::Argument::Literal {
                        value: #value.into(),
                    }
                }
            }
        };

        arguments.push(quote! {
            arguments.push(#argument);
        });
    }

    let mut parse_args = vec![];

    let mut i = 0;
    for argument in usage.arguments.iter() {
        match argument {
            Argument::Parameter { .. } | Argument::OptionalParameter { .. } => {
                let parameter = parameters[i];
                let ident = &parameter.pat;
                let ty = &parameter.ty;
                let ctx_ident = match ctx_type {
                    Some((_, ident)) => quote! { #ident },
                    None => quote! { _ctx },
                };

                parse_args.push(quote! {
                    let #ident = <<#ty as lieutenant::ArgumentKind<#ctx_param>>::Parser
                    as lieutenant::ArgumentParser<#ctx_param>>::default().parse(#ctx_ident, &mut input).unwrap();
                });

                i += 1;
            },
            Argument::Literal { .. } => parse_args.push(quote! { input.head(" "); }),
        }
    }

    let ctx_type = match ctx_type {
        Some((t, name)) => quote! { #name: &mut #t },
        None => quote! { _ctx: &mut C },
    };

    let description = match description {
        Some(description) => quote! { Some(#description.into()) },
        None => quote! { None },
    };

    let arguments_len = arguments.len();

    let res = quote! {
        let mut arguments = Vec::with_capacity(#arguments_len);
        #(#arguments)*

        lieutenant::CommandSpec {
            arguments,
            description: #description,
            exec: Box::new(|#ctx_type, args| {
                use lieutenant::{ArgumentParser as _, ArgumentChecker as _};
                let mut input = lieutenant::Input::new(args);
                #(#parse_args)*
                #block
            }),
        }
    };
    res
}
