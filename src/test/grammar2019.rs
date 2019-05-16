use crate::grammar::{Grammar, Formula};

//-------------------------------------------------------------------------------------------------
// TYPE

/// represents a state
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State
{
   Expr,
   Factor,
   One,
   Add,
   Mul
}

//-------------------------------------------------------------------------------------------------
// FUNCTIONS

/// computes a formula
fn compute(formula: &[State]) -> i64
{
   /// computes the first element of the formula and returns its value followed with any leftover
   fn compute_rec(formula: &[State]) -> (i64, &[State])
   {
      match formula
      {
         [formula.., State::One] => (1, formula),
         [formula.., State::Add] =>
         {
            let (x, formula) = compute_rec(formula);
            let (y, formula) = compute_rec(formula);
            (x + y, formula)
         }
         [formula.., State::Mul] =>
         {
            let (x, formula) = compute_rec(formula);
            let (y, formula) = compute_rec(formula);
            (x * y, formula)
         }
         [.., uncomputable] => panic!("Tried to compute a non terminal state : {:?}", uncomputable),
         [] => panic!("Tried to compute the empty formula.")
      }
   }
   // checks wether there is any leftover
   match compute_rec(formula)
   {
      (result, []) => result,
      (_, leftover) => panic!("There are some leftover states : {:?} => {:?}", formula, leftover)
   }
}

//-------------------------------------------------------------------------------------------------
// GRAMMAR

/// an implementation of the, very simple, 2019 grammar
impl Grammar for State
{
   /// represents the root of a formula
   fn root_state() -> State
   {
      State::Expr
   }

   /// expands a state into potential substitution rules
   /// an empty vector represents a terminal state: there is no rule associated with it
   fn expand(self) -> Vec<Vec<State>>
   {
      match self
      {
         State::Expr => vec![vec![State::One], 
                        vec![State::Add, State::Expr, State::Expr], 
                        vec![State::Mul, State::Factor, State::Factor]],
         State::Factor => vec![vec![State::Add, State::Expr, State::Expr], 
                               vec![State::Mul, State::Factor, State::Factor]],
         _ => vec![]
      }
   }

   /// turn a formula into a displayable string
   fn to_string(formula: &Formula<State>) -> String
   {
      /// turn the first element of the formula into a string and returns its value followed with any leftover
      fn to_string_rec(formula: &[State]) -> (String, &[State])
      {
         match formula
         {
            [formula.., State::One] => ("1".to_string(), formula),
            [formula.., State::Add] =>
            {
               let (x, formula) = to_string_rec(formula);
               let (y, formula) = to_string_rec(formula);
               let result = format!("{} + {}", x, y);
               (result, formula)
            }
            [formula.., State::Mul] =>
            {
               let (x, formula) = to_string_rec(formula);
               let (y, formula) = to_string_rec(formula);
               let result = format!("({})*({})", x, y);
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
   fn evaluate(formula: &Formula<State>) -> Option<f64>
   {
      let value = compute(formula);
      let score = (2019 - value).abs() as f64;
      Some(-score)
   }

   /// computes the cost of a formula to build a pareto front
   fn cost(formula: &Formula<State>) -> usize
   {
      formula.iter().filter(|&&s| s == State::One).count()
   }
}
