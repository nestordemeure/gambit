use std::ops::{Deref, DerefMut};
use std::fmt;

//-------------------------------------------------------------------------------------------------
// GRAMMAR

/// represents a grammar and all the associated operations
/// TODO implement macro that derive grammar from simple representation
pub trait Grammar
   where Self: Copy + Clone
{
   /// represents the root of a formula
   fn root_state() -> Self;

   /// expands a state into potential substitution rules
   /// an empty vector represents a terminal state: there is no rule associated with it
   fn expand(self) -> Vec<Vec<Self>>;

   /// turn a formula into a displayable string
   fn to_string(formula: &Formula<Self>) -> String;

   /// evaluates a formula
   fn evaluate(formula: &Formula<Self>) -> Option<f64>;
}

//-------------------------------------------------------------------------------------------------
// FORMULA

/// represents a serie of states
#[derive(Clone)]
pub struct Formula<State:Grammar>
(
   Vec<State>
);

/// macro to acess methods of inner vector
impl<State:Grammar> Deref for Formula<State>
{
   type Target = Vec<State>;
   fn deref(&self) -> &Self::Target 
   {
      &self.0
   }
}

/// macro to acess methods of inner vector
impl<State:Grammar> DerefMut for Formula<State>
{
   fn deref_mut(&mut self) -> &mut Self::Target 
   {
      &mut self.0
   }
}

/// macro to display a formula
impl<State:Grammar> fmt::Display for Formula<State> 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        write!(f, "{}", State::to_string(self))
    }
}

impl<State:Grammar> Formula<State>
{
   /// creates a new, empty, formula
   pub fn empty() -> Formula<State>
   {
      Formula(vec![])
   }
   
   /// evaluates a formula
   pub fn evaluate(&self) -> Option<f64>
   {
      State::evaluate(self)
   }
}
