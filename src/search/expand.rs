use rand::Rng;
use float_ord::FloatOrd;
use crate::distribution::Distribution;
use crate::grammar::{Grammar, Formula};
use super::*;

/// takes a tree, its prior, a random number generator and the available depth and expand the tree
/// return the result of the expansion as a (ReturnType, formula, Option<score>)
pub fn expand<State, Distr, RNG>(mut tree: &mut Tree<Distr>,
                                 mut formula: Formula<State>,
                                 mut stack: Vec<State>,
                                 rng: &mut RNG,
                                 available_depth: i64)
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
               expand(&mut tree, formula, stack, rng, available_depth)
            }
            [rule] =>
            {
               // single rule, we can focus on it
               stack.pop();
               stack.extend(rule);
               expand(&mut tree, formula, stack, rng, available_depth)
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
                     let children = (0..rules.len()).map(|_| Tree::Leaf).collect();
                     let mut new_node = Tree::Node(Box::new(Node { distribution: Distr::new(), children }));
                     let result = expand(&mut new_node, formula, stack, rng, available_depth - 1);
                     new_tree(result, new_node)
                  }
                  Tree::KnownLeaf(box distribution) =>
                  {
                     // we expand the leaf and then explore it
                     let children = (0..rules.len()).map(|_| Tree::Leaf).collect();
                     let mut new_node =
                        Tree::Node(Box::new(Node { distribution: distribution.clone(), children }));
                     let result = expand(&mut new_node, formula, stack, rng, available_depth - 1);
                     new_tree(result, new_node)
                  }
                  Tree::Node(box Node { ref mut distribution, ref mut children }) =>
                  {
                     // we choose a child using the prior and explore it
                     let index_best_child = best_child(children, distribution, rng, available_depth);
                     // update the stack
                     let rule = rules[index_best_child].clone();
                     stack.pop();
                     stack.extend(rule);
                     // expand the child
                     let (action, formula, score) =
                        expand(&mut children[index_best_child], formula, stack, rng, available_depth);
                     distribution.update(score);
                     match action
                     {
                        ReturnType::DeleteChild =>
                        {
                           children[index_best_child] = Tree::Deleted;
                           if children.iter().all(|t| discriminant(t) == discriminant(&Tree::Deleted))
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
