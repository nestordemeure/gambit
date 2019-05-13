
// random number generation
//use rand::FromEntropy; // for random initialisation
use rand::Rng; // basic operations
use rand::SeedableRng; // to get reproducible runs
use rand_xoshiro::Xoshiro256Plus; // choice of generator
                                  // float manipulation
use float_ord::FloatOrd;
use std::f64;
// memory measure
use systemstat::{Platform, System}; // to measure memory use
                                    // my modules
use crate::grammar;

//-----------------------------------------------------------------------------
// PRIOR

/// stores information gotten during previous runs
struct Prior
{
   nb_visit: u64,
   nb_score: u64,
   sum_scores: f64,
   max_score: f64
}

impl Prior
{
   /// returns a default, empty, prior
   fn default() -> Prior
   {
      Prior { nb_visit: 0, nb_score: 0, sum_scores: 0., max_score: -std::f64::INFINITY }
   }

   /// adds a score to the prior
   fn update(&mut self, score_opt: Option<f64>)
   {
      self.nb_visit += 1;
      if let Some(score) = score_opt
      {
         self.nb_score += 1;
         self.sum_scores += score;
         if score > self.max_score
         {
            self.max_score = score;
         }
      }
   }

   /// uses the prior sample a potential score
   fn sample(&self, rng: &mut Xoshiro256Plus) -> f64
   {
      let exp1 = (1.0 as f64).exp();
      let k = self.nb_score as f64;
      let mean = self.sum_scores / k;
      rng.gen_range(mean, (k + exp1).ln() * (self.max_score) as f64)
   }

   /// gives a score to the node, we will take the node with the maximum score
   fn score(&self, default_prior: &Prior, mut rng: &mut Xoshiro256Plus) -> f64
   {
      if self.nb_visit == 0
      {
         return std::f64::INFINITY;
      }
      match rng.gen_ratio((self.nb_score + 1) as u32, (self.nb_visit + 2) as u32) // laplacian smoothing
      {
         false => -std::f64::INFINITY,
         true if self.nb_score == 0 => default_prior.sample(&mut rng),
         true => self.sample(&mut rng)
      }
   }
}

//-----------------------------------------------------------------------------
// TREE

/// either a leaf with a current formula or a node with several children and their prior
enum Tree
{
   Leaf
   {
      formula: Vec<grammar::State>, stack: Vec<grammar::State>
   },
   Node
   {
      childrens_priors: Vec<Prior>,
      children: Vec<Tree> // Box<[Prior]>
   }
}

/// selects the node with the maximum score
/// breaks ties at random
/// leafs having an infinite score, they are taken in priority
fn best_child(priors: &[Prior],
              default_prior: &Prior,
              mut rng: &mut Xoshiro256Plus,
              available_depth: i16)
              -> usize
{
   if available_depth <= 0
   {
      0
   }
   else
   {
      let (index, _) =
         priors.iter()
               .enumerate()
               .max_by_key(|&(_, prior)| (FloatOrd(prior.score(default_prior, &mut rng)), rng.gen::<usize>()))
               .expect("Tried to find the best child in an empty array.");
      index
   }
}

//-----------------------------------------------------------------------------
// RETURN

/// represents the output of an expand operation
enum ReturnType
{
   NewTree(Tree),
   DeleteChild,
   DoNothing
}

/// if the result does not have a tree, we inject the given tree
fn new_tree(result: (ReturnType, Option<f64>), tree: Tree) -> (ReturnType, Option<f64>)
{
   match result
   {
      (ReturnType::DoNothing, score) => (ReturnType::NewTree(tree), score),
      _ => result
   }
}

// we might be able to accomplish the needed action as we detect it instead of checking a ReturnType
// to do so we would need to pass the father node to its child or at least the index of the child and its vectors

//-----------------------------------------------------------------------------
// EXPAND

fn expand(mut tree: &mut Tree,
          prior_root: &Prior,
          mut rng: &mut Xoshiro256Plus,
          available_depth: i16)
          -> (ReturnType, Option<f64>)
{
   match tree
   {
      Tree::Node { ref mut childrens_priors, ref mut children } =>
      {
         let index_best_child = best_child(&childrens_priors, &prior_root, &mut rng, available_depth);
         let (action, score) = expand(&mut children[index_best_child],
                                      &childrens_priors[index_best_child],
                                      &mut rng,
                                      available_depth);
         match action
         {
            ReturnType::DeleteChild if children.len() == 1 =>
            {
               // no more child if we remove this child : we can remove this node
               (ReturnType::DeleteChild, score)
            }
            ReturnType::DeleteChild =>
            {
               // we can remove this child from the node
               children.swap_remove(index_best_child);
               childrens_priors.swap_remove(index_best_child);
               (ReturnType::DoNothing, score)
            }
            ReturnType::DoNothing =>
            {
               // we can update the child's prior
               childrens_priors[index_best_child].update(score);
               (action, score)
            }
            ReturnType::NewTree(child_tree) =>
            {
               // we can replace this child and update its prior
               children[index_best_child] = child_tree;
               childrens_priors[index_best_child].update(score);
               (ReturnType::DoNothing, score)
            }
         }
      }
      Tree::Leaf { formula, stack } if !stack.is_empty() =>
      {
         // non terminal leaf, we expand into a node
         let state = stack.pop().unwrap();
         match grammar::expand(state).as_slice()
         {
            [] =>
            {
               // terminal state
               formula.push(state);
               expand(&mut tree, prior_root, &mut rng, available_depth)
            }
            [rule] =>
            {
               // single rule, we can focus on it
               stack.extend(rule);
               expand(&mut tree, prior_root, &mut rng, available_depth)
            }
            rules =>
            {
               // non terminal state, we build a node
               let childrens_priors = (0..rules.len()).map(|_| Prior::default()).collect();
               let children = rules.iter()
                                   .map(|rule| stack.iter().chain(rule).cloned().collect())
                                   .map(|stack| Tree::Leaf { formula: formula.clone(), stack })
                                   .collect();
               let mut new_node = Tree::Node { childrens_priors, children };
               let result = expand(&mut new_node, prior_root, &mut rng, available_depth - 1);
               new_tree(result, new_node)
            }
         }
      }
      Tree::Leaf { formula, .. } =>
      {
         // terminal leaf, we evaluate the formula and backpropagate
         let score = grammar::evaluate(&formula);
         (ReturnType::DeleteChild, score)
      }
   }
}

//-----------------------------------------------------------------------------
// SEARCH

/// returns the memory use in bytes
fn memory_usage<P>(system: &P) -> usize
   where P: Platform
{
   match system.memory()
   {
      Ok(mem) => (mem.total - mem.free).as_usize(),
      Err(x) => panic!("Unable to measure memory: {}", x)
   }
}

/// performs the search for a given number of iterations
/// TODO add arbitrary result
/// TODO add arbitrary grammar
pub fn search(available_depth: i16, nb_iterations: u64) -> f64
{
   // memory use for benchmarking purposes
   let system = System::new();
   let memory_before = memory_usage(&system);

   let mut rng = Xoshiro256Plus::seed_from_u64(0); //from_entropy();
   let mut prior_root = Prior::default();
   let mut tree = Tree::Leaf { formula: vec![], stack: vec![grammar::ROOTSTATE] };
   for _ in 0..nb_iterations
   {
      let (action, score) = expand(&mut tree, &prior_root, &mut rng, available_depth);
      prior_root.update(score);
      // TODO update result
      match action
      {
         ReturnType::NewTree(updated_tree) =>
         {
            tree = updated_tree;
         }
         ReturnType::DeleteChild => break,
         ReturnType::DoNothing => ()
      }
   }

   // display the memory used by the tree
   // (under the assumption that there is no other memory consummer on the computeur)
   let memory_after = memory_usage(&system);
   println!("memory consumption: {} Mo", (memory_after - memory_before) / 1_000_000);

   prior_root.max_score
}

/*
   100_000 iterations what is the memory usage ?
   baseline with conslist : 660Mo
   with vects : 420Mo
   (measures with seed=0 to help with reproducibility)
*/

/*
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