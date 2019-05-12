/*
   an implementation of the, very simple, 2019 grammar
*/

/// represents a state
#[derive(Copy, Clone, Debug)]
pub enum State
{
   Expr,
   One,
   Add,
   Mul
}

/// represents the root of a formula
pub static rootState: State = State::Expr;

/// expands a state into potential substitution rules
/// an empty vector represents a terminal state  :there is no rule associated with it
pub fn expand(state: State) -> Vec<Vec<State>>
{
   match state
   {
      State::Expr => vec![vec![State::One], vec![State::Add, State::Expr, State::Expr], vec![State::Mul,
                                                                                             State::Expr,
                                                                                             State::Expr]],
      _ => vec![]
   }
}

/// computes a formula
fn compute(formula: &[State]) -> i64
{
   /// computes the first element of the formula and returns its value followed with any leftover
   fn computeRec(formula: &[State]) -> (i64, &[State])
   {
      match formula
      {
         [State::One, formula..] => (1, formula),
         [State::Add, formula..] =>
         {
            let (x, formula) = computeRec(formula);
            let (y, formula) = computeRec(formula);
            (x + y, formula)
         }
         [State::Mul, formula..] =>
         {
            let (x, formula) = computeRec(formula);
            let (y, formula) = computeRec(formula);
            (x * y, formula)
         }
         [uncomputableState, ..] => panic!("Tried to compute a non terminal state : {:?}", uncomputableState),
         [] => panic!("Tried to compute the empty formula.")
      }
   }
   // checks wether there is any leftover
   match computeRec(formula)
   {
      (result, []) => result,
      (_, leftover) => panic!("There are some leftover states : {:?}", leftover)
   }
}

/// evaluates a formula
pub fn evaluate(formula: &[State]) -> Option<f64>
{
   let value = compute(formula);
   let score = (2019 - value) as f64;
   Some(score)
}