use crate::grammar::{Grammar, Formula};

//-------------------------------------------------------------------------------------------------
// TYPE

/// represents a state
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum State
{
   Expr,
   Base,
   Function,
   FCos,
   FSin,
   FLog,
   FSqrt,
   Operator,
   O(char),
   Variable,
   Number,
   N(i8)
}

//-------------------------------------------------------------------------------------------------
// FUNCTIONS

impl State
{
   /// returns true if the given state represents a function
   fn is_function(&self) -> bool
   {
      match self
      {
         State::FCos | State::FLog | State::FSin | State::FSqrt => true,
         _ => false
      }
   }
   /// returns a string representation of the state
   fn to_string(&self) -> String
   {
      match self
      {
         State::FCos => "cos".to_string(),
         State::FLog => "log".to_string(),
         State::FSin => "sin".to_string(),
         State::FSqrt => "sqrt".to_string(),
         _ => unimplemented!()
      }
   }

   fn to_function(&self) -> fn(f64) -> f64
   {
      match self
      {
         State::FCos => f64::cos,
         State::FLog => f64::ln,
         State::FSin => f64::sin,
         State::FSqrt => f64::sqrt,
         _ => panic!("this is not a function")
      }
   }
}

fn to_operator(c: char) -> fn(f64, f64) -> f64
{
   match c
   {
      '+' => |x, y| x + y,
      '-' => |x, y| x + y,
      '*' => |x, y| x * y,
      '/' => |x, y| x / y,
      '^' => |x, y| x.powf(y),
      _ => panic!("this is not an operator")
   }
}

/// computes a formula
fn compute(formula: &[State]) -> Box<(Fn(f64) -> f64)>
{
   /// computes the first element of the formula and returns its value followed with any leftover
   fn compute_rec(formula: &[State]) -> (Box<(Fn(f64) -> f64)>, &[State])
   {
      match formula
      {
         [formula.., State::Variable] =>
         {
            let f = Box::new(|variable| variable);
            (f, formula)
         }
         [formula.., State::N(n)] =>
         {
            let x = *n as f64;
            let f = Box::new(move |_| x);
            (f, formula)
         }
         [formula.., State::O(c)] =>
         {
            let fc = to_operator(*c);
            let (fx, formula) = compute_rec(formula);
            let (fy, formula) = compute_rec(formula);
            let f = Box::new(move |variable| fc(fx(variable), fy(variable)));
            (f, formula)
         }
         [formula.., fun] if fun.is_function() =>
         {
            let ff = fun.to_function();
            let (fx, formula) = compute_rec(formula);
            let f = Box::new(move |variable| ff(fx(variable)));
            (f, formula)
         }
         [.., uncomputable] => panic!("Tried to compute a non terminal state : {:?}", uncomputable),
         [] => panic!("Tried to compute the empty formula.")
      }
   }
   // checks wether there is any leftover
   match compute_rec(formula)
   {
      (result, []) => result,
      (_, leftover) => panic!("There are some leftover states : {:?} => {:?}", formula, leftover)
   }
}

//-------------------------------------------------------------------------------------------------
// GRAMMAR

/// an implementation of the Kepler grammar
impl Grammar for State
{
   type ScoreType = Option<f64>;
   /// represents the root of a formula
   fn root_state() -> State
   {
      State::Expr
   }

   /// expands a state into potential substitution rules
   /// an empty vector represents a terminal state: there is no rule associated with it
   fn expand(self) -> Vec<Vec<State>>
   {
      match self
      {
         State::Expr => vec![vec![State::Base], vec![State::Function, State::Expr,], vec![State::Operator,
                                                                                          State::Expr,
                                                                                          State::Expr]],
         State::Base => vec![vec![State::Variable], vec![State::Number], vec![State::O('^'),
                                                                              State::Variable,
                                                                              State::Number]],
         State::Operator => ['+', '-', '/'].iter().map(|&o| vec![State::O(o)]).collect(),
         State::Number => (1..=4).map(|n| vec![State::N(n)]).collect(),
         State::Function => vec![vec![State::FCos], vec![State::FSin], vec![State::FLog], vec![State::FSqrt]],
         _ => vec![]
      }
   }

   /// turn a formula into a displayable string
   fn to_string(formula: &Formula<State>) -> String
   {
      /// turn the first element of the formula into a string and returns its value followed with any leftover
      fn to_string_rec(formula: &[State]) -> (String, &[State])
      {
         match formula
         {
            [formula.., State::Variable] => ("distance".to_string(), formula),
            [formula.., State::N(n)] => (n.to_string(), formula),
            [formula.., State::O(c)] =>
            {
               let (x, formula) = to_string_rec(formula);
               let (y, formula) = to_string_rec(formula);
               let result = format!("{} {} {}", x, c, y);
               (result, formula)
            }
            [formula.., f] if f.is_function() =>
            {
               let (x, formula) = to_string_rec(formula);
               let result = format!("{}({})", f.to_string(), x);
               (result, formula)
            }
            [.., uncomputable] => panic!("Tried to compute a non terminal state : {:?}", uncomputable),
            [] => panic!("Tried to turn the empty formula into a string.")
         }
      }
      // checks wether there is any leftover
      match to_string_rec(formula)
      {
         (result, []) => result,
         (_, leftover) => panic!("There are some leftover states : '{}' => {:?}", formula, leftover)
      }
   }

   /// evaluates a formula
   fn evaluate(formula: &Formula<State>) -> Self::ScoreType
   {
      let f = compute(formula); // evaluate the formula once to reuse it multiple times
      let distance = vec![0.72, 1.0, 1.52, 5.20, 9.53, 19.10];
      let period = vec![0.61, 1.00, 1.84, 11.90, 29.40, 83.50];
      let error: f64 = period.iter()
                             .zip(&distance)
                             .map(|(&period, &distance)| period - f(distance))
                             .map(|error| error * error)
                             .sum();
      if error.is_nan() || error.is_infinite()
      {
         None // take care of division by 0 and other such problems
      }
      else
      {
         Some(-error) // we want to minimize the error
      }
   }
}
