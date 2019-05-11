module DataExpand 

open Grammar
open Search
open Result.Single
open CSV

//-----------------------------------------------------------------------------
// TYPES

/// represents a tree with node and non expanded leafs
type Tree<'State,'Context> =
   | Node of Node<'State,'Context>
   | Leaf of formula:Rule<'State> * stack:('State list) // stack, value

/// represents a node's child with informations relative to its score
and Node<'State,'Context> = { context:'Context; nbVisit:int; children:Tree<'State,'Context> array }

//-----------------------------------------------------------------------------
// FUNCTIONS

/// expands a single child
let exploreChild indexChild expandFunction (children:Tree<_,_> array) =
   match expandFunction children.[indexChild] with 
   | length, eval, Some tree -> 
      children.[indexChild] <- tree
      length, eval, children
   | length, eval, None -> 
      let children = Array.remove indexChild children
      length, eval, children

/// returns the node with the highest score
let argmaxScore algorithm fathersContext children =
   let score tree =
      match tree with 
      | Leaf _ -> infinity
      | Node node -> algorithm.score node.context fathersContext
   Array.argmax score children // argmax will randomly deambiguate equal solutions

/// returns true if a tree is a node
let isNode tree =
   match tree with 
   | Node _ -> true 
   | _ -> false
   
/// takes a csv, a context, a depth and an eval in order to store normalized information
let storeDepth (csv:Csv) (algorithm:Algorithm<_>) context depth eval =
   match eval with 
   | None -> ()
   | Some eval -> let normalizedEval = algorithm.normalize context eval
                  csv.addLine depth eval// normalizedEval

//-----------------------------------------------------------------------------
// EXPAND

/// explores a path in a tree going for the best node according to the score function of the given context
/// stops when a formula is written or if the given depth is reached
let rec expand depth store grammar (algorithm:Algorithm<_>) (result:Result<_,_>) tree =
   match tree with
   | Node node -> // a node
      let indexChild = if depth <= 0 then 0 else argmaxScore algorithm node.context node.children
      match exploreChild indexChild (expand depth store grammar algorithm result) node.children with
      | length, eval, [||] -> // this subtree is now fully explored
         store node.context node.nbVisit(*length*) eval
         length+1, eval, None
      | length, eval, children -> // updated childrens, we rebuild the node
         store node.context node.nbVisit(*length*) eval
         let node = Node {context=algorithm.update node.context eval; nbVisit=node.nbVisit+1; children=children}
         length+1, eval, Some node
   | Leaf (formula, state::stack) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let node = Leaf (state::formula, stack)
         expand depth store grammar algorithm result node
      | childrens -> // non terminal state
         let children = Array.map (fun rule -> Leaf(formula, rule@stack)) childrens
         let node = Node {context=algorithm.defaultContext; nbVisit=0; children=children}
         expand (depth-1) store grammar algorithm result node  
   | Leaf (formula,[]) -> // a terminal leaf (this branch is fully explored)
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      0, eval, None

//-----------------------------------------------------------------------------
// SEARCH

/// search that exports (steps, normalized value) to a given csv
let dataCollectionSearch depth algorithm grammar maxIterations (csv:Csv) =
   let tree = Leaf ([], [grammar.rootState])
   let result = Result.Single.create false
   let store = storeDepth csv algorithm
   let rec search tree =
      match expand depth store grammar algorithm result tree with 
      | _, _, Some tree when result.nbEvaluation < maxIterations -> search tree
      | _ -> ()
   search tree

//-----------------------------------------------------------------------------
// DATA COLLECTION

// performs nbDatapoints search and document them in the given csv file for later analysis
let dataCollection depth algorithm grammar maxIterations nbDatapoints outFile =
   let csv = Csv()
   for p = 1 to nbDatapoints do
      dataCollectionSearch depth algorithm grammar maxIterations csv
   csv.write outFile
