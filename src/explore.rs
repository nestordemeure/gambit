use grammar
use cons_list::ConsList;

//-----------------------------------------------------------------------------
// TYPES

/// stores information gotten during previous runs
struct Prior 
{
   nbVisit : u64,
   sumScore : f64,
   maxScore : f64
}

/// stores the prior and tree of each children
struct Children
{
   priors : Vec<Prior>
   tree : Vec<Tree>
}

/// stores an unexpanded leaf
struct Leaf
{
   formula : ConsList<grammar::State>,
   stack : ConsList<grammar::State>
}

/// either a leaf or a node with several childrens
enum Tree
{
   Leaf(Leaf),
   Node(Children)
}

//-----------------------------------------------------------------------------
// EXPAND

fn expand(tree:&mut Tree, rootPrior: &Prior)
{
   match tree 
   {
      Node(children) => 0.,
      Leaf(leaf) => // leaf with a non-empty stack, builds a node
      {
         match leaf.stack.head()
         {
            Some(state) => 
            {
               let stack = leaf.stack.tail();
               let rules = gramma::expand(state);
               
            }
            None => grammar::evaluate(leaf.formula) // TODO we might need to reverse the formula before evaluation
         }
      }
   }
}

//-----------------------------------------------------------------------------
// SEARCH

