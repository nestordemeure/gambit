use rand::Rng;
use crate::tools::lne;
use crate::distribution::Distribution;
use crate::grammar::{Grammar, Formula};
use super::Tree;
use super::random_expand::random_expand;
use super::expand::{ReturnType, best_child, expand};

//-----------------------------------------------------------------------------
// FUNCTION

/// computes the mean length of a branch in the tree
fn mean_branch_length<State, Distr>(tree: &Tree<State, Distr>) -> f64
   where State: Grammar,
         Distr: Distribution<ScoreType = State::ScoreType>
{
   /// computes (number of leafs in the tree, the sum of their length)
   fn length<State, Distr>(tree: &Tree<State, Distr>) -> (usize, usize)
      where State: Grammar,
            Distr: Distribution<ScoreType = State::ScoreType>
   {
      match tree
      {
         Tree::Leaf { .. } => (1, 0),
         Tree::Node { children, .. } => children.iter().fold((0, 0), |(na, ta), child| {
                                                          let (n, t) = length(child);
                                                          (na + n, ta + t + na)
                                                       })
      }
   }
   let (nb_leafs, total_length) = length(tree);
   (nb_leafs as f64) / (total_length as f64)
}

/// computes the balance factor of the tree
/// the larger it is, the more unbalanced the tree is
/// NOTE: we are modeling the growth of the tree with the formula:
/// balance_factor * lne(nb_visit) = mean_formula_length
pub fn compute_balance_factor<State, Distr>(tree: &Tree<State, Distr>, nb_visit: usize) -> f64
   where State: Grammar,
         Distr: Distribution<ScoreType = State::ScoreType>
{
   let length = mean_branch_length(tree);
   let theorical_length = lne(nb_visit as f64); // mean length in a perfectly balanced tree
   length / theorical_length
}

/// tries to predict the mean length of a formula in the tree
/// given a balance factor and a number of visits
/// NOTE: we are modeling the growth of the tree (and all its subtrees) with the formula:
/// balance_factor * lne(nb_visit) = mean_formula_length
fn expected_formula_length(balance_factor: f64, nb_visit: u64) -> i64
{
   (lne(nb_visit as f64) * balance_factor) as i64
}

//-----------------------------------------------------------------------------
// EXPAND

/// takes a tree, its prior, a random number generator and the available depth and expand the tree
/// return the result of the expansion as a (ReturnType, formula, Option<score>)
/// NOTE: this function will not grow the tree, instead it will only update priors
pub fn no_expand<State, Distr, RNG>(mut tree: &mut Tree<State, Distr>,
                                    distribution_root: &Distr,
                                    rng: &mut RNG,
                                    available_depth: i64,
                                    balance_factor: f64)
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
         let (action, formula, score) = no_expand(&mut children[index_best_child],
                                                  &childrens_distributions[index_best_child],
                                                  rng,
                                                  available_depth,
                                                  balance_factor);
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
         let state = *stack.last().unwrap(); // note that we do not mutate the stack yet
         match state.expand().as_slice()
         {
            [] =>
            {
               // terminal state
               stack.pop();
               formula.push(state);
               no_expand(&mut tree, distribution_root, rng, available_depth, balance_factor)
            }
            [rule] =>
            {
               // single rule, we can focus on it
               stack.pop();
               stack.extend(rule);
               no_expand(&mut tree, distribution_root, rng, available_depth, balance_factor)
            }
            _rules =>
            {
               // non terminal state, we explore randomly (at a depth function of the balance_factor)
               let length = expected_formula_length(balance_factor, distribution_root.nb_visit());
               let search_depth = length + available_depth - 1;
               let (formula, score) = random_expand(formula.clone(), stack.clone(), rng, search_depth);
               (ReturnType::DoNothing, formula, score)
            }
         }
      }
      Tree::Leaf { formula, .. } =>
      {
         let score = formula.evaluate();
         (ReturnType::DeleteChild, formula.clone(), score)
      }
   }
}