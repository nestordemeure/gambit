//use rand::FromEntropy; // for random initialisation
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use crate::distribution::Distribution;
use crate::distribution;
use crate::grammar::{Grammar, Formula};
use crate::result;
use crate::result::{Result};
use crate::memory::{MemoryTracker, memory_summary};

mod expand;
use expand::{expand, ReturnType};
mod no_expand;
use no_expand::{no_expand, compute_balance_factor};

//-----------------------------------------------------------------------------
// TREE

/// either a leaf with a current formula or a node with several children and their prior
pub enum Tree<State, Distr>
   where State: Grammar,
         Distr: Distribution
{
   Leaf
   {
      formula: Formula<State>, stack: Vec<State>
   },
   Node
   {
      children: Vec<Tree<State, Distr>>, childrens_distributions: Vec<Distr>
   }
}

impl<State, Distr> Tree<State, Distr>
   where State: Grammar,
         Distr: Distribution
{
   /// creates a root tree
   fn root() -> Tree<State, Distr>
   {
      Tree::Leaf { formula: Formula::<State>::empty(), stack: vec![State::root_state()] }
   }
}

//-----------------------------------------------------------------------------
// SEARCH

/// performs the search for a given number of iterations
/// WARNING: this function is memory hungry and could fill the RAM
pub fn search<State, Distr, Res>(available_depth: usize, nb_iterations: usize) -> Res
   where State: Grammar,
         Distr: Distribution<ScoreType = State::ScoreType>,
         Res: Result<State, ScoreType = State::ScoreType>
{
   let memory_tracker = MemoryTracker::new();

   let mut rng = Xoshiro256Plus::seed_from_u64(0); //from_entropy();
   let mut distribution_root = Distr::new();
   let mut tree = Tree::<State, Distr>::root();
   let mut result = Res::new();
   for _ in 0..nb_iterations
   {
      let (action, formula, score) = expand(&mut tree, &distribution_root, &mut rng, available_depth as i64);
      distribution_root.update(score);
      result.update(formula, score);
      match action
      {
         ReturnType::NewTree(updated_tree) => tree = updated_tree,
         ReturnType::DeleteChild => break,
         ReturnType::DoNothing => ()
      }
   }

   memory_summary(&tree);
   memory_tracker.print_memory_usage();
   result
}

/// performs the search for a given number of iterations
/// NOTE: this version is suitable for a grammar that returns an Option<T> score
/// WARNING: this function is memory hungry and could fill the RAM
pub fn search_optional<State, Distr, Res>(available_depth: usize, nb_iterations: usize) -> Res
   where State: Grammar<ScoreType = Option<Res::ScoreType>>,
         Distr: Distribution<ScoreType = Res::ScoreType>,
         Res: Result<State>,
         Res::ScoreType: Copy + std::fmt::Debug
{
   let result =
      search::<State, distribution::Optional<Distr>, result::Optional<Res>>(available_depth, nb_iterations);
   result.get_result()
}

/// performs the search for a given number of iterations
/// WARNING: this function is memory hungry and could fill the RAM
pub fn memory_limited_search<State, Distr, Res>(available_depth: usize,
                                                nb_iterations: usize,
                                                free_memory_size: usize)
                                                -> Res
   where State: Grammar,
         Distr: Distribution<ScoreType = State::ScoreType>,
         Res: Result<State, ScoreType = State::ScoreType>
{
   let memory_tracker = MemoryTracker::new();

   let mut rng = Xoshiro256Plus::seed_from_u64(0); //from_entropy();
   let mut distribution_root = Distr::new();
   let mut tree = Tree::<State, Distr>::root();
   let mut result = Res::new();

   // searches while there is memory available
   let mut iteration = 0;
   let mut iteration_previous = 0;
   let mut free_memory_current = memory_tracker.free_memory();
   let mut free_memory_previous = free_memory_current;
   let mut memory_growth = 0.;
   let step_size = 1000;
   while (iteration < nb_iterations) && (free_memory_current > free_memory_size)
   {
      let (action, formula, score) = expand(&mut tree, &distribution_root, &mut rng, available_depth as i64);
      distribution_root.update(score);
      result.update(formula, score);
      match action
      {
         ReturnType::NewTree(updated_tree) => tree = updated_tree,
         ReturnType::DeleteChild => break,
         ReturnType::DoNothing => ()
      }
      // updates iteration and free_memory_current (using a simple linear model)
      iteration += 1;
      free_memory_current =
         free_memory_previous + (((iteration - iteration_previous) as f64) * memory_growth) as usize;
      if ((iteration_previous + iteration) % step_size == 0) || (free_memory_current > free_memory_size)
      {
         free_memory_current = memory_tracker.free_memory();
         memory_growth =
            (free_memory_current - free_memory_previous) as f64 / (iteration - iteration_previous) as f64;
         iteration_previous = iteration;
         free_memory_previous = free_memory_current;
      }
   }

   // searches that avoids growing the memory
   let balance_factor = compute_balance_factor(&tree, iteration);
   println!("memory limits reached at iteration n°{}, balance factor is {}, free memory is {}Mo", iteration, balance_factor, free_memory_current);
   for _ in iteration..nb_iterations
   {
      let (action, formula, score) =
         no_expand(&mut tree, &distribution_root, &mut rng, available_depth as i64, balance_factor);
      distribution_root.update(score);
      result.update(formula, score);
      match action
      {
         ReturnType::NewTree(updated_tree) => tree = updated_tree,
         ReturnType::DeleteChild => break,
         ReturnType::DoNothing => ()
      }
   }

   memory_summary(&tree);
   memory_tracker.print_memory_usage();
   result
}

// TODO implement slower memory explore