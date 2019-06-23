#![recursion_limit = "128"]
#![feature(proc_macro_diagnostic)]
#![feature(box_patterns)]

//use gambit::grammar;
extern crate proc_macro;
use std::collections::HashSet;
use self::proc_macro::TokenStream;
use quote::quote;
use syn::*;

//-----------------------------------------------------------------------------
// STATE TYPEDEF

/// extracts the name of the state type, the rootstate and a set of all state names
/// TODO: we could assert that all variants are simple
fn get_states(items: &Vec<Item>) -> (&Visibility, &Ident, &Ident, HashSet<&Ident>)
{
   // finds the states enum amongst the items
   let state_typedef = items.iter()
                            .find_map(|item| match item
                            {
                               Item::Enum(states) => Some(states),
                               _ => None
                            })
                            .expect("You forgot to define an enum to represent the states.");

   // visibility of the type
   let visibility = &state_typedef.vis;

   // name of the type
   let state_typename = &state_typedef.ident;

   // first variant which is used as rootstate
   let root_state = &state_typedef.variants.first().expect("You need to have at least one state that is used as a root.").into_value().ident;

   // vector of all variants
   let state_names: HashSet<&Ident> = state_typedef.variants.iter().map(|variant| &variant.ident).collect();

   (visibility, state_typename, root_state, state_names)
}

//-----------------------------------------------------------------------------
// RULES

/// returns all of the rules as a hashmap of state->Vec<TokenStream>
/// TODO: we could check the types to insure that the fucntion is properly formated
fn get_rules(items: &Vec<Item>)
{
   // extracts the rules item
   let fn_rules = items.iter()
                       .find_map(|item| match item
                       {
                          Item::Fn(rules) if rules.ident == "rules" => Some(rules),
                          _ => None
                       })
                       .expect("You forgot to define a 'rules' function.");

   // get iterator of inputs of type :
   // https://docs.rs/syn/0.15.36/syn/enum.FnArg.html
   // should ignore the first input and consider the others as captured
   //let inputs = fn_rules.decl.inputs.iter();
   // vector of statements
   if let Some(Stmt::Expr(Expr::Match(match_expression))) = fn_rules.block.stmts.first()
   {
      for arm in &match_expression.arms
      {
         let pattern = arm.pats.iter().next().expect("you did not set a pattern for one of the rules");
         match pattern
         {
            Pat::Ident(patident) => 
            {
               // matched state
               let state = &patident.ident;
               // associated expression
               let expr = &arm.body;
               let tokens = quote!{#expr};
               println!("{} => {}", state, tokens);
               // TODO redact function that extracts States from tokenStream
            }
            //Pat::Struct(_) => println!("struct"),
            _ => panic!("You have used a pattern that is not authorized instead of a state in your 'rules' function.")
         }
      }
   }
   else
   {
      panic!("You forgot to define a match in your 'rules' function.")
   }
}

//-----------------------------------------------------------------------------
// EVALUATE

/// returns the evaluate function and its return type
/// TODO: we could check the input type of the function
fn get_evaluate(items: &Vec<Item>) -> (&ItemFn, &Type)
{
   items.iter().find_map(|item| match item
   {
      Item::Fn(evaluate) if evaluate.ident == "evaluate" => match &evaluate.decl.output
      {
         ReturnType::Type(_, box resultType) => Some((evaluate, resultType)),
         _ => None
      },
      _ => None
   })
   .expect("You forgot to define an 'evaluate' function.")
}

//-----------------------------------------------------------------------------
// MAIN

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
   let (state_type_visibility, state_typename, root_state, states) = get_states(&items);

   // extracts the rules function
   get_rules(&items);

   // extracts the evaluate function
   let (fn_evaluate, score_typename) = get_evaluate(&items);

   //let item_evaluate = item.find();
   let expanded = quote! {
      // type representing the states
      #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
      #state_type_visibility enum #state_typename
      {
         #(#states),*
      }

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
            #state_typename::#root_state
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
            // here stringify!(#expr) might be useful
            unimplemented!()
         }

         /// evaluates a formula
         #fn_evaluate

         //TODO: optional cost function
      }
   };

   TokenStream::from(expanded)
}
