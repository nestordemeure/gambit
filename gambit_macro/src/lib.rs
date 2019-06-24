#![recursion_limit = "128"]
#![feature(proc_macro_diagnostic)]
#![feature(box_patterns)]

//use gambit::grammar;
extern crate proc_macro;
use std::collections::HashSet;
use std::collections::HashMap;
use quote::quote;
use syn::*;

//-----------------------------------------------------------------------------
// STATE TYPEDEF

/// extracts the name of the state type, the rootstate and a set of all state names
/// TODO: we could assert that all variants are simple
fn get_states(items: &Vec<Item>) -> (&Visibility, &Ident, &Ident, HashSet<Ident>)
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
   let state_names: HashSet<Ident> = state_typedef.variants.iter().map(|variant| variant.ident.clone()).collect();

   (visibility, state_typename, root_state, state_names)
}

//-----------------------------------------------------------------------------
// RULES

/// returns all of the rules as a hashmap of state->Vec<TokenStream>
/// TODO: we could check the types to insure that the fucntion is properly formated
/// TODO: could be used to extract the keys
fn get_rules(items: &Vec<Item>) -> HashMap<&Ident, Vec<proc_macro2::TokenStream>>
{
   // extracts the rules item
   let fn_rules = items.iter()
                       .find_map(|item| match item
                       {
                          Item::Fn(rules) if rules.ident == "rules" => Some(rules),
                          _ => None
                       })
                       .expect("You forgot to define a 'rules' function.");
   
   // hashmap of state->[rule]
   let mut rules = HashMap::new();

   // get to match block
   if let Some(Stmt::Expr(Expr::Match(match_expression))) = fn_rules.block.stmts.first()
   {
      // adds each rule to the dictionnary
      for arm in &match_expression.arms
      {
         match arm.pats.iter().next().expect("you did not set a pattern for one of the rules")
         {
            Pat::Ident(patident) => 
            {
               // matched state
               let state = &patident.ident;

               // associated rule
               let expr = &arm.body;
               let tokens = quote!{#expr};
               
               // adds the rule to the rule set
               match rules.get_mut(state)
               {
                  None => {rules.insert(state, vec![tokens]);}
                  Some(v) => v.push(tokens)
               }
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
   
   rules
}

//-----------------------------------------------------------------------------
// EXPAND

/// takes a tokenstream, an hashset with all legal states and a vector in which to store all state meet in order
fn extract_states_from_rule(tokens: proc_macro2::TokenStream, states: &HashSet<Ident>, result: &mut Vec<Ident>)
{
   for tokentree in tokens.into_iter()
   {
      match tokentree 
      {
         // single identifier
         proc_macro2::TokenTree::Ident(ident) => 
         {
            // is it a state ?
            if states.contains(&ident)
            {
               result.push(ident)
            }
         }
         // tokenstream
         proc_macro2::TokenTree::Group(group) => 
         {
            extract_states_from_rule(group.stream(), states, result)
         }
         _ => ()
      }
   }
}

fn build_expand_rules(rules: &HashMap<&Ident, Vec<proc_macro2::TokenStream>>, states: &mut HashSet<Ident>, state_typename: &Ident) -> Vec<proc_macro2::TokenStream>
{
   let mut expanded_rules = Vec::new();
   
   for (state, rules) in rules 
   {
      let mut rule_number = 0;
      let mut state_of_rules = Vec::new();
      for rule in rules
      {
         rule_number += 1;
         let rule_header = Ident::new(&format!("{}_Rule{}", state, rule_number), proc_macro2::Span::call_site());
         states.insert(rule_header.clone());
         let mut rule_states = vec![rule_header];
         extract_states_from_rule(rule.clone(), states, &mut rule_states);
         let state_typename = (0..rule_states.len()).map(|_| state_typename);
         let quoted_rule = quote!{vec![#(#state_typename::#rule_states),*]};
         state_of_rules.push(quoted_rule);
      }
      
      let expanded = quote!{#state => vec![#(#state_of_rules),*]};
      //println!("{}", expanded);
      expanded_rules.push(expanded);
   }
   
   expanded_rules
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
pub fn grammar(input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
   let items = parse_macro_input!(input as File).items;

   // extracts the states enum
   let (state_type_visibility, state_typename, root_state, mut states) = get_states(&items);

   // extracts the rules function
   let rules = get_rules(&items);
   let expanded_rules = build_expand_rules(&rules, &mut states, &state_typename);

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
            match self
            {
               #(#expanded_rules,)*
               _ => vec![]
            }
         }

         /// turn a formula into a displayable string
         fn to_string(formula: &Formula<#state_typename>) -> String
         {
            // here stringify!(#expr) might be useful

            /// turn the first element of the formula into a string and returns its value followed with any leftover
            fn to_string_rec(formula: &[#state_typename]) -> (String, &[#state_typename])
            {
               match formula
               {
                  [formula.., #state_typename::Add] =>
                  {
                     // TODO
                     let (x, formula) = to_string_rec(formula);
                     let result = format!("{} + 1", x);
                     (result, formula)
                  }
                  [.., uncomputable] => panic!("Tried to compute a non terminal state : {:?}", uncomputable),
                  [] => panic!("Tried to turn the empty formula into a string.")
               }
            }
            // checks wether there is any leftover
            match to_string_rec(formula)
            {
               (result, []) => result,
               (_, leftover) => panic!("There are some leftover states : '{}' => {:?}", formula, leftover)
            }
         }

         /// evaluates a formula
         #fn_evaluate

         //TODO: optional cost function
      }
   };

   proc_macro::TokenStream::from(expanded)
}
