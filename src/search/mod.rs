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
use crate::memory::{MemoryTracker, memory_summary, memory_used};
use std::mem::discriminant;

mod expand;
use expand::expand;
mod no_expand;
use no_expand::{no_expand, compute_balance_factor};
mod random_expand;

//-----------------------------------------------------------------------------
// TYPES

/// represents a path among the infinite formula that can be written with the grammar
/// NOTE: uses box in order to minimize the memory footprint of this type
pub enum Tree<Distr: Distribution>
{
   Deleted,                // previously deleted node
   Leaf,                   // unexplored leaf
   KnownLeaf(Box<Distr>),  // previously explored leaf (used in memory scarce mode)
   Node(Box<Node<Distr>>)  // node with several children
}

/// encapsulate a distribution and several children
pub struct Node<Distr: Distribution>
{
   pub distribution: Distr, // the distribution of the reward coming from this node
   pub children: Box<[Tree<Distr>]>  // the children of this node
}

/// represents the action that should be done now that we have expanded the tree
pub enum ReturnType<Tree>
{
   NewTree(Tree), // replace the tree with this new tree
   DeleteChild,   // delete the tree
   DoNothing      // keep everything in place
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

   /// gets the distribution from the tree
   /// WARNING: panics if it is not possible to get a distribution from this tree
   fn distribution(&self) -> &Distr
   {
      match self
      {
         Tree::KnownLeaf(box distr) => distr,
         Tree::Node(box Node { distribution: distr, .. }) => distr,
         _ => panic!("distribution: tried to get distribution from a tree that does not have one.")
      }
   }
   /// returns true if the tree is a leaf
   fn is_unknown_leaf(&self) -> bool
   {
      match self
      {
         Tree::Leaf => true,
         _ => false
      }
   }
   /// returns true if the tree is deleted
   fn is_deleted(&self) -> bool
   {
      match self
      {
         Tree::Deleted => true,
         _ => false
      }
   }
   /// returns true if the node type has a distribution
   fn has_distribution(&self) -> bool
   {
      match self
      {
         Tree::KnownLeaf(_) | Tree::Node(_) => true,
         _ => false
      }
   }
}

/// selects the node with the maximum score
/// leafs having an infinite score, they are taken in priority
fn best_child<Distr, RNG>(children: &[Tree<Distr>],
                          distribution_father: &Distr,
                          mut rng: &mut RNG,
                          available_depth: i64)
                          -> usize
   where Distr: Distribution,
         RNG: Rng
{
   // we return the first child which, by convention, should be on the shortest path to a valid formula
   if available_depth <= 0
   {
      return 0;
   }
   // if there is a leaf, return on leaf at random
   let leaf_index = children.iter()
                            .enumerate()
                            .filter(|&(_, tree)| discriminant(tree) == discriminant(&Tree::Leaf))
                            .max_by_key(|_| rng.gen::<usize>()) // choose one at random
                            .map(|(i, _)| i);
   match leaf_index
   {
      Some(index) => index,
      None =>
      {
         // if there is a children, returns the children with the maximum score
         children.iter()
                 .enumerate()
                 .filter(|&(_, child)| discriminant(child) != discriminant(&Tree::Deleted))
                 .max_by_key(|(_, child)| FloatOrd(child.distribution().score(distribution_father, &mut rng)))
                 .map(|(i, _)| i)
                 .expect("best_child: tried to find the best child in an empty array.")
      }
   }
}

/// injects the given tree in the result unless it suggest dropping
fn new_tree<State: Grammar, Distr: Distribution>(
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

/// prunes all child but one
fn prune<Distr: Distribution>(tree: &mut Tree<Distr>)
{
   if let Tree::Node(box Node { children, .. }) = tree
   {
      let max_visit: Vec<usize> =
         children.iter().enumerate().filter(|(_, child)| child.has_distribution()).map(|(i, _)| i).collect();
      match max_visit.as_slice()
      {
         [] => panic!("tried to prune a tree with zero children"),
         [index] => prune(&mut children[*index]),
         _ =>
         {
            let best_index =
               *max_visit.iter().max_by_key(|&&i| children[i].distribution().nb_visit()).unwrap();
            for index in max_visit
            {
               if index != best_index
               {
                  children[index] = Tree::Deleted;
               }
            }
         }
      }
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
/// NOTE: change searching strategy once the RAM drops below the given level
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

/// performs the search for a given number of iterations
/// NOTE: change searching strategy once the RAM drops below the given level
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
            prune(&mut tree);
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
// TODO implement prune on memory limit
// TODO put tree in a separate file
