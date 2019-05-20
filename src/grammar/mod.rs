mod formula;
pub use formula::Formula;

mod shortest_rule;

/// represents a grammar and all the associated operations
pub trait Grammar
   where Self: Copy + Clone + std::hash::Hash + std::cmp::Eq
{
   type ScoreType: Copy + std::fmt::Debug;

   /// represents the root of a formula
   fn root_state() -> Self;

   /// expands a state into potential substitution rules
   /// an empty vector represents a terminal state: there is no rule associated with it
   fn expand(self) -> Vec<Vec<Self>>;

   /// turn a formula into a displayable string
   fn to_string(formula: &Formula<Self>) -> String;

   /// evaluates a formula
   fn evaluate(formula: &Formula<Self>) -> Self::ScoreType;

   /// computes teh cost of the formula (useful to build a pareto front)
   fn cost(formula: &Formula<Self>) -> usize
   {
      formula.len()
   }
}

// TODO implement macro that derive grammar from simple representation
