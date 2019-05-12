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
pub static ROOTSTATE: State = State::Expr;

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
   fn compute_rec(formula: &[State]) -> (i64, &[State])
   {
      match formula
      {
         [State::One, formula..] => (1, formula),
         [State::Add, formula..] =>
         {
            let (x, formula) = compute_rec(formula);
            let (y, formula) = compute_rec(formula);
            (x + y, formula)
         }
         [State::Mul, formula..] =>
         {
            let (x, formula) = compute_rec(formula);
            let (y, formula) = compute_rec(formula);
            (x * y, formula)
         }
         [uncomputable, ..] => panic!("Tried to compute a non terminal state : {:?}", uncomputable),
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

/// evaluates a formula
pub fn evaluate(formula: &[State]) -> Option<f64>
{
   let value = compute(formula);
   let score = (2019 - value) as f64;
   Some(score)
}