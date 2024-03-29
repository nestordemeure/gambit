pub mod single;
pub mod pareto;
pub mod display;
pub mod option;

use crate::grammar::{Grammar, Formula};
pub use single::Single;
pub use pareto::ParetoFront;
pub use display::DisplayProgress;
pub use option::Optional;

/// represents a result of the algorithm
pub trait Result<State>: std::fmt::Display
   where State: Grammar
{
   type ScoreType;

   /// creates a new instance of the type
   fn new() -> Self;

   /// returns the best (formula,score) so far
   fn best(&self) -> (Formula<State>, f64);

   /// updates the result with a f64 score, returns true if the result is better than the best so far
   fn update(&mut self, formula: Formula<State>, score: Self::ScoreType) -> bool;
}
