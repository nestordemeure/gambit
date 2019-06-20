#![recursion_limit = "128"]
#![feature(proc_macro_diagnostic)]

//use gambit::grammar;
extern crate proc_macro;
use self::proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item, File, Expr, parse_quote};

/*
 * input contains :
 * an enum with the state type
 * the expansion rules
 * an evaluation function
 *
 * to use the AST, see :
 * https://docs.rs/syn/0.15.36/syn/enum.Item.html
 */
#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream
{
   let items = parse_macro_input!(input as File).items;

   // extracts the states enum
   let state_typedef = items.iter()
                            .find_map(|item| match item
                            {
                               Item::Enum(states) => Some(states),
                               _ => None
                            })
                            .expect("You forgot to define an enum to represent the states.");
   // extracts the name of the state type and its variants
   let state_typename = &state_typedef.ident;
   let state_names: Vec<&syn::Ident> = state_typedef.variants.iter().map(|variant| &variant.ident).collect();
   let root_state = state_names.first().expect("You need to have at least one state that is used as a root.");
   println!("typename: {}, variants:{:?}", state_typename, state_names);

   // extracts the rules function
   // NOTE: we could check the types to insure that the fucntion is properly formated
   let fn_rules = items.iter()
                       .find_map(|item| match item
                       {
                          Item::Fn(rules) if rules.ident == "rules" => Some(rules),
                          _ => None
                       })
                       .expect("You forgot to define a 'rules' function.");

   // extracts the evaluate function
   // NOTE: we could check the types to insure that the fucntion is properly formated
   let (fn_evaluate, score_typename) =
      items.iter()
           .find_map(|item| match item
           {
              Item::Fn(evaluate) if evaluate.ident == "evaluate" => match &evaluate.decl.output
              {
                 syn::ReturnType::Type(_, resultType) => Some((evaluate, resultType)),
                 _ => None
              },
              _ => None
           })
           .expect("You forgot to define an 'evaluate' function.");
   //println!("score : {:#?}", score_typename);

   //let item_evaluate = item.find();
   let expanded = quote! {
      // type representing the states
      #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
      #state_typedef

      /// computes a formula
      fn interpret(formula: &[#state_typename]) -> i64
      {
         unimplemented!()
      }

      // implementing grammar for the state
      impl Grammar for #state_typename
      {
         /// the type of the scores
         type ScoreType = #score_typename;

         /// represents the root of a formula
         fn root_state() -> #state_typename
         {
            #root_state
         }

         /// expands a state into potential substitution rules
         /// an empty vector represents a terminal state: there is no rule associated with it
         fn expand(self) -> Vec<Vec<#state_typename>>
         {
            unimplemented!()
         }

         /// turn a formula into a displayable string
         fn to_string(formula: &Formula<#state_typename>) -> String
         {
            unimplemented!()
         }

         //#fn_evaluate
         /// evaluates a formula
         fn evaluate(formula: &Formula<#state_typename>) -> Self::ScoreType
         {
            let value = interpret(formula);
            let score = (2019 - value).abs() as f64;
            -score
         }

         //TODO: optional cost function
      }
   };

   TokenStream::from(expanded)
}
