use crate::grammar::{Grammar, Formula};
pub mod single;
pub mod pareto;
pub use single::Single;

/// represents a result of the algorithm
pub trait Result<State> where State:Grammar
{
   /// creates a new instance of the type
   fn new() -> Self;

   /// returns the best (formula,score) so far
   fn best(&self) -> (Formula<State>, f64);

   /// updates the result with a f64 score
   fn update(&mut self, formula: Formula<State>, score: f64);

   /// updates the result with a option<f64> score
   fn update_opt(&mut self, formula: Formula<State>, score_opt: Option<f64>)
   {
      if let Some(score) = score_opt
      {
         self.update(formula, score)
      }
   }
}

// implements display of intermediate results ?
// implements pareto