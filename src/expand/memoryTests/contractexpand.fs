module ExpandContract

open Grammar
open Search
open Result.Single
open CSV

//-----------------------------------------------------------------------------
// TYPES

/// represents a tree with node and non expanded leafs
type Tree<'State,'Context> =
   | Node of Node<'State,'Context>
   | Leaf of stack:('State list) // stack, value
   | Contracted of stack:('State list) * context:'Context // stack, value, context

/// represents a node's child with informations relative to its score
and Node<'State,'Context> = { context:'Context; children:Tree<'State,'Context> array
                              state:'State Option; stack:'State list ;}

//-----------------------------------------------------------------------------
// FUNCTIONS

/// returns true if a child is a node
let isNode tree =
   match tree with 
   | Node _ -> true 
   | _ -> false

/// push opt in front of list if it contains a value
let consOpt opt list =
   match opt with 
   | None -> list 
   | Some x -> x :: list

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
      | Contracted (_,context) -> algorithm.score context fathersContext
      | Node node -> algorithm.score node.context fathersContext
   Array.argmax score children // argmax will randomly deambiguate equal solutions

/// returns the node (and not a leaf or contracted) with the lowest score
let argminScore algorithm fathersContext children =
   let score tree =
      match tree with 
      | Leaf _ | Contracted _ -> infinity
      | Node node -> algorithm.score node.context fathersContext
   Array.argmin score children // argmin will randomly deambiguate equal solutions

//-----------------------------------------------------------------------------
// EXPAND

/// expand function that only expands a node if it has been visited more than n times
let rec expand depth grammar (algorithm:Algorithm<_>) (result:Result<_,_>) nbAvailableNodes formula tree =
   match tree with
   | Node node -> // a node
      let indexChild = if depth <= 0 then 0 else argmaxScore algorithm node.context node.children
      let formula = consOpt node.state formula
      match exploreChild indexChild (expand depth grammar algorithm result nbAvailableNodes formula) node.children with
      | eval, [||] -> // this subtree is now fully explored
         incr nbAvailableNodes
         eval, None
      | eval, children -> // updated childrens, we rebuild the node
         let node = Node {node with context=algorithm.update node.context eval; children=children}
         eval, Some node
   | Contracted (state::stack as oldStack, context) -> // a previously contracted node
      match grammar.expand state with 
      | [||] -> // terminal state 
         let children = [| Contracted (stack, context) |]
         let node = Node {context=context; children=children; state=Some state; stack=stack}
         decr nbAvailableNodes
         let eval, treeOpt = expand depth grammar algorithm result nbAvailableNodes formula node
         eval, treeOpt
      | children -> // non terminal state
         let children = Array.map (fun rule -> Leaf (rule@stack)) children
         let node = Node {context=context; children=children; state=None; stack=oldStack}
         decr nbAvailableNodes
         let eval, treeOpt = expand (depth-1) grammar algorithm result nbAvailableNodes formula node
         eval, treeOpt
   | Leaf (state::stack as oldStack) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let children = [| Leaf stack |]
         let node = Node {context=algorithm.defaultContext; children=children; state=Some state; stack=stack}
         decr nbAvailableNodes
         let eval, treeOpt = expand depth grammar algorithm result nbAvailableNodes formula node
         eval, treeOpt
      | children -> // non terminal state
         let children = Array.map (fun rule -> Leaf (rule@stack)) children
         let node = Node {context=algorithm.defaultContext; children=children; state=None; stack=oldStack}
         decr nbAvailableNodes
         let eval, treeOpt = expand (depth-1) grammar algorithm result nbAvailableNodes formula node
         eval, treeOpt
   | Leaf [] | Contracted ([], _) -> // a terminal leaf (this branch is fully explored)
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      eval, None

//-----------------------------------------------------------------------------
// CONTRACT

/// returns true if it was able to contract the a node and false otherwise
let rec contract algorithm nbAvailableNodes tree =
   if !nbAvailableNodes <= 0 then 
      match tree with 
      | Leaf _ | Contracted _ -> ()
      | Node node when not (Array.exists isNode node.children) -> () 
      | Node node ->
         let indexChild = argminScore algorithm node.context node.children
         let child = node.children.[indexChild]
         match child with 
         | Leaf _ | Contracted _ -> // this could be fully prevented with a dedicated argmin
            contract algorithm nbAvailableNodes tree
         | Node childNode ->
            contract algorithm nbAvailableNodes child
            if !nbAvailableNodes <= 0 then 
               let stack = consOpt childNode.state childNode.stack
               node.children.[indexChild] <- Contracted (stack, childNode.context)
               incr nbAvailableNodes
               contract algorithm nbAvailableNodes tree 

//-----------------------------------------------------------------------------
// SEARCH

/// search for a given number of iterations using the given expand function, context and grammar
let search result expand algorithm grammar maxIterations memorysize =
   let mutable tree = Leaf ([grammar.rootState])
   let nbAvailableNodes = ref memorysize
   while result.nbEvaluation < maxIterations do 
      if !nbAvailableNodes <= 0 then 
         contract algorithm nbAvailableNodes tree
      else 
         match expand grammar algorithm result nbAvailableNodes [] tree with 
         | _, Some newTree -> tree <- newTree
         | _ -> ()
   result

//-----------------------------------------------------------------------------
// DATA COLLECTION

/// search that exports (steps, value) to a given csv
let dataCollectionSearch expand algorithm grammar maxIterations memorysize (csv:Csv) =
   let mutable tree = Leaf ([grammar.rootState])
   let result = Result.Single.create false
   let nbAvailableNodes = ref memorysize
   while result.nbEvaluation < maxIterations do 
      if !nbAvailableNodes <= 0 then 
         contract algorithm nbAvailableNodes tree
      else 
         match expand grammar algorithm result nbAvailableNodes [] tree with 
         | _, Some newTree -> tree <- newTree
                              csv.addLine result.nbEvaluation (result.best () |> snd)
         | _ -> ()

// performs nbDatapoints search and document them in the given csv file for later analysis
let dataCollection expand algorithm grammar maxIterations memorysize nbDatapoints outFile =
   let csv = Csv()
   for p = 1 to nbDatapoints do
      dataCollectionSearch expand algorithm grammar maxIterations memorysize csv
   csv.write outFile
