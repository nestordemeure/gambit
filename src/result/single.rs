use super::{Result};
use crate::grammar::{Grammar, Formula, Wrap};
use std::fmt;

/// encapsulate the best result so far
pub struct Single<State:Grammar>
{
   pub score: f64,
   pub formula: Formula<State>
}

/// macro to display a result
impl<State:Grammar> fmt::Display for Single<State> 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        write!(f, "{{score:{}\tformula:'{}'}}", self.score, self.formula)
    }
}

impl<State:Grammar> Result<State> for Single<State>
{
   /// creates an empty result
   fn new() -> Single<State>
   {
      Single { score: std::f64::NEG_INFINITY, formula: Formula::<State>::empty() }
   }

   /// returns the best formula, score so far
   fn best(&self) -> (Formula<State>, f64)
   {
      (self.formula.clone(), self.score)
   }
   
   /// if the result is better than the best result so far, we update it
   fn update(&mut self, formula: Formula<State>, score: State::ScoreType) -> bool
   {
      match score.wrap()
      {
         Some(score) if score > self.score =>
         {
            self.score = score;
            self.formula = formula;
            true
         }
         _ => false
      }
   }
}
