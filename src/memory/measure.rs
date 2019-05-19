use crate::distribution::Distribution;
use crate::grammar::Grammar;
use crate::search::Tree;
use std::mem::size_of;

/// counts the number of (nodes, distr, statesformula, statesstack) in a tree
/// NOTE: it DOES take into account uninitialized elements in the capacity of the vectors
fn count_elements<State, Distr>(tree: &Tree<State, Distr>) -> (usize, usize, usize, usize)
   where State: Grammar,
         Distr: Distribution
{
   match tree
   {
      Tree::Leaf { formula, stack } => (1, 0, formula.capacity(), stack.capacity()),
      Tree::Node { children, childrens_distributions } =>
      {
         let nb_distr = childrens_distributions.capacity();
         let nb_nodes = 1 + children.capacity() - children.len();
         children.iter()
                 .map(|child| count_elements(child))
                 .fold((nb_nodes, nb_distr, 0, 0), |(na, da, fa, sa), (n, d, f, s)| {
                    (na + n, da + d, fa + f, sa + s)
                 })
      }
   }
}

/// prints a summay of the memory use of the given tree
/// the measure seem to be optimistic by roughly 30%, maybe because of fragmentation ?
pub fn memory_summary<State, Distr>(tree: &Tree<State, Distr>)
   where State: Grammar,
         Distr: Distribution
{
   let node_size = size_of::<Tree<State, Distr>>();
   let state_size = size_of::<State>();
   let distr_size = size_of::<Distr>();

   let (nb_nodes, nb_distr, nb_states_formula, nb_states_stack) = count_elements(tree);
   let memory_nodes = nb_nodes * node_size;
   let memory_distr = nb_distr * distr_size;
   let memory_formula = nb_states_formula * state_size;
   let memory_stack = nb_states_stack * state_size;
   let total_memory_use = memory_nodes + memory_distr + memory_formula + memory_stack;

   println!("Memory use: {} Mo", total_memory_use / 1_000_000);
   println!("- nodes : {} Mo ({} x {} bytes)", memory_nodes / 1_000_000, nb_nodes, node_size);
   println!("- distributions : {} Mo ({} x {} bytes)", memory_distr / 1_000_000, nb_distr, distr_size);
   println!("- states formula : {} Mo ({} x {} bytes)",
            memory_formula / 1_000_000,
            nb_states_formula,
            state_size);
   println!("- states stack : {} Mo ({} x {} bytes)", memory_stack / 1_000_000, nb_states_stack, state_size);
}

/*
   ways to reduce memory use :

   use an [enum-vec](https://github.com/Badel2/enum_vec) for the formula and stack
   reducing their memory use

   put states of formula inside nodes
   (a test showed that it does not increase the memory use fo the nodes)

   TODO
   do not store formula and stack
   instead rebuild them from scratch at each run
   to know which son is which, do not delete sons, thus keeping the ordering (it also lets us use Box<[T]>)
   one can use a tombstone to indicate wether a node has been deleted (or a special value in the prior)
   => this would be a direct example of sacrificing performances for memory use
   => but, if performance becomes a worry (over memory), removing deletions would make parelelism easier anyway
*/