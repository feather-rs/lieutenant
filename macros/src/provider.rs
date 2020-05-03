use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, GenericArgument, PathArguments, ReturnType, Type};
use syn::{FnArg, ItemFn, Pat};

/// Detected context type.
#[derive(Copy, Clone)]
enum ContextType<'a> {
    /// Concrete type
    Known {
        /// Type of the context
        typ: &'a Type,
        /// Ident of the context variable
        ident: &'a Pat,
    },
    /// Unknown - make the provider generic over contexts
    Generic,
}

/// Detected `Output` type.
#[derive(Clone)]
struct OutputType<'a> {
    /// The Ok type
    ok: &'a Type,
    /// The error type
    error: TokenStream,
    /// Whether the error type is defined by the block
    error_defined: bool,
}

pub fn provider(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    verify_input(&input);

    let ctx_type = detect_context_type(&input);
    let output_type = detect_output(&input);

    let tokens = generate_provider(&input, ctx_type, output_type);
    tokens.into()
}

/// Checks that the input function doesn't
/// have illegal conditions.
fn verify_input(input: &ItemFn) {
    // no generics
    if let Some(first_param) = input.sig.generics.params.iter().next() {
        let span = first_param.span();

        emit_error!(
            span, "provider function may not have generic paramters";

            help = "remove the generic parameter"
        );
    }

    // must be async
    if input.sig.asyncness.is_none() {
        let span = input.sig.fn_token.span();
        let name = &input.sig.ident;

        emit_error!(
            span, "provider function must be `async`";

            help = "make the function async: `async fn {}`", name
        );
    }
}

/// Detects the context type of the input function.
fn detect_context_type(input: &ItemFn) -> ContextType {
    // Context is first argument.
    if let Some(first_arg) = input.sig.inputs.iter().next() {
        let first_arg = match first_arg {
            FnArg::Receiver(rec) => {
                let span = rec.span();
                abort!(
                    span, "provider function may not take `self` parameter";

                    help = "remove the `self` parameter"
                );
            }
            FnArg::Typed(typ) => typ,
        };

        let typ = match first_arg.ty.as_ref() {
            Type::Reference(r) => {
                if let Some(mutability) = r.mutability {
                    let span = mutability.span();

                    emit_error!(
                        span, "provider context parameter cannot be mutable";

                        help = "remove the `mut`"
                    );
                }

                r.elem.as_ref()
            }
            typ => abort!(
                typ.span(),
                "provider context type must be an immutable reference"
            ),
        };

        let ident = first_arg.pat.as_ref();

        ContextType::Known { typ, ident }
    } else {
        // No first argument - context type can be any Context
        ContextType::Generic
    }
}

/// Detects the provider's output type.
fn detect_output(input: &ItemFn) -> OutputType {
    let ret = &input.sig.output;

    match ret {
        ReturnType::Default => {
            abort!(ret.span(), "provider must have a return type"; help = "add a return type for the type you wish to provide")
        }
        ReturnType::Type(_, typ) => {
            // Extract the output type.
            // If it's a Result, we know both Ok and Err.
            // If it's not a Result, we assume Error is Infallible and
            // Ok is the return type.

            match typ.as_ref() {
                Type::Path(path) => {
                    let path = &path.path;

                    let last_segment = path.segments.iter().last().unwrap();

                    if last_segment.ident.to_string() == "Result" {
                        match &last_segment.arguments {
                            PathArguments::None | PathArguments::Parenthesized(_) => abort!(
                                last_segment.span(),
                                "provider `Result` return type must have Ok and Err variants"
                            ),
                            PathArguments::AngleBracketed(bracketed) => {
                                let mut iter = bracketed.args.iter();
                                let ok = iter.next().unwrap();
                                let error = iter.next().unwrap();

                                let (ok, error) = match (ok, error) {
                                    (GenericArgument::Type(ok), GenericArgument::Type(error)) => {
                                        (ok, error)
                                    }
                                    _ => abort!(ok.span(), "result must have two type parameters"),
                                };

                                OutputType {
                                    ok,
                                    error: quote! { #error },
                                    error_defined: true,
                                }
                            }
                        }
                    } else {
                        OutputType {
                            ok: typ,
                            error: quote! { std::convert::Infallible },
                            error_defined: false,
                        }
                    }
                }
                typ => {
                    abort!(typ.span(), "invalid provider return type"; note = "expected concrete type, not a reference")
                }
            }
        }
    }
}

fn generate_provider(
    input: &ItemFn,
    ctx_type: ContextType,
    output_type: OutputType,
) -> TokenStream {
    let ident = &input.sig.ident;
    let block = &input.block;
    let vis = &input.vis;

    let impl_head = match ctx_type {
        ContextType::Known { typ, .. } => quote! {  impl lieutenant::Provider<#typ> for #ident },
        ContextType::Generic => {
            quote! { impl <C> lieutenant::Provider<C> for #ident where C: lieutenant::Context }
        }
    };

    let ok = output_type.ok;
    let error = &output_type.error;

    let (ctx_param_type, ctx_param_ident) = match ctx_type {
        ContextType::Known { typ, ident } => (quote! { #typ }, quote! { #ident }),
        ContextType::Generic => (quote! { C }, quote! { > }),
    };

    let convert_ret = if output_type.error_defined {
        quote! { ret }
    } else {
        quote! { Ok(ret) }
    };

    let tokens = quote! {
        #[allow(non_camel_case_types)]
        #[derive(Default)]
        #vis struct #ident;

        #impl_head {
            type Output = #ok;
            type Error = #error;

            fn provide(&self, #ctx_param_ident: &#ctx_param_type) -> std::result::Result<Self::Output, Self::Error> {
                let ret = {
                    #block
                };

                #convert_ret
            }
        }
    };
    tokens
}
