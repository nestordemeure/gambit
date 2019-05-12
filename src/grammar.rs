#[macro_use]
use cons;
use cons_list::ConsList;

/*
   an implementation of the, very simple, 2019 grammar
*/

/// represents a state
#[derive(Debug)]
pub enum State
{
   Expr,
   One,
   Add,
   Mul
}

/// represents the root of a formula
static rootState: State = State::Expr;

/// expands a state into potential substitution rules
/// an empty vector represents a terminal state  :there is no rule associated with it
pub fn expand(state: &State) -> Vec<ConsList<State>>
{
   match state
   {
      State::Expr => vec![cons!(State::One),
                          conslist!(State::Add, State::Expr, State::Expr),
                          conslist!(State::Mul, State::Expr, State::Expr)],
      _ => vec![]
   }
}

/// computes a formula
fn compute(formula: ConsList<State>) -> i64
{
   /// computes the first element of the formula and returns its value followed with any leftover
   fn computeRec(formula: ConsList<State>) -> (i64, ConsList<State>)
   {
      match formula.head()
      {
         Some(State::One) => (1, formula.tail()),
         Some(State::Add) =>
         {
            let (x, formula) = computeRec(formula.tail());
            let (y, formula) = computeRec(formula.tail());
            (x + y, formula.tail())
         }
         Some(State::Mul) =>
         {
            let (x, formula) = computeRec(formula.tail());
            let (y, formula) = computeRec(formula.tail());
            (x * y, formula.tail())
         }
         Some(uncomputableState) => panic!("Tried to compute a non terminal state : {:?}", uncomputableState),
         None => panic!("Tried to compute the empty formula.")
      }
   }
   /// checks wether there is any leftover
   match computeRec(formula)
   {
      (result, ref emptyFormula) if emptyFormula.is_empty() => result,
      (_, leftover) => panic!("There are some leftover states : {:?}", leftover)
   }
}

/// evaluates a formula
pub fn evaluate(formula: ConsList<State>) -> f64
{
   let value = compute(formula);
   (2019 - value) as f64
}