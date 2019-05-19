use super::{Result};
use crate::grammar::{Grammar, Formula};
use std::fmt::Display;

/// converts a result from an option<f64> to an f64
pub struct Optional<ResultType>(ResultType);

impl<ResultType> Optional<ResultType>
{
   /// extracts the underlying result type
   pub fn get_result(self) -> ResultType
   {
      self.0
   }
}

/// implements the display trait needed by the result trait
impl<ResultType: Display> Display for Optional<ResultType>
{
   fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
   {
      write!(f, "{}", self.0)
   }
}

/// implements the result trait
impl<State, ResultType, UnderlyingScoreType> Result<State> for Optional<ResultType>
   where State: Grammar,
         ResultType: Result<State, ScoreType = UnderlyingScoreType>
{
   type ScoreType = Option<UnderlyingScoreType>;

   fn new() -> Self
   {
      Optional(ResultType::new())
   }

   fn best(&self) -> (Formula<State>, f64)
   {
      self.0.best()
   }

   /// update the result if the score_opt option type is not none
   fn update(&mut self, formula: Formula<State>, score_opt: Self::ScoreType) -> bool
   {
      if let Some(score) = score_opt
      {
         self.0.update(formula, score)
      }
      else
      {
         false
      }
   }
}
