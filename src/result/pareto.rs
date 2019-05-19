use crate::grammar::{Grammar, Formula};
use super::Result;
use linked_list::{LinkedList, Cursor};
use std::fmt;

//-------------------------------------------------------------------------------------------------
// TYPES

/// represents an individual result stored in the pareto front
struct ParetoElement<State: Grammar>
{
   formula: Formula<State>,
   score: f64,
   cost: usize
}

/// stores a pareto front of the results so far
pub struct ParetoFront<State: Grammar>
{
   front: LinkedList<ParetoElement<State>>
}

/// inserts a new element in the pareto front
/// returns true if it is better than the best element so far
fn insert<State: Grammar>(mut front_cursor: Cursor<ParetoElement<State>>,
                          new_element: ParetoElement<State>)
                          -> bool
{
   match front_cursor.peek_next()
   {
      None =>
      {
         front_cursor.insert(new_element);
         true
      }
      Some(ref element) if element.score <= new_element.score && element.cost >= new_element.cost =>
      {
         // we pareto dominate this result
         front_cursor.remove();
         insert(front_cursor, new_element)
      }
      Some(ref element) if element.score < new_element.score =>
      {
         // we are better but more expensive
         front_cursor.insert(new_element);
         true
      }
      Some(ref element) if element.cost > new_element.cost =>
      {
         // we are worst but cheaper
         front_cursor.next();
         insert(front_cursor, new_element);
         false
      }
      _ => false // we are pareto dominated
   }
}

//-------------------------------------------------------------------------------------------------
// TRAIT

/// macro to display a result
impl<State: Grammar> fmt::Display for ParetoFront<State>
{
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
   {
      writeln!(f, "{{").unwrap();
      for element in self.front.iter()
      {
         writeln!(f, "\tscore:{}\tcost:{}\tformula:'{}'", element.score, element.cost, element.formula).unwrap();
      }
      writeln!(f, "}}")
   }
}

impl<State: Grammar> Result<State> for ParetoFront<State>
{
   type ScoreType = f64;

   /// creates an empty result
   fn new() -> ParetoFront<State>
   {
      ParetoFront { front: LinkedList::new() }
   }

   /// returns the best (formula, score) so far
   fn best(&self) -> (Formula<State>, f64)
   {
      match self.front.front()
      {
         None => (Formula::<State>::empty(), std::f64::NEG_INFINITY),
         Some(ParetoElement { formula, score, .. }) => (formula.clone(), *score)
      }
   }

   /// if the result is non dominated by the front so far, we update it
   fn update(&mut self, formula: Formula<State>, score: Self::ScoreType) -> bool
   {
      let cost = formula.cost();
      let new_element = ParetoElement { formula, score, cost };
      insert(self.front.cursor(), new_element)
   }
}