module ExpandSlow 

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
and Node<'State,'Context> = { context:'Context; children:Tree<'State,'Context> array; nbVisit:int }

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

//-----------------------------------------------------------------------------
// EXPAND

/// expand function that only expands a node if it has been visited more than n times
let rec expand depth minVisit grammar (algorithm:Algorithm<_>) (result:Result<_,_>) tree =
   match tree with
   | Node node when node.nbVisit >= minVisit -> // a node
      let indexChild = if depth <= 0 then 0 else argmaxScore algorithm node.context node.children
      match exploreChild indexChild (expand depth minVisit grammar algorithm result) node.children with
      | eval, [||] -> // this subtree is now fully explored
         eval, None
      | eval, children -> // updated childrens, we rebuild the node
         let node = Node {context=algorithm.update node.context eval; children=children; nbVisit=node.nbVisit+1}
         eval, Some node
   | Node node -> // a node that CANNOT currently grow childrens
      let indexChild = if depth <= 0 then 0 else argmaxScore algorithm node.context node.children
      let child = node.children.[indexChild]
      match exploreChild indexChild (expand depth minVisit grammar algorithm result) node.children with
      | eval, [||] -> // this subtree is now fully explored
         eval, None
      | eval, _ -> // we rebuild the node but discard the new children
         node.children.[indexChild] <- child // insure that no modification happened
         let node = Node {node with context=algorithm.update node.context eval; nbVisit=node.nbVisit+1}
         eval, Some node
   | Leaf (formula, state::stack) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let node = Leaf (state::formula, stack)
         expand depth minVisit grammar algorithm result node
      | childrens -> // non terminal state
         let children = Array.map (fun rule -> Leaf(formula, rule@stack)) childrens
         let node = Node {context=algorithm.defaultContext; children=children; nbVisit=1}
         expand (depth-1) minVisit grammar algorithm result node
   | Leaf (formula,[]) -> // a terminal leaf (this branch is fully explored)
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      eval, None

//-----------------------------------------------------------------------------
// SEARCH

/// search for a given number of iterations using the given expand function, context and grammar
let search result expand algorithm grammar maxIterations =
   let tree = Leaf ([], [grammar.rootState])
   let rec search tree =
      match expand grammar algorithm result tree with 
      | _, Some tree when result.nbEvaluation < maxIterations -> search tree
      | _ -> result
   search tree

//-----------------------------------------------------------------------------
// DATA COLLECTION

/// search that exports (steps, value) to a given csv
let dataCollectionSearch expand algorithm grammar maxIterations (csv:Csv) =
   let tree = Leaf ([], [grammar.rootState])
   let result = Result.Single.create false
   let rec search tree =
      match expand grammar algorithm result tree with 
      | _, Some tree when result.nbEvaluation < maxIterations -> 
         csv.addLine result.nbEvaluation (result.best () |> snd)
         search tree
      | _ -> ()
   search tree

// performs nbDatapoints search and document them in the given csv file for later analysis
let dataCollection search expand algorithm grammar maxIterations nbDatapoints outFile =
   let csv = Csv()
   for p = 1 to nbDatapoints do
      search expand algorithm grammar maxIterations csv
   csv.write outFile
