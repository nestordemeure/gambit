use crate::grammar::Grammar;

/// represents a result of the algorithm
pub trait Result 
{
   /// updates the result
   fn update<State:Grammar>(&mut Self, formula : Vec<State>, score: Option<f64>);
}