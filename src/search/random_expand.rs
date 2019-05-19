use rand::Rng;
use rand::seq::SliceRandom;
use crate::grammar::{Grammar, Formula};

/// takes a stack and a formula and randomly expands it until we reach a complete formula
/// avoids useless intermediate structures and tests
pub fn random_expand<State, RNG>(mut formula: Formula<State>,
                                 mut stack: Vec<State>,
                                 rng: &mut RNG,
                                 mut available_depth: i64)
                                 -> (Formula<State>, State::ScoreType)
   where State: Grammar,
         RNG: Rng
{
   loop
   {
      match stack.pop()
      {
         None =>
         {
            let score = formula.evaluate();
            return (formula, score);
         }
         Some(state) =>
         {
            match state.expand().as_slice()
            {
               [] =>
               {
                  // terminal state
                  formula.push(state);
               }
               [rule] =>
               {
                  // single rule, we can focus on it
                  stack.extend(rule);
               }
               [rule, ..] if available_depth <= 0 =>
               {
                  // no more depth available to make decisions
                  available_depth -= 1;
                  stack.extend(rule);
               }
               rules =>
               {
                  // non terminal state, it costs a node
                  available_depth -= 1;
                  let rule = rules.choose(rng).unwrap();
                  stack.extend(rule);
               }
            }
         }
      }
   }
}