use gambit::grammar::{Grammar, Formula};

//-------------------------------------------------------------------------------------------------
// TYPE

/// represents the functions that can appear in the code
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Function
{
   Cos,
   Sin,
   Log,
   Sqrt
}

/// represents a state
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum State
{
   Expr,
   Base,
   Function,
   F(Function),
   Operator,
   O(char),
   Variable,
   Number,
   N(i8)
}

//-------------------------------------------------------------------------------------------------
// FUNCTIONS

impl Function
{
   /// returns a string representation of the function
   fn to_string(self) -> String
   {
      match self
      {
         Function::Cos => "cos".to_string(),
         Function::Log => "log".to_string(),
         Function::Sin => "sin".to_string(),
         Function::Sqrt => "sqrt".to_string()
      }
   }

   /// returns the function represented
   fn to_function(self) -> fn(f64) -> f64
   {
      match self
      {
         Function::Cos => f64::cos,
         Function::Log => f64::ln,
         Function::Sin => f64::sin,
         Function::Sqrt => f64::sqrt
      }
   }
}

/// returns the operator represented
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

//-------------------------------------------------------------------------------------------------
// INTERPRETATION

/// interprets a formula into a function using the third futamura projection
fn interpret(formula: &[State]) -> Box<(dyn Fn(f64) -> f64)>
{
   /// computes the first element of the formula and returns its value followed with any leftover
   fn interpret_rec(formula: &[State]) -> (Box<(dyn Fn(f64) -> f64)>, &[State])
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
            let (fx, formula) = interpret_rec(formula);
            let (fy, formula) = interpret_rec(formula);
            let f = Box::new(move |variable| fc(fx(variable), fy(variable)));
            (f, formula)
         }
         [formula.., State::F(fun)] =>
         {
            let ff = fun.to_function();
            let (fx, formula) = interpret_rec(formula);
            let f = Box::new(move |variable| ff(fx(variable)));
            (f, formula)
         }
         [.., uncomputable] => panic!("Tried to interpret a non terminal state : {:?}", uncomputable),
         [] => panic!("Tried to interpret the empty formula.")
      }
   }
   // checks wether there is any leftover
   match interpret_rec(formula)
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
         State::Function =>
         {
            [Function::Cos, Function::Sin, Function::Log, Function::Sqrt].iter()
                                                                         .map(|&f| vec![State::F(f)])
                                                                         .collect()
         }
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
            [formula.., State::F(f)] =>
            {
               let (x, formula) = to_string_rec(formula);
               let result = format!("{}({})", f.to_string(), x);
               (result, formula)
            }
            [.., uncomputable] => panic!("Tried to display a non terminal state : {:?}", uncomputable),
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
      let f = interpret(formula); // interprets the formula once to reuse it multiple times
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
