
/// represents a grammar and all its operations
pub trait Grammar
   where Self: Copy + Clone
{
   /// represents the root of a formula
   fn root_state() -> Self;

   /// expands a state into potential substitution rules
   /// an empty vector represents a terminal state: there is no rule associated with it
   fn expand(self) -> Vec<Vec<Self>>;

   /// turn a formula into a displayable string
   fn to_string(formula: &[Self]) -> String;

   /// evaluates a formula
   fn evaluate(formula: &[Self]) -> Option<f64>;
}

// TODO is there an existin to_string trait ?
// TODO should we encapsulate a formula inside a struct
// TODO implement macro that derive grammar from simple representation