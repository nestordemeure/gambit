use gambit::grammar::{Grammar, Formula};
use gambit_macro::grammar;

grammar! {
   pub enum State
   {
      Expr,
      Factor
   }

   // how to expand a state ?
   fn rules(state: State) // no variable at the moment
   {
      match(state)
      {
         Expr => 1,
         Expr => Expr + Expr,
         Expr => Factor * Factor,
         Factor => Expr + Expr, // a factor is never a one
         Factor => Factor * Factor,
      }
   }

   // how to evaluate a formula
   fn evaluate(formula: &Formula<State>) -> f64
   {
      let value = interpret(formula);
      let score = (2019 - value).abs() as f64;
      -score
   }
}
