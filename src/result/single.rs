use super::Result;
use crate::grammar::Grammar;

/// encapsulate the best result so far
#[derive(Debug)]
pub struct Single<State>
{
   pub score: f64,
   pub formula: Vec<State>
}

impl<State:Grammar> Result<State> for Single<State>
{
   /// creates an empty result
   fn new() -> Single<State>
   {
      Single { score: std::f64::NEG_INFINITY, formula: vec![] }
   }

   /// returns the best formula, score so far
   fn best(&self) -> (Vec<State>, f64)
   {
      (self.formula.to_vec(), self.score)
   }

   /// if the result is better than the best result so far, we update it
   fn update(&mut self, formula: Vec<State>, score: f64)
   {
      if score > self.score
      {
         self.score = score;
         self.formula = formula;
      }
   }
}