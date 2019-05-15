use super::Result;

/// stores a pareto front of the results so far
#[derive(Debug)]
pub struct ParetoFront<State>
{
   score: f64,
   formula: Vec<State>
}