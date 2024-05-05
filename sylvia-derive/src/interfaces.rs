use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{GenericArgument, ItemImpl};

use crate::crate_module;
use crate::parser::attributes::msg::MsgType;
use crate::parser::{ContractMessageAttr, ParsedSylviaAttributes};

#[derive(Debug, Default)]
pub struct Interfaces {
    interfaces: Vec<ContractMessageAttr>,
}

impl Interfaces {
    pub fn new(source: &ItemImpl) -> Self {
        let interfaces = ParsedSylviaAttributes::new(source.attrs.iter()).messages_attrs;
        Self { interfaces }
    }

    pub fn emit_glue_message_variants(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        let sylvia = crate_module();

        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module,
                    variant,
                    generics,
                    ..
                } = interface;

                let generics = if !generics.is_empty() {
                    quote! { < #generics > }
                } else {
                    quote! {}
                };
                let interface_enum =
                    quote! { <#module ::sv::Api #generics as #sylvia ::types::InterfaceApi> };
                let type_name = msg_ty.as_accessor_name();

                quote! { #variant ( #interface_enum :: #type_name) }
            })
            .collect()
    }

    pub fn emit_messages_call(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr { module, .. } = interface;

                let ep_name = msg_ty.emit_ep_name();
                let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), module.span());
                quote! {
                    &#module ::sv:: #messages_fn_name()
                }
            })
            .collect()
    }

    pub fn emit_deserialization_attempts(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module, variant, ..
                } = interface;
                let ep_name = msg_ty.emit_ep_name();
                let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), module.span());

                quote! {
                    let msgs = &#module ::sv:: #messages_fn_name();
                    if msgs.into_iter().any(|msg| msg == &recv_msg_name) {
                        match val.deserialize_into() {
                            Ok(msg) => return Ok(Self:: #variant (msg)),
                            Err(err) => return Err(D::Error::custom(err)).map(Self:: #variant),
                        };
                    }
                }
            })
            .collect()
    }

    pub fn emit_response_schemas_calls(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        let sylvia = crate_module();

        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module, generics, ..
                } = interface;

                let generics = if !generics.is_empty() {
                    quote! { < #generics > }
                } else {
                    quote! {}
                };

                let type_name = msg_ty.as_accessor_name();
                quote! {
                    <#module ::sv::Api #generics as #sylvia ::types::InterfaceApi> :: #type_name :: response_schemas_impl()
                }
            })
            .collect()
    }

    pub fn emit_dispatch_arms(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        let sylvia = crate_module();
        let contract_enum_name = msg_ty.emit_msg_wrapper_name();

        self.interfaces.iter().map(|interface| {
            let ContractMessageAttr {
                variant,
                customs,
                ..
            } = interface;

            let ctx = msg_ty.emit_ctx_dispatch_values(customs);

            match (msg_ty, customs.has_msg) {
                (MsgType::Exec, true) | (MsgType::Sudo, true) => quote! {
                    #contract_enum_name:: #variant(msg) => #sylvia ::into_response::IntoResponse::into_response(msg.dispatch(contract, Into::into( #ctx ))?)
                },
                _ => quote! {
                    #contract_enum_name :: #variant(msg) => msg.dispatch(contract, Into::into( #ctx ))
                },
            }
        }).collect()
    }

    pub fn emit_impl_into_underlying(&self, msg_ty: &MsgType) -> TokenStream {
        let sylvia = crate_module();
        let contract_enum_name = msg_ty.emit_msg_wrapper_name();
        println!("{:?}", msg_ty);

        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module,
                    generics,
                    variant,
                    ..
                } = interface;

                let generics = if !generics.is_empty() {
                    quote! { < #generics > }
                } else {
                    quote! {}
                };
                let interface_enum =
                    quote! { <#module ::sv::Api #generics as #sylvia ::types::InterfaceApi> };
                let type_name = msg_ty.as_accessor_name();
                let whole_type_name = quote! {#interface_enum :: #type_name };

                println!("whole_type_name: {}", whole_type_name);
                println!("contract_enum_name: {}", contract_enum_name);
                println!("variant: {}", variant);

                println!("From<{:#}> for {:#}\n", whole_type_name, contract_enum_name);

                quote! {
                    impl From<#whole_type_name> for #contract_enum_name {
                        fn from(value: #whole_type_name) -> Self {
                            Self::#variant(value)
                        }
                    }
                }
            })
            .collect()
    }

    pub fn as_generic_args(&self) -> Vec<&GenericArgument> {
        self.interfaces
            .iter()
            .flat_map(|interface| &interface.generics)
            .collect()
    }
}
