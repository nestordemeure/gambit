
// random number generation
use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;

use crate::grammar;
use cons_list::ConsList;
use float_ord::FloatOrd; // to compare floats

//-----------------------------------------------------------------------------
// PRIOR

/// stores information gotten during previous runs
struct Prior
{
   nbVisit: u32,
   nbScore: u32,
   sumScores: f64,
   maxScore: f64
}

impl Prior
{
   /// returns a default, empty, prior
   fn default() -> Prior
   {
      Prior { nbVisit: 0, nbScore: 0, sumScores: 0., maxScore: -std::f64::INFINITY }
   }

   /// adds a score to the prior
   fn update(&mut self, score: Option<f64>)
   {
      self.nbVisit += 1;
      if let Some(score) = score
      {
         self.nbScore += 1;
         self.sumScores += score;
         if score > self.maxScore
         {
            self.maxScore = score;
         }
      }
   }

   /// uses the prior sample a potential score
   fn sample(&self, rng: &mut Xoshiro256Plus) -> f64
   {
      let mean = self.sumScores / (self.nbScore as f64);
      let max = (self.nbScore as f64).ln() * self.maxScore;
      rng.gen_range(mean, max)
   }

   /// gives a score to the node, we will take the node with the maximum score
   fn score(&self, defaultPrior: &Prior, mut rng: &mut Xoshiro256Plus) -> f64
   {
      if self.nbVisit == 0
      {
         std::f64::INFINITY
      }
      else
      {
         match rng.gen_ratio(self.nbScore + 1, self.nbVisit + 2) // laplacian smoothing
         {
            false => -std::f64::INFINITY,
            true if self.nbScore == 0 => defaultPrior.sample(&mut rng),
            true => self.sample(&mut rng)
         }
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
      formula: ConsList<grammar::State>, stack: ConsList<grammar::State>
   },
   Node
   {
      childrensPriors: Vec<Prior>, children: Vec<Tree>
   }
}

/// selects the node with the maximum score
/// breaks ties at random
/// leafs having an infinite score, they are taken in priority
fn bestChild(priors: &[Prior],
             defaultPrior: &Prior,
             mut rng: &mut Xoshiro256Plus,
             availableDepth: i16)
             -> usize
{
   if availableDepth <= 0
   {
      0
   }
   else
   {
      let (index, _) =
         priors.iter()
               .enumerate()
               .max_by_key(|&(_, prior)| (FloatOrd(prior.score(defaultPrior, &mut rng)), rng.gen::<usize>()))
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
fn newTree(result: (ReturnType, Option<f64>), tree: Tree) -> (ReturnType, Option<f64>)
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
// FORMULA

/// adds a vector on top of a conslist
fn concat(head: &[grammar::State], tail: &ConsList<grammar::State>) -> ConsList<grammar::State>
{
   head.iter().fold(tail.clone(), |result, state| result.append(*state))
}

/// reverse a formula into a vector
fn reverse(formula: &ConsList<grammar::State>) -> Vec<grammar::State>
{
   let mut vector: Vec<grammar::State> = formula.iter().cloned().collect();
   vector.reverse();
   vector
}

//-----------------------------------------------------------------------------
// EXPAND

fn expand(tree: &mut Tree,
          rootPrior: &Prior,
          mut rng: &mut Xoshiro256Plus,
          availableDepth: i16)
          -> (ReturnType, Option<f64>)
{
   match tree
   {
      Tree::Node { ref mut childrensPriors, ref mut children } =>
      {
         let indexBestChildren = bestChild(&childrensPriors, &rootPrior, &mut rng, availableDepth);
         let (action, score) = expand(&mut children[indexBestChildren],
                                      &childrensPriors[indexBestChildren],
                                      &mut rng,
                                      availableDepth);
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
               children.remove(indexBestChildren);
               childrensPriors.remove(indexBestChildren);
               (ReturnType::DoNothing, score)
            }
            ReturnType::DoNothing =>
            {
               // we can update the child's prior
               childrensPriors[indexBestChildren].update(score);
               (action, score)
            }
            ReturnType::NewTree(childTree) =>
            {
               // we can replace this child and update its prior
               children[indexBestChildren] = childTree;
               childrensPriors[indexBestChildren].update(score);
               (ReturnType::DoNothing, score)
            }
         }
      }
      Tree::Leaf { formula, stack } if !stack.is_empty() =>
      {
         // non terminal leaf, we expand into a node
         let state = *stack.head().unwrap();
         let stack = stack.tail();
         match grammar::expand(state).as_slice()
         {
            [] =>
            {
               // terminal state
               let formula = formula.append(state);
               let mut newLeaf = Tree::Leaf { formula, stack };
               let result = expand(&mut newLeaf, rootPrior, &mut rng, availableDepth);
               newTree(result, newLeaf)
            }
            [rule] =>
            {
               // single rule, we can focus on it
               let stack = concat(rule, &stack);
               let mut newLeaf = Tree::Leaf { formula: formula.clone(), stack };
               let result = expand(&mut newLeaf, rootPrior, &mut rng, availableDepth);
               newTree(result, newLeaf)
            }
            rules =>
            {
               // non terminal state, we build a node
               let childrensPriors = (0..rules.len()).map(|_| Prior::default()).collect();
               let children = rules.iter()
                                   .map(|rule| concat(rule, &stack))
                                   .map(|stack| Tree::Leaf { formula: formula.clone(), stack })
                                   .collect();
               let mut newNode = Tree::Node { childrensPriors, children };
               let result = expand(&mut newNode, rootPrior, &mut rng, availableDepth - 1);
               newTree(result, newNode)
            }
         }
      }
      Tree::Leaf { formula, stack } =>
      {
         // terminal leaf, we evaluate the formula and backpropagate
         let score = grammar::evaluate(&reverse(&formula));
         (ReturnType::DeleteChild, score)
      }
   }
}

//-----------------------------------------------------------------------------
// SEARCH

/// performs the search for a given number of iterations
/// TODO add arbitrary result
/// TODO add arbitrary grammar
pub fn search(availableDepth: i16, nbIterations: u64) -> f64
{
   let mut rng = Xoshiro256Plus::seed_from_u64(0);
   let mut rootPrior = Prior::default();
   let mut tree = Tree::Leaf { formula: ConsList::new(), stack: ConsList::new().append(grammar::rootState) };
   for iteration in 0..nbIterations
   {
      let (action, score) = expand(&mut tree, &rootPrior, &mut rng, availableDepth);
      rootPrior.update(score);
      // TODO update result
      match action
      {
         ReturnType::NewTree(updatedTree) =>
         {
            tree = updatedTree;
         }
         ReturnType::DeleteChild => break,
         ReturnType::DoNothing => ()
      }
   }
   rootPrior.maxScore
}

// TODO add backpropagation of formula to result