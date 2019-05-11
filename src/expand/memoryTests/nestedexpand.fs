module ExpandNested

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
and Node<'State,'Context> = { context:'Context; children:Tree<'State,'Context> array }

//-----------------------------------------------------------------------------
// FUNCTIONS

/// expands a single child
let exploreChild indexChild expandFunction (children:Tree<_,_> array) =
   match expandFunction children.[indexChild] with 
   | eval, Some tree -> 
      children.[indexChild] <- tree
      eval, children
   | eval, None -> 
      let childrens = Array.remove indexChild children
      eval, childrens

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

//-----------------------------------------------------------------------------
// EXPAND

/// explores a path in a tree going for the best node according to the score function of the given context
/// stops when a formula is written or if the given depth is reached
let rec expand depth grammar (algorithm:Algorithm<_>) (result:Result<_,_>) nbNodes tree =
   match tree with
   | Node node -> // a node
      let indexChild = if depth <= 0 then 0 else argmaxScore algorithm node.context node.children
      match exploreChild indexChild (expand depth grammar algorithm result nbNodes) node.children with
      | eval, [||] -> // this subtree is now fully explored
         decr nbNodes
         eval, None
      | eval, children -> // updated childrens, we rebuild the node
         let node = Node {context=algorithm.update node.context eval ; children=children}
         eval, Some node
   | Leaf (formula, state::stack) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let node = Leaf (state::formula, stack)
         expand depth grammar algorithm result nbNodes node
      | childrens -> // non terminal state
         let children = Array.map (fun rule -> Leaf(formula, rule@stack)) childrens
         let node = Node {context=algorithm.defaultContext; children=children}
         incr nbNodes
         expand (depth-1) grammar algorithm result nbNodes node  
   | Leaf (formula,[]) -> // a terminal leaf (this branch is fully explored)
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      eval, None

//-----------------------------------------------------------------------------
// PRUNE

/// counts the number of nodes in the tree
let rec countNodes tree = 
   match tree with 
   | Leaf _ -> 0
   | Node node -> 1 + Array.sumBy countNodes node.children

/// keeps only the best child
let pruneAllButBest algorithm tree =
   let getScore tree = match tree with Node n -> algorithm.detScore n.context | _ -> -infinity
   match tree with 
   | Node node -> Array.maxBy getScore node.children
   | Leaf _ -> tree

/// prunes a tree by keeping only the best child
/// keeps nbnode up to date
let prune algorithm nbNodes tree =
   let tree = pruneAllButBest algorithm tree 
   nbNodes := countNodes tree 
   //printfn "prunning done, number of nodes reduces to %d" (!nbNodes)
   tree

//-----------------------------------------------------------------------------
// SEARCH

/// search that exports (steps, value) to a given csv
let dataCollectionSearch depth algorithm grammar maxIterations memorySize (csv:Csv) =
   let tree = Leaf ([], [grammar.rootState])
   let result = Result.Single.create false
   let nbNodes = ref 0
   let rec search tree =
      if !nbNodes >= memorySize then tree |> prune algorithm nbNodes |> search else 
         match expand depth grammar algorithm result nbNodes tree with 
         | _, Some tree when result.nbEvaluation < maxIterations -> 
            csv.addLine result.nbEvaluation (result.best () |> snd)
            search tree
         | _ -> ()
   search tree

//-----------------------------------------------------------------------------
// DATA COLLECTION

// performs nbDatapoints search and document them in the given csv file for later analysis
let dataCollection depth algorithm grammar maxIterations memorySize nbDatapoints outFile =
   let csv = Csv()
   for p = 1 to nbDatapoints do
      dataCollectionSearch depth algorithm grammar maxIterations memorySize csv
   csv.write outFile
