use rand::Rng;
use crate::tools::lne;
use crate::distribution::Distribution;
use crate::grammar::{Grammar, Formula};
use super::Tree;
use super::random_expand::random_expand;
use super::*;

//-----------------------------------------------------------------------------
// FUNCTION

/// computes the mean length of a branch in the tree
fn mean_branch_length<Distr: Distribution>(tree: &Tree<Distr>) -> f64
{
   /// computes (number of leafs in the tree, the sum of their length)
   fn length<Distr: Distribution>(tree: &Tree<Distr>) -> (usize, usize)
   {
      match tree
      {
         Tree::Node(box Node { children, .. }) => children.iter().fold((0, 0), |(na, ta), child| {
                                                                    let (n, t) = length(child);
                                                                    (na + n, ta + t + na)
                                                                 }),
         _ => (0,1)
      }
   }
   let (nb_leafs, total_length) = length(tree);
   (nb_leafs as f64) / (total_length as f64)
}

/// computes the balance factor of the tree
/// the larger it is, the more unbalanced the tree is
/// NOTE: we are modeling the growth of the tree with the formula:
/// balance_factor * lne(nb_visit) = mean_formula_length
pub fn compute_balance_factor<Distr: Distribution>(tree: &Tree<Distr>, nb_visit: usize) -> f64
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
pub fn no_expand<State, Distr, RNG>(mut tree: &mut Tree<Distr>,
                                    mut formula: Formula<State>,
                                    mut stack: Vec<State>,
                                    rng: &mut RNG,
                                    available_depth: i64,
                                    balance_factor: f64)
                                    -> (ReturnType<Tree<Distr>>, Formula<State>, State::ScoreType)
   where State: Grammar,
         Distr: Distribution<ScoreType = State::ScoreType>,
         RNG: Rng
{
   match stack.last()
   {
      None =>
      {
         // terminal node, we evaluate the formula and backpropagate
         let score = formula.evaluate();
         (ReturnType::DeleteChild, formula, score)
      }
      Some(&state) =>
      {
         // non terminal leaf, we expand into a node
         match state.expand().as_slice()
         {
            [] =>
            {
               // terminal state
               stack.pop();
               formula.push(state);
               no_expand(&mut tree, formula, stack, rng, available_depth, balance_factor)
            }
            [rule] =>
            {
               // single rule, we can focus on it
               stack.pop();
               stack.extend(rule);
               no_expand(&mut tree, formula, stack, rng, available_depth, balance_factor)
            }
            rules =>
            {
               // we need to choose a rule
               match tree
               {
                  Tree::Deleted => panic!("Expand: tried to explore a deleted tree!"),
                  Tree::Leaf =>
                  {
                     // non terminal state, we explore randomly (at a depth function of the balance_factor)
                     let mut distribution = Distr::new();
                     let length = expected_formula_length(balance_factor, distribution.nb_visit());
                     let search_depth = length + available_depth - 1;
                     let (formula, score) = random_expand(formula, stack, rng, search_depth);
                     distribution.update(score);
                     let known_leaf = Tree::KnownLeaf(Box::new(distribution));
                     (ReturnType::NewTree(known_leaf), formula, score)
                  }
                  Tree::KnownLeaf(box distribution) =>
                  {
                     // non terminal state, we explore randomly (at a depth function of the balance_factor)
                     let length = expected_formula_length(balance_factor, distribution.nb_visit());
                     let search_depth = length + available_depth - 1;
                     let (formula, score) = random_expand(formula, stack, rng, search_depth);
                     distribution.update(score);
                     (ReturnType::DoNothing, formula, score)
                  }
                  Tree::Node(box Node { ref mut distribution, ref mut children }) =>
                  {
                     // we choose a child using the prior and explore it
                     let index_best_child = best_child(children,
                                                       distribution,
                                                       rng,
                                                       available_depth);
                     // update the stack
                     let rule = rules[index_best_child].clone();
                     stack.pop();
                     stack.extend(rule);
                     // expand the child
                     let (action, formula, score) = no_expand(&mut children[index_best_child],
                                                              formula,
                                                              stack,
                                                              rng,
                                                              available_depth,
                                                              balance_factor);
                     distribution.update(score);
                     match action
                     {
                        ReturnType::DeleteChild =>
                        {
                           children[index_best_child] = Tree::Deleted;
                           if children.iter().all(Tree::is_deleted)
                           {
                              // no more children, we can delete this node
                              (ReturnType::DeleteChild, formula, score)
                           }
                           else
                           {
                              // still some children, we keep this node
                              (ReturnType::DoNothing, formula, score)
                           }
                        }
                        ReturnType::DoNothing =>
                        {
                           // we can update the child's prior
                           (ReturnType::DoNothing, formula, score)
                        }
                        ReturnType::NewTree(child_tree) =>
                        {
                           // we can replace this child and update its prior
                           children[index_best_child] = child_tree;
                           (ReturnType::DoNothing, formula, score)
                        }
                     }
                  }
               }
            }
         }
      }
   }
}
