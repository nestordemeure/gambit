use crate::distribution::Distribution;
use crate::search::{Node, Tree};
use std::mem::size_of;

/// counts the number of (tree, nodes, distr) in a tree
fn count_elements<Distr: Distribution>(tree: &Tree<Distr>) -> (usize, usize, usize)
{
   match tree
   {
      Tree::Node(box Node { children, .. }) =>
      {
         children.iter()
                 .map(|child| count_elements(child))
                 .fold((1, 1, 1), |(ta, na, da), (t, n, d)| (ta + t, na + n, da + d))
      }
      Tree::KnownLeaf(_) => (1, 0, 1),
      _ => (1, 0, 0)
   }
}

/// returns the full memory used, expressed in Mo
pub fn memory_used<Distr: Distribution>(tree: &Tree<Distr>) -> usize
{
   let tree_size = size_of::<Tree<Distr>>();
   let node_size = size_of::<Node<Distr>>();
   let distr_size = size_of::<Distr>();
   let (nb_tree, nb_nodes, nb_distr) = count_elements(tree);
   let memory_trees = nb_tree * tree_size;
   let memory_nodes = nb_nodes * node_size;
   let memory_distr = nb_distr * distr_size;
   (memory_nodes + memory_distr + memory_trees) / 1_000_000
}

/// prints a summay of the memory use of the given tree
/// the measure seem to be optimistic by roughly 30%, maybe because of fragmentation ?
pub fn memory_summary<Distr: Distribution>(tree: &Tree<Distr>)
{
   let tree_size = size_of::<Tree<Distr>>();
   let node_size = size_of::<Node<Distr>>();
   let distr_size = size_of::<Distr>();

   let (nb_tree, nb_nodes, nb_distr) = count_elements(tree);
   let memory_trees = nb_tree * tree_size;
   let memory_nodes = nb_nodes * node_size;
   let memory_distr = nb_distr * distr_size;
   let total_memory_use = memory_nodes + memory_distr + memory_trees;

   println!("Memory use: {} Mo", total_memory_use / 1_000_000);
   println!("- trees : {} Mo ({} x {} bytes)", memory_trees / 1_000_000, nb_tree, tree_size);
   println!("- nodes : {} Mo ({} x {} bytes)", memory_nodes / 1_000_000, nb_nodes, node_size);
   println!("- distributions : {} Mo ({} x {} bytes)", memory_distr / 1_000_000, nb_distr, distr_size);
}
