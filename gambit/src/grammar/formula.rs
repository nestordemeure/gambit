use std::ops::{Deref, DerefMut};
use super::Grammar;

/// represents a serie of states
#[derive(Clone)]
pub struct Formula<State: Grammar>(Vec<State>);

/// macro to acess methods of the inner vector
impl<State: Grammar> Deref for Formula<State>
{
   type Target = Vec<State>;
   fn deref(&self) -> &Self::Target
   {
      &self.0
   }
}

/// macro to acess methods of the inner vector
impl<State: Grammar> DerefMut for Formula<State>
{
   fn deref_mut(&mut self) -> &mut Self::Target
   {
      &mut self.0
   }
}

/// macro to display a formula
impl<State: Grammar> std::fmt::Display for Formula<State>
{
   fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
   {
      write!(f, "{}", State::to_string(self))
   }
}

impl<State: Grammar> Formula<State>
{
   /// creates a new, empty, formula
   pub fn empty() -> Formula<State>
   {
      Formula(vec![])
   }

   /// evaluates a formula
   pub fn evaluate(&self) -> State::ScoreType
   {
      State::evaluate(self)
   }

   /// computes the cost of a formula (useful for pareto front)
   pub fn cost(&self) -> usize
   {
      State::cost(self)
   }
}
