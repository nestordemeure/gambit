use rand::Rng;
use float_ord::FloatOrd;
use crate::distribution::Distribution;
use crate::grammar::{Grammar, Formula};

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
   pub fn new() -> Self
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
   pub fn is_deleted(&self) -> bool
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
   /// selects the child with the maximum score
   /// leafs having an infinite score, they are taken in priority
   pub fn best_child<RNG: Rng>(children: &[Tree<Distr>],
                               distribution_father: &Distr,
                               mut rng: &mut RNG,
                               available_depth: i64)
                               -> usize
   {
      // we return the first child which, by convention, should be on the shortest path to a valid formula
      // TODO provide mecanism to avoid relying on a convention and always going for the first child
      if available_depth <= 0
      {
         return 0;
      }
      // if there is a leaf, return on leaf at random
      let leaf_index = children.iter()
                               .enumerate()
                               .filter(|&(_, child)| child.is_unknown_leaf())
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
                    .filter(|&(_, child)| !child.is_deleted())
                    .max_by_key(|(_, child)| {
                       FloatOrd(child.distribution().score(distribution_father, &mut rng))
                    })
                    .map(|(i, _)| i)
                    .expect("best_child: tried to find the best child in an empty array.")
         }
      }
   }

   /// prunes all child but one in order to reduce memory occupation
   /// TODO: this function is a quick hack in order to (inv)validate the concept
   pub fn prune(&mut self)
   {
      if let Tree::Node(box Node { children, .. }) = self
      {
         let max_visit: Vec<usize> = children.iter()
                                             .enumerate()
                                             .filter(|(_, child)| child.has_distribution())
                                             .map(|(i, _)| i)
                                             .collect();
         match max_visit.as_slice()
         {
            [] => panic!("tried to prune a tree with zero children"),
            [index] => children[*index].prune(),
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
}

impl<Distr: Distribution> ReturnType<Tree<Distr>>
{
   /// injects the given tree in the result unless it suggest another action
   pub fn new_tree<State: Grammar>(result: (ReturnType<Tree<Distr>>, Formula<State>, State::ScoreType),
                                   tree: Tree<Distr>)
                                   -> (ReturnType<Tree<Distr>>, Formula<State>, State::ScoreType)
   {
      match result
      {
         (ReturnType::DoNothing, formula, score) => (ReturnType::NewTree(tree), formula, score),
         _ => result
      }
   }
}
