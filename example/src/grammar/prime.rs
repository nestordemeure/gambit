use gambit::grammar::{Grammar, Formula};
use primes::PrimeSet;
use std::sync::Mutex;
use lazy_static;

//-------------------------------------------------------------------------------------------------
// TYPE

/// represents a state
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum State
{
   Expr,
   O(char),
   Variable,
   Number,
   Bit,
   Bit0,
   Bit1,
   EndBit
}

//-------------------------------------------------------------------------------------------------
// INTERPRETOR

/// interprets a sequence of bits as a strictly positiv number
/// NOTE: the number being non-zero helps avoid wasting iterations
fn interpret_bits(formula: &[State]) -> (i64, &[State])
{
   fn interpret_rec(formula: &[State], result: i64) -> (i64, &[State])
   {
      match formula
      {
         [formula.., State::Bit0] => interpret_rec(formula, result * 2),
         [formula.., State::Bit1] => interpret_rec(formula, result * 2 + 1),
         [formula.., State::EndBit] => (1 + result, formula),
         _ => panic!("no EndBit was detected")
      }
   }
   interpret_rec(formula, 0)
}

/// interprets a formula into a function using the third futamura projection
fn interpret(formula: &[State]) -> Box<(Fn(i64) -> i64)>
{
   /// computes the first element of the formula and returns its value followed with any leftover
   fn interpret_rec(formula: &[State]) -> (Box<(Fn(i64) -> i64)>, &[State])
   {
      match formula
      {
         [formula.., State::Variable] =>
         {
            let f = Box::new(|variable| variable);
            (f, formula)
         }
         [formula.., State::O('~')] =>
         {
            let (fx, formula) = interpret_rec(formula);
            let f = Box::new(move |variable| -fx(variable));
            (f, formula)
         }
         [formula.., State::O('+')] =>
         {
            let (fx, formula) = interpret_rec(formula);
            let (fy, formula) = interpret_rec(formula);
            let f = Box::new(move |variable| fx(variable) + fy(variable));
            (f, formula)
         }
         [formula.., State::O('*')] =>
         {
            let (fx, formula) = interpret_rec(formula);
            let (fy, formula) = interpret_rec(formula);
            let f = Box::new(move |variable| fx(variable) * fy(variable));
            (f, formula)
         }
         [.., State::Bit0] | [.., State::Bit1] =>
         {
            let (n, formula) = interpret_bits(formula);
            let f = Box::new(move |_| n);
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
// PRIME

lazy_static! {
   static ref PRIMES: Mutex<PrimeSet> = Mutex::new(PrimeSet::new());
}

/// returns true if a number is eiter one or a prime
fn is_prime(n: i64) -> bool
{
   let n = n.abs() as u64;
   (n == 1) || PRIMES.lock().unwrap().is_prime(n)
}

//-------------------------------------------------------------------------------------------------
// GRAMMAR

/// an implementation of the Kepler grammar
impl Grammar for State
{
   type ScoreType = f64;

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
         State::Expr => vec![vec![State::Variable],
                             vec![State::Number],
                             vec![State::O('+'), State::Expr, State::Expr],
                             vec![State::O('*'), State::Expr, State::Expr]],
         State::Bit => vec![vec![State::Bit0], vec![State::Bit1]],
         State::Number =>
         {
            vec![vec![State::Bit, State::Bit, State::Bit, State::Bit, State::Bit, State::EndBit],
                 vec![State::O('~'),
                      State::Bit,
                      State::Bit,
                      State::Bit,
                      State::Bit,
                      State::Bit,
                      State::EndBit]]
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
            [formula.., State::Variable] => ("x".to_string(), formula),
            [formula.., State::O('~')] =>
            {
               let (x, formula) = to_string_rec(formula);
               let result = format!("-{}", x);
               (result, formula)
            }
            [formula.., State::O('+')] =>
            {
               let (x, formula) = to_string_rec(formula);
               let (y, formula) = to_string_rec(formula);
               let result = format!("{} + {}", x, y);
               (result, formula)
            }
            [formula.., State::O('*')] =>
            {
               let (x, formula) = to_string_rec(formula);
               let (y, formula) = to_string_rec(formula);
               let result = format!("({}) * ({})", x, y);
               (result, formula)
            }
            [.., State::Bit0] | [.., State::Bit1] =>
            {
               let (n, formula) = interpret_bits(formula);
               (n.to_string(), formula)
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
      let polynomial = interpret(formula);
      let mut x = 0;
      let mut previous_y = 0;
      let mut y = polynomial(x);
      while (y != previous_y) && is_prime(y)
      {
         x += 1;
         previous_y = y;
         y = polynomial(x);
      }
      x as f64
   }
}
