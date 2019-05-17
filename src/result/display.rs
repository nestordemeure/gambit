use super::{Result};
use crate::grammar::{Grammar, Formula};
use std::fmt::Display;

/// encapsulate a result but displays every improvement to the current best solution
pub struct DisplayProgress<ResultType>(ResultType);

/// implements the display trait needed by the result trait
impl<ResultType:Display> Display for DisplayProgress<ResultType>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result 
    {
        write!(f, "{}", self.0)
    }
}

/// implements the result trait
impl<State, ResultType> Result<State> for DisplayProgress<ResultType> where State: Grammar, ResultType:Result<State>
{
   fn new() -> Self
   {
      DisplayProgress(ResultType::new())
   }

   fn best(&self) -> (Formula<State>, f64)
   {
      self.0.best()
   }
   
   /// update the result and displays a message if we improved on the best value so far
   fn update(&mut self, formula: Formula<State>, score: State::ScoreType) -> bool
   {
      let improvement = self.0.update(formula, score);
      if improvement
      {
         let (formula, score) = self.0.best();
         println!("New result, score={:?} for '{}'", score, formula);
      }
      improvement
   }
}

// TODO use a similar approach to deal with options in distributions ?
// this would require a unified approach to failure, the one used in thompson would make sense
