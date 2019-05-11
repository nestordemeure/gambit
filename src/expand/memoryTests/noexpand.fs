module ExpandNo 

open Grammar
open Search
open Result.Single
open CSV

//-----------------------------------------------------------------------------
// TYPES

/// represents a tree with node and non expanded leafs
type Tree<'State,'Context> =
   | Node of Node<'State,'Context>
   | Leaf of formula:Rule<'State> * stack:('State list) * nbVisit:int // stack, value

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
      | Leaf (_, _, 0) -> infinity
      | Leaf _ -> algorithm.score fathersContext fathersContext // we do not want leafs to be super attractiv when we are memory limited
      | Node node -> algorithm.score node.context fathersContext
   Array.argmax score children // argmax will randomly deambiguate equal solutions

//-----------------------------------------------------------------------------
// EXPAND

/// explores a path in a tree going for the best node according to the score function of the given context
/// stops when a formula is written or if the given depth is reached
let rec expand depth growthFactor grammar (algorithm:Algorithm<_>) (result:Result<_,_>) nbNodesAvailable tree =
   match tree with
   | Node node -> // a node
      let indexChild = if depth <= 0 then 0 else argmaxScore algorithm node.context node.children
      match exploreChild indexChild (expand depth growthFactor grammar algorithm result nbNodesAvailable) node.children with
      | eval, [||] -> // this subtree is now fully explored
         incr nbNodesAvailable
         eval, None
      | eval, children -> // updated childrens, we rebuild the node
         let node = Node {context=algorithm.update node.context eval ; children=children}
         eval, Some node
   | Leaf (formula, state::stack, nbVisit) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let node = Leaf (state::formula, stack, nbVisit)
         expand depth growthFactor grammar algorithm result nbNodesAvailable node
      | children when !nbNodesAvailable > 0 -> // non terminal state
         let children = Array.map (fun rule -> Leaf(formula, rule@stack, 0)) children
         let node = Node {context=algorithm.defaultContext; children=children}
         decr nbNodesAvailable
         expand (depth-1) growthFactor grammar algorithm result nbNodesAvailable node 
      | children -> // non terminal state but we do not have enough memory to store it
         let children = Array.map (fun rule -> Leaf(formula, rule@stack, 0)) children
         let node = Node {context=algorithm.defaultContext; children=children}
         // uses the growth factor (learned on the whole tree) and the number of visits on that node to deduce how far we should be allowed to go
         let length = log(1. + float nbVisit) * growthFactor |> int
         let newDepth = depth - 1 + length // TODO
         // we do not decr the number of nodes available but we will do it depending on the result
         match expand newDepth growthFactor grammar algorithm result nbNodesAvailable node with 
         | eval, Some (Node _) -> // this node cannot be forwarded, we return the original tree
            let updatedTree = Leaf (formula, state::stack, nbVisit + 1)
            eval, Some updatedTree
         | result -> // this node can be forwarded without increasing memory
            decr nbNodesAvailable
            result
   | Leaf (formula,[],_) -> // a terminal leaf (this branch is fully explored)
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      eval, None

/// computes the mean length of a branch in the tree
let meanLength tree =
   let rec length tree = 
      match tree with 
      | Leaf _ -> (1,0)
      | Node node -> 
         Array.foldBack (fun child (n,t) -> let (n2,t2) = length child in n+n2,t+t2+n2 ) node.children (0,0)
   let (n,t) = length tree 
   (float t) / (float n)

//-----------------------------------------------------------------------------
// SEARCH

/// search that exports (steps, value) to a given csv
let search depth result algorithm grammar maxIterations memorySize =
   let tree = Leaf ([], [grammar.rootState], 0)
   let nbNodesAvailable = ref memorySize
   let mutable growthFactor = 0.
   let rec search tree =
      match expand depth growthFactor grammar algorithm result nbNodesAvailable tree with 
      | _, Some tree when !nbNodesAvailable <= 0 && growthFactor = 0. ->
         // we are now missing some memory, lets compute the growth factor for the search in memory scarce conditions
         // the growthFactor let us convert a number of visits in the tree to a mean formula length
         growthFactor <- (meanLength tree - float depth) / log(1. + float result.nbEvaluation)
         //printfn "the memoryLimit has been hit (%d iterations)" result.nbEvaluation
         search tree
      | _, Some tree when result.nbEvaluation < maxIterations -> 
         search tree
      | _ -> result
   search tree

/// search that exports (steps, value) to a given csv
let dataCollectionSearch depth algorithm grammar maxIterations memorySize (csv:Csv) =
   let tree = Leaf ([], [grammar.rootState], 0)
   let result = Result.Single.create false
   let nbNodesAvailable = ref memorySize
   let mutable growthFactor = 0.
   let rec search tree =
      match expand depth growthFactor grammar algorithm result nbNodesAvailable tree with 
      | _, Some tree when !nbNodesAvailable <= 0 && growthFactor = 0. ->
         // we are now missing some memory, lets compute the growth factor for the search in memory scarce conditions
         // the growthFactor let us convert a number of visits in the tree to a mean formula length
         growthFactor <- (meanLength tree - float depth) / log(1. + float result.nbEvaluation)
         csv.addLine result.nbEvaluation (result.best () |> snd)
         search tree
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

(*
   https://docs.microsoft.com/fr-fr/dotnet/api/system.gc.gettotalmemory?view=netframework-4.8
   GC.gettotalmemory(false)
   returns the memory usage in octets
   we could use that as a criteria for when to freeze memory use
*)