use rand::Rng;
use float_ord::FloatOrd;
use crate::distribution::Distribution;
use crate::grammar::{Grammar, Formula};
use super::Tree;

//-----------------------------------------------------------------------------
// FUNCTIONS

/// represents the output of an expand operation
pub enum ReturnType<Tree>
{
   NewTree(Tree),
   DeleteChild,
   DoNothing
}

/// selects the node with the maximum score (breaks ties at random)
/// leafs having an infinite score, they are taken in priority
/// NOTE: this function could be rewritten in a more efficient way if needed
pub fn best_child<Distr, RNG>(distributions: &[Distr],
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
pub fn new_tree<State, Distr>(result: (ReturnType<Tree<State, Distr>>, Formula<State>, State::ScoreType),
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
pub fn expand<State, Distr, RNG>(mut tree: &mut Tree<State, Distr>,
                                 distribution_root: &Distr,
                                 rng: &mut RNG,
                                 available_depth: i64)
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