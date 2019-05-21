use rand::Rng;
//use rand::FromEntropy; // for random initialisation
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use float_ord::FloatOrd;
use crate::distribution::Distribution;
use crate::distribution;
use crate::grammar::{Grammar, Formula};
use crate::result;
use crate::result::{Result};
use crate::memory::{MemoryTracker, memory_summary};

mod expand;
use expand::expand;
mod no_expand;
use no_expand::{no_expand, compute_balance_factor};
mod random_expand;

//-----------------------------------------------------------------------------
// TYPES

/// several children and their distributions
pub struct Node<Distr: Distribution>
{
   pub distribution: Distr,
   pub children: Box<[Tree<Distr>]>
}

/// either a previously deleted node, a leaf or a node with children
pub enum Tree<Distr: Distribution>
{
   Deleted,
   Leaf,
   KnownLeaf(Box<Distr>),
   Node(Box<Node<Distr>>) // we use a box to reduce the memory footprint of the tree that are not nodes
}

/// represents the output of an expand operation
pub enum ReturnType<Tree>
{
   NewTree(Tree),
   DeleteChild,
   DoNothing
}

//-----------------------------------------------------------------------------
// FUNCTIONS

impl<Distr: Distribution> Tree<Distr>
{
   /// creates a new, empty, tree
   fn new() -> Self
   {
      Tree::Leaf
   }

   /// returns true if a tree has been deleted
   fn is_deleted(&self) -> bool
   {
      match self
      {
         Tree::Deleted => true,
         _ => false
      }
   }
   fn is_unknown_leaf(&self) -> bool
   {
      match self
      {
         Tree::Leaf => true,
         _ => false
      }
   }

   fn distribution(&self) -> &Distr
   {
      match self
      {
         Tree::KnownLeaf(box distr) => distr,
         Tree::Node(box Node { distribution: distr, .. }) => distr,
         _ => panic!("tried to get distribution from Tree that does not have one.")
      }
   }
}

/// selects the node with the maximum score (breaks ties at random)
/// leafs having an infinite score, they are taken in priority
/// NOTE: this function could be rewritten in a more efficient way if needed
pub fn best_child<Distr, RNG>(children: &[Tree<Distr>],
                              default_distr: &Distr,
                              mut rng: &mut RNG,
                              available_depth: i64)
                              -> usize
   where Distr: Distribution,
         RNG: Rng
{
   if available_depth <= 0
   {
      // we return the first child which, by convention, should be on the shortest path to a valid formula
      return 0;
   }
   let best_leaf = children.iter()
                           .enumerate()
                           .filter(|(_, tree)| tree.is_unknown_leaf())
                           .max_by_key(|_| rng.gen::<usize>());
   match best_leaf
   {
      Some((i, _)) => i,
      None =>
      {
         let (i, _) =
            children.iter()
                    .enumerate()
                    .filter(|(_, tree)| !tree.is_deleted())
                    .max_by_key(|(_, tree)| FloatOrd(tree.distribution().score(default_distr, &mut rng)))
                    .expect("best_child: tried to find the best child in an empty array.");
         i
      }
   }
}

/// if the result does not modify the tree, we inject the given tree
pub fn new_tree<State: Grammar, Distr: Distribution>(
   result: (ReturnType<Tree<Distr>>, Formula<State>, State::ScoreType),
   tree: Tree<Distr>)
   -> (ReturnType<Tree<Distr>>, Formula<State>, State::ScoreType)
{
   match result
   {
      (ReturnType::DoNothing, formula, score) => (ReturnType::NewTree(tree), formula, score),
      _ => result
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
   let mut tree = Tree::<Distr>::new();
   let mut result = Res::new();

   // searches while there is memory available
   // uses a simple linear model to avoid measuring memory at each iteration
   let mut iteration = 0;
   let mut iteration_previous = 0;
   let mut free_memory_current = memory_tracker.free_memory();
   let mut free_memory_previous = free_memory_current;
   let mut memory_growth = 0.; // by how much does the memory grow per iteration
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

// TODO implement slower memory explore
