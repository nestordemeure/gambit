use crate::grammar::Grammar;
use super::Result;
use linked_list::{LinkedList, Cursor};

//-------------------------------------------------------------------------------------------------
// TYPES

/// represents an individual result stored in the pareto front
struct ParetoElement<State>
{
   formula: Vec<State>,
   score: f64,
   cost: usize
}

/// stores a pareto front of the results so far
pub struct ParetoFront<State>
{
   front: LinkedList<ParetoElement<State>>
}

//-------------------------------------------------------------------------------------------------
// TRAIT

impl<State:Grammar> Result<State> for ParetoFront<State>
{
   /// creates an empty result
   fn new() -> ParetoFront<State>
   {
      ParetoFront { front: LinkedList::new() }
   }

   /// returns the best formula, score so far
   fn best(&self) -> (Vec<State>, f64)
   {
      match self.front.front()
      {
         None => (vec![], std::f64::NEG_INFINITY),
         Some(ParetoElement{formula, score, ..}) => (formula.to_vec(), *score)
      }
   }

   /// if the result is non dominated by the front so far, we update it
   fn update(&mut self, formula: Vec<State>, score: f64)
   {
      let cost = formula.len(); // TODO we need user-defined cost
      let new_element = ParetoElement { formula, score, cost };
      
      /// inserts a new element in the pareto front
      fn insert<State>(mut front_cursor: Cursor<ParetoElement<State>>, new_element: ParetoElement<State>) 
      {
         match front_cursor.peek_next()
         {
            None => front_cursor.insert(new_element),
            Some(ref element) if element.score <= new_element.score && element.cost >= new_element.cost => 
            {
               // we pareto dominate this result
               front_cursor.remove();
               insert(front_cursor, new_element)
            },
            Some(ref element) if element.score < new_element.score => 
            {
               // we are better but more expensive
               front_cursor.insert(new_element)
            },
            Some(ref element) if element.cost > new_element.cost =>
            {
               // we are worst but cheaper
               front_cursor.next();
               insert(front_cursor, new_element)
            },
            _ => () // we are pareto dominated
         }
      }
      
      insert(self.front.cursor(), new_element);
   }
}