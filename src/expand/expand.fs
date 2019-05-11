module Expand 

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
// EXPLORE

/// expands a single child
let exploreChild indexChild expandFunction (children:Tree<_,_> array) =
   match expandFunction children.[indexChild] with 
   | eval, Some tree -> 
      children.[indexChild] <- tree
      eval, children
   | eval, None -> 
      let childrens = Array.remove indexChild children
      eval, childrens

/// expands all the childrens
let exploreAllChildren expandFunction children =
   let mutable sumEval = None
   let extractEval child =
      let (eval, treeOpt) = expandFunction child
      sumEval <- Option.sum sumEval eval 
      treeOpt
   let children = Array.choose extractEval children
   sumEval, children

//-----------------------------------------------------------------------------
// EXPAND

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

//-----------------------------------------------

/// explores a path in a tree going for the best node according to the score function of the given context
/// stops when a formula is written or if the given depth is reached
let rec expandDepth depth grammar (algorithm:Algorithm<_>) (result:Result<_,_>) tree =
   match tree with
   | Node node -> // a node
      let indexChild = if depth <= 0 then 0 else argmaxScore algorithm node.context node.children
      match exploreChild indexChild (expandDepth depth grammar algorithm result) node.children with
      | eval, [||] -> // this subtree is now fully explored
         eval, None
      | eval, children -> // updated childrens, we rebuild the node
         let node = Node {context=algorithm.update node.context eval ; children=children}
         eval, Some node
   | Leaf (formula, state::stack) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let node = Leaf (state::formula, stack)
         expandDepth depth grammar algorithm result node
      | childrens -> // non terminal state
         let children = Array.map (fun rule -> Leaf(formula, rule@stack)) childrens
         let node = Node {context=algorithm.defaultContext; children=children}
         expandDepth (depth-1) grammar algorithm result node  
   | Leaf (formula,[]) -> // a terminal leaf (this branch is fully explored)
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      eval, None

/// expand function that expands by a single node per expansion to reduce memory growth
/// when we drop new nodes, we might want to at least keep some information on their quality for later pass
let rec expandDrop depth grammar (algorithm:Algorithm<_>) (result:Result<_,_>) tree =
   match tree with
   | Node node -> // a node
      let indexChild = argmaxScore algorithm node.context node.children
      match exploreChild indexChild (expandDrop depth grammar algorithm result) node.children with
      | eval, [||] -> // this subtree is now fully explored
         eval, None
      | eval, children -> // updated childrens, we rebuild the node
         let node = Node {context=algorithm.update node.context eval; children=children}
         eval, Some node
   | Leaf (formula, state::stack) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let node = Leaf (state::formula, stack)
         expandDrop depth grammar algorithm result node
      | childrens -> // non terminal state
         let children = Array.map (fun rule -> Leaf(formula, rule@stack)) childrens
         let node = Node {context=algorithm.defaultContext; children=Array.copy children}
         // expand but then drop the expansion (if it contains a children, otherwise no need)
         match expandDepth (depth-1) grammar algorithm result node with
         | eval, Some (Node node) when Array.exists isNode node.children -> eval, Some (Node {node with children = children})
         | output -> output
   | Leaf (formula,[]) -> // a terminal leaf (this branch is fully explored)
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      eval, None

/// explores all paths in a tree
/// stops when a formula is written or if the given depth is reached
let rec expandAll depth grammar (algorithm:Algorithm<_>) (result:Result<_,_>) tree =
   match tree with
   | Node node -> // a node
      let explore = if depth <= 0 then exploreChild 0 else exploreAllChildren
      match explore (expandAll depth grammar algorithm result) node.children with
      | eval, [||] -> // this subtree is now fully explored
         eval, None
      | eval, children -> // updated childrens, we rebuild the node
         let node = Node {context=algorithm.update node.context eval; children=children}
         eval, Some node
   | Leaf (formula, state::stack) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let node = Leaf (state::formula, stack)
         expandAll depth grammar algorithm result node
      | childrens -> // non terminal state
         let children = Array.map (fun rule -> Leaf(formula, rule@stack)) childrens
         let node = Node {context=algorithm.defaultContext; children=children}
         expandAll (depth-1) grammar algorithm result node  
   | Leaf (formula,[]) -> // a terminal leaf (this branch is fully explored)
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      eval, None

//-----------------------------------------------------------------------------
// PRUNE

/// keeps only the best children
let pruneAllButBest algorithm tree =
   let getContext tree = match tree with Node n -> n.context | _ -> failwith "Leafs have no context."
   match tree with 
   | Node node -> Array.maxBy (fun n -> algorithm.detScore (getContext n)) node.children
   | Leaf _ -> tree

//-----------------------------------------------------------------------------
// SEARCH

/// childrens will be chosen at random due to the constant score
let randomSearch = 
   { 
      defaultContext = (0, 0.)
      update = fun (n,x) evalopt -> match evalopt with None -> (n,x) | Some y -> (n+1, x+y)
      score = fun _ _ -> 0.
      detScore = fun (n,x) -> x / float n
      normalize = fun _ x -> x
   }

/// search for a given number of iterations using the given expand function, context and grammar
let search result expand algorithm grammar maxIterations =
   let tree = Leaf ([], [grammar.rootState])
   let rec search tree =
      match expand grammar algorithm result tree with 
      | _, Some tree when result.nbEvaluation < maxIterations -> search tree
      | _ -> result
   search tree

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

/// search for a given number of iteration but, every nbIterationPerStep steps, get rid of all main child but the best
let stackedSearch nbIterationPerStep result expand algorithm grammar maxIterations =
   let tree = Leaf ([], [grammar.rootState])
   let rec search iterLastStep tree =
      match expand grammar algorithm result tree with 
      | _, Some tree when result.nbEvaluation < maxIterations -> 
         if result.nbEvaluation - iterLastStep < nbIterationPerStep then search iterLastStep tree else
            search result.nbEvaluation (pruneAllButBest algorithm tree)
      | _ -> result
   search 0 tree

/// stacked search that exports (steps, value) to a given csv
let dataCollectionStackedSearch nbIterationPerStep expand algorithm grammar maxIterations (csv:Csv) =
   let tree = Leaf ([], [grammar.rootState])
   let result = Result.Single.create false
   let rec search iterLastStep tree =
      match expand grammar algorithm result tree with 
      | _, Some tree when result.nbEvaluation < maxIterations -> 
         csv.addLine result.nbEvaluation (result.best () |> snd)
         if result.nbEvaluation - iterLastStep < nbIterationPerStep then search iterLastStep tree else
            search result.nbEvaluation (pruneAllButBest algorithm tree)
      | _ -> ()
   search 0 tree

//-----------------------------------------------------------------------------
// DATA COLLECTION

// performs nbDatapoints search and document them in the given csv file for later analysis
let dataCollection search expand algorithm grammar maxIterations nbDatapoints outFile =
   let csv = Csv()
   for p = 1 to nbDatapoints do
      search expand algorithm grammar maxIterations csv
   csv.write outFile
