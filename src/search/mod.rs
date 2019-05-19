
// random number generation
//use rand::FromEntropy; // for random initialisation
use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
// float manipulation
use float_ord::FloatOrd;

use crate::distribution::Distribution;
use crate::grammar::{Grammar, Formula};
use crate::result::{Result};
use crate::memory::{MemoryTracker, memory_summary};

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

/// represents the output of an expand operation
enum ReturnType<Tree>
{
   NewTree(Tree),
   DeleteChild,
   DoNothing
}

//-----------------------------------------------------------------------------
// FUNCTIONS

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

/// selects the node with the maximum score (breaks ties at random)
/// leafs having an infinite score, they are taken in priority
/// NOTE: this function could be rewritten in a more efficient way if needed
fn best_child<Distr, RNG>(distributions: &[Distr],
                          default_distr: &Distr,
                          mut rng: &mut RNG,
                          available_depth: i16)
                          -> usize
   where Distr: Distribution,
         RNG: Rng
{
   if available_depth <= 0
   {
      // we return the first child which, by convention, should be on the shortest path to a valid formula
      0
   }
   else
   {
      let (index, _) = distributions.iter()
                                    .enumerate()
                                    .max_by_key(|&(_, distr)| {
                                       (FloatOrd(distr.score(default_distr, &mut rng)), rng.gen::<usize>()) // ties are broken randomly
                                    })
                                    .expect("best_child: tried to find the best child in an empty array.");
      index
   }
}

/// if the result does not modify the tree, we inject the given tree
fn new_tree<State, Distr>(result: (ReturnType<Tree<State, Distr>>, Formula<State>, State::ScoreType),
                          tree: Tree<State, Distr>)
                          -> (ReturnType<Tree<State, Distr>>, Formula<State>, State::ScoreType)
   where State: Grammar,
         Distr: Distribution
{
   match result
   {
      (ReturnType::DoNothing, formula, score) => (ReturnType::NewTree(tree), formula, score),
      _ => result
   }
}

//-----------------------------------------------------------------------------
// EXPAND

/// takes a tree, its prior, a random number generator and the available depth and expand the tree
/// return the result of the expansion as a (ReturnType, formula, Option<score>)
fn expand<State, Distr, RNG>(mut tree: &mut Tree<State, Distr>,
                             distribution_root: &Distr,
                             rng: &mut RNG,
                             available_depth: i16)
                             -> (ReturnType<Tree<State, Distr>>, Formula<State>, State::ScoreType)
   where State: Grammar,
         Distr: Distribution<ScoreType = State::ScoreType>,
         RNG: Rng
{
   match tree
   {
      Tree::Node { ref mut childrens_distributions, ref mut children } =>
      {
         let index_best_child = best_child(childrens_distributions, distribution_root, rng, available_depth);
         let (action, formula, score) = expand(&mut children[index_best_child],
                                               &childrens_distributions[index_best_child],
                                               rng,
                                               available_depth);
         match action
         {
            ReturnType::DeleteChild if children.len() == 1 =>
            {
               // no more child if we remove this child : we can remove this node
               (action, formula, score)
            }
            ReturnType::DeleteChild =>
            {
               // we can remove this child from the node
               children.swap_remove(index_best_child);
               childrens_distributions.swap_remove(index_best_child);
               // save a bit of memory since it matters more than speed
               children.shrink_to_fit();
               childrens_distributions.shrink_to_fit();
               (ReturnType::DoNothing, formula, score)
            }
            ReturnType::DoNothing =>
            {
               // we can update the child's prior
               childrens_distributions[index_best_child].update(score);
               (action, formula, score)
            }
            ReturnType::NewTree(child_tree) =>
            {
               // we can replace this child and update its prior
               children[index_best_child] = child_tree;
               childrens_distributions[index_best_child].update(score);
               (ReturnType::DoNothing, formula, score)
            }
         }
      }
      Tree::Leaf { formula, stack } if !stack.is_empty() =>
      {
         // non terminal leaf, we expand into a node
         let state = stack.pop().unwrap();
         match state.expand().as_slice()
         {
            [] =>
            {
               // terminal state
               formula.push(state);
               expand(&mut tree, distribution_root, rng, available_depth)
            }
            [rule] =>
            {
               // single rule, we can focus on it
               stack.extend(rule);
               expand(&mut tree, distribution_root, rng, available_depth)
            }
            rules =>
            {
               // non terminal state, we build a node
               let childrens_distributions = (0..rules.len()).map(|_| Distr::new()).collect();
               let children = rules.iter()
                                   .map(|rule| stack.iter().chain(rule).cloned().collect())
                                   .map(|stack| Tree::Leaf { formula: formula.clone(), stack })
                                   .collect();
               let mut new_node = Tree::Node { childrens_distributions, children };
               let result = expand(&mut new_node, distribution_root, rng, available_depth - 1);
               new_tree(result, new_node)
            }
         }
      }
      Tree::Leaf { formula, .. } =>
      {
         // terminal leaf, we evaluate the formula and backpropagate
         let score = formula.evaluate();
         (ReturnType::DeleteChild, formula.clone(), score)
      }
   }
}

//-----------------------------------------------------------------------------
// SEARCH

/// performs the search for a given number of iterations
pub fn search<State, Distr, Res>(available_depth: i16, nb_iterations: u64) -> Res
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
      let (action, formula, score) = expand(&mut tree, &distribution_root, &mut rng, available_depth);
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

/*
   TODO implement memory limited explore

   we can measure memory use at regular intervals to stop consumming it when we are a few hundreds of Mo before the end of the RAM
   it does not matter wether we are the one using the memory we just want to avoid crashing the computeur

   let sys = System::new();
   match sys.memory()
   {
      Ok(mem) => println!("\nMemory: {} used / {} ({} bytes)",
                          mem.total - mem.free,
                          mem.total,
                          (mem.total - mem.free).as_usize()),
      Err(x) => println!("\nMemory: error: {}", x)
   }
   // 1Go = 1000000000 bytes
*/
