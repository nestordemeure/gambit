mod tree;
mod expand;
mod no_expand;
mod random_expand;

use rand::FromEntropy; // for random initialisation
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use crate::distribution::Distribution;
use crate::grammar::{Grammar, Formula};
use crate::result::Result;
use crate::memory::{MemoryTracker, memory_summary, memory_used};
use tree::*;
pub use tree::{Node, Tree};
use expand::expand;
use no_expand::*;

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

   //let mut rng = Xoshiro256Plus::seed_from_u64(0);
   let mut rng = Xoshiro256Plus::from_entropy();
   let mut tree = Tree::<Distr>::new();
   let mut result = Res::new();
   for _ in 0..nb_iterations
   {
      let formula = Formula::empty();
      let stack = vec![State::root_state()];
      let (action, formula, score) = expand(&mut tree, formula, stack, &mut rng, available_depth as i64);
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
      search::<State, crate::distribution::Optional<Distr>, crate::result::Optional<Res>>(available_depth,
                                                                                          nb_iterations);
   result.get_result()
}

/// performs the search for a given number of iterations
/// NOTE: change searching strategy once the available RAM drops below the given level
///       this function can run forever without crashing the computeur
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
   let mut tree = Tree::<Distr>::new();
   let mut result = Res::new();

   // searches while there is memory available
   // uses a simple linear model to avoid measuring memory at each iteration
   let mut iteration = 0;
   let mut iteration_previous = 0;
   let mut free_memory_current = memory_tracker.free_memory();
   let mut free_memory_previous = free_memory_current;
   let mut memory_growth = 0.; // by how much does the memory growth per iteration
   let step_size = 1000; // refresh memory measure every step_size iterations
   while (iteration < nb_iterations) && (free_memory_current > free_memory_size)
   {
      let formula = Formula::empty();
      let stack = vec![State::root_state()];
      let (action, formula, score) = expand(&mut tree, formula, stack, &mut rng, available_depth as i64);
      result.update(formula, score);
      match action
      {
         ReturnType::NewTree(updated_tree) => tree = updated_tree,
         ReturnType::DeleteChild => break,
         ReturnType::DoNothing => ()
      }
      // updates iteration and free_memory_current
      iteration += 1;
      free_memory_current =
         free_memory_previous + (((iteration - iteration_previous) as f64) * memory_growth) as usize;
      if ((iteration_previous + iteration) % step_size == 0) || (free_memory_current < free_memory_size)
      {
         free_memory_current = memory_tracker.free_memory();
         memory_growth =
            (free_memory_current - free_memory_previous) as f64 / (iteration - iteration_previous) as f64;
         iteration_previous = iteration;
         free_memory_previous = free_memory_current;
      }
   }

   // searches that avoids growing the memory
   let balance_factor = tree.balance_factor(iteration);
   println!("memory limits reached at iteration n°{}, balance factor is {}, free memory is {}Mo",
            iteration, balance_factor, free_memory_current);
   for _ in iteration..nb_iterations
   {
      let formula = Formula::empty();
      let stack = vec![State::root_state()];
      let (action, formula, score) =
         no_expand(&mut tree, formula, stack, &mut rng, available_depth as i64, balance_factor);
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
pub fn memory_limited_search_optional<State, Distr, Res>(available_depth: usize,
                                                         nb_iterations: usize,
                                                         free_memory_size: usize)
                                                         -> Res
   where State: Grammar<ScoreType = Option<Res::ScoreType>>,
         Distr: Distribution<ScoreType = Res::ScoreType>,
         Res: Result<State>,
         Res::ScoreType: Copy + std::fmt::Debug
{
   let result = memory_limited_search::<State,
                                      crate::distribution::Optional<Distr>,
                                      crate::result::Optional<Res>>(available_depth,
                                                                    nb_iterations,
                                                                    free_memory_size);
   result.get_result()
}

/// performs the search for a given number of iterations
/// NOTE: change searching strategy once the RAM drops below the given level
/// TODO this fucntion is a work in progress
pub fn nested_search<State, Distr, Res>(available_depth: usize,
                                        nb_iterations: usize,
                                        free_memory_size: usize)
                                        -> Res
   where State: Grammar,
         Distr: Distribution<ScoreType = State::ScoreType>,
         Res: Result<State, ScoreType = State::ScoreType>
{
   let memory_tracker = MemoryTracker::new();

   let mut rng = Xoshiro256Plus::seed_from_u64(0); //from_entropy();
   let mut tree = Tree::<Distr>::new();
   let mut result = Res::new();

   // searches while there is memory available
   // uses a simple linear model to avoid measuring memory at each iteration
   let free_memory_base = memory_tracker.free_memory();
   let mut iteration_previous = 0;
   let mut free_memory_current = free_memory_base - memory_used(&tree);
   let mut free_memory_previous = free_memory_current;
   let mut memory_growth = 0.; // by how much does the memory grow per iteration
   let step_size = 10000; // refresh memory measure every step_size iterations
   for iteration in 0..nb_iterations
   {
      let formula = Formula::empty();
      let stack = vec![State::root_state()];
      let (action, formula, score) = expand(&mut tree, formula, stack, &mut rng, available_depth as i64);
      result.update(formula, score);
      match action
      {
         ReturnType::NewTree(updated_tree) => tree = updated_tree,
         ReturnType::DeleteChild => break,
         ReturnType::DoNothing => ()
      }
      // updates free_memory_current
      free_memory_current =
         free_memory_previous + (((iteration - iteration_previous) as f64) * memory_growth) as usize;
      if ((iteration_previous + iteration) % step_size == 0) || (free_memory_current < free_memory_size)
      {
         // has a direct computation of the memory used is an expensive operation, we need a large step size to amortize the cost
         free_memory_current = free_memory_base - memory_used(&tree);
         if free_memory_current < free_memory_size
         {
            println!("iteration {}, pruning tree", iteration);
            tree.prune();
            // we cannot use RAM usage here as it is not refreshed quicly enough
            free_memory_current = free_memory_base - memory_used(&tree);
         }
         memory_growth =
            (free_memory_current - free_memory_previous) as f64 / (iteration - iteration_previous) as f64;
         iteration_previous = iteration;
         free_memory_previous = free_memory_current;
      }
   }

   memory_summary(&tree);
   memory_tracker.print_memory_usage();
   result
}

// TODO implement slower memory explore

// TODO the evolutionnary strategy crate has a nice idea :
// they have a search type and an into_iter traits on it (producing a sequence of costs)
// with that, the user can implement any stopping criteria he wishes for
// and, if he wants to, monitor the evolution of the cost
