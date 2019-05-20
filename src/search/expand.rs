use rand::Rng;
use float_ord::FloatOrd;
use crate::distribution::Distribution;
use crate::grammar::{Grammar, Formula};
use super::*;

/// takes a tree, its prior, a random number generator and the available depth and expand the tree
/// return the result of the expansion as a (ReturnType, formula, Option<score>)
pub fn expand<State, Distr, RNG>(mut tree: &mut Tree<Distr>,
                                 distribution_root: &Distr,
                                 mut formula: Formula<State>,
                                 mut stack: Vec<State>,
                                 rng: &mut RNG,
                                 available_depth: i64)
                                 -> (ReturnType<Tree<Distr>>, Formula<State>, State::ScoreType)
   where State: Grammar,
         Distr: Distribution<ScoreType = State::ScoreType>,
         RNG: Rng
{
   match stack.pop()
   {
      None =>
      {
         // terminal node, we evaluate the formula and backpropagate
         let score = formula.evaluate();
         (ReturnType::DeleteChild, formula, score)
      }
      Some(state) =>
      {
         // non terminal leaf, we expand into a node
         match state.expand().as_slice()
         {
            [] =>
            {
               // terminal state
               formula.push(state);
               expand(&mut tree, distribution_root, formula, stack, rng, available_depth)
            }
            [rule] =>
            {
               // single rule, we can focus on it
               stack.extend(rule);
               expand(&mut tree, distribution_root, formula, stack, rng, available_depth)
            }
            rules =>
            {
               // we need to choose a rule
               match tree
               {
                  Tree::Deleted => panic!("Expand: tried to explore a deleted tree!"),
                  Tree::Leaf =>
                  {
                     // we expand the leaf and then explore it
                     let childrens_distributions = (0..rules.len()).map(|_| Distr::new()).collect();
                     let children = (0..rules.len()).map(|_| Tree::Leaf).collect();
                     let mut new_node = Tree::Node(Box::new(Node { childrens_distributions, children }));
                     stack.push(state); // there might be a more elegant way
                     let result =
                        expand(&mut new_node, distribution_root, formula, stack, rng, available_depth - 1);
                     new_tree(result, new_node)
                  }
                  Tree::Node(box Node { ref mut childrens_distributions, ref mut children }) =>
                  {
                     // we choose a child using the prior and explore it
                     let index_best_child = best_child(childrens_distributions,
                                                       children,
                                                       distribution_root,
                                                       rng,
                                                       available_depth);
                     // update the stack
                     let rule = rules[index_best_child].clone();
                     stack.extend(rule);
                     // expand the child
                     let (action, formula, score) = expand(&mut children[index_best_child],
                                                           &childrens_distributions[index_best_child],
                                                           formula,
                                                           stack,
                                                           rng,
                                                           available_depth);
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
               }
            }
         }
      }
   }
}