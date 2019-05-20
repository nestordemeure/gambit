use std::collections::HashMap;
use super::Grammar;

// TODO derive priority table that gives distance to exit for all symbols in a grammar
// and thus can be used to select the shortest rule

/*
   i want to know the distance from each rule of each state to a terminal
   i could do it with several pass
   i have a dictionnary of rules and a dictionnary of costs

   given a state, i want to know the time from each rule to a terminal
   it is the sum of the time from their states to a terminal

   we could have a set of unknown types and a dictionary of distances
   we pop from the set and, if it not known
*/

fn make_distance_table<State: Grammar>() -> HashMap<State, i16>
{
   let mut unknown_states: Vec<State> = vec![Grammar::root_state()];
   let mut distance_of_state = HashMap::new();

   while !unknown_states.is_empty()
   {
      let new_unknown_states = vec![];
      for state in unknown_states
      {
         if !distance_of_state.contains_key(&state)
         {
            let rules = state.expand();
            if rules.is_empty()
            {
               // terminal state
               distance_of_state.insert(state, 0);
            }
            else
            {
               let rules = rules.iter().filter(|rule| rule.iter().all(|s| distance_of_state.contains_key(s)));
            }
         }
      }
      unknown_states = new_unknown_states;
   }

   distance_of_state
}
