use super::Result;

/// encapsulate the best result so far
#[derive(Debug)]
pub struct SingleResult<State>
{
   score: f64,
   formula: Vec<State>
}

impl<State> Result<State> for SingleResult<State>
{
   /// creates an empty result
   fn new() -> SingleResult<State>
   {
      SingleResult { score: -std::f64::INFINITY, formula: vec![] }
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