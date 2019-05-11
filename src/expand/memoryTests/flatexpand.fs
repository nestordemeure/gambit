module ExpandFlat

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

/// represents a node's child with informations relative to its score
and Node<'State,'Context> = { context:'Context; mutable children:Tree<'State,'Context> array; state:'State Option}

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

/// return the fusion of both array with the omission of childIndex
let flatten (children:_[]) childIndex (childChildren:_[]) =
   let result = Array.zeroCreate (children.Length + childChildren.Length - 1)
   for i = 0 to childChildren.Length - 1 do 
      result.[i] <- childChildren.[i]
   for i = 0 to childIndex - 1 do 
      result.[childChildren.Length + i] <- children.[i]
   for i = childIndex + 1 to children.Length - 1 do 
      result.[childChildren.Length + i - 1] <- children.[i]
   result 

/// expands a single child
let exploreChild indexChild expandFunction (children:Tree<_,_> array) =
   match expandFunction children.[indexChild] with 
   | eval, Some (Node node) when false -> // (node.state = None) && (Array.forall isNode node.children) -> 
      // this children can be flattened
      let children = flatten children indexChild node.children
      eval, children
   | eval, Some tree -> 
      children.[indexChild] <- tree
      eval, children
   | eval, None -> 
      let children = Array.remove indexChild children
      eval, children

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
let rec expand depth grammar (algorithm:Algorithm<_>) (result:Result<_,_>) formula tree =
   match tree with
   | Node node -> // a node
      let indexChild = if depth <= 0 then 0 else argmaxScore algorithm node.context node.children
      let formula = consOpt node.state formula
      match exploreChild indexChild (expand depth grammar algorithm result formula) node.children with
      | eval, [||] -> // this subtree is now fully explored
         eval, None
      | eval, children -> // updated childrens, we rebuild the node
         let node = Node {node with context=algorithm.update node.context eval; children=children}
         eval, Some node
   | Leaf (state::stack) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let children = [| Leaf stack |]
         let node = Node {context=algorithm.defaultContext; children=children; state=Some state}
         // WARNING using depth-1 here cause a change in asymptotic performances
         let eval, treeOpt = expand depth grammar algorithm result formula node
         eval, treeOpt
      | children -> // non terminal state
         let children = Array.map (fun rule -> Leaf (rule@stack)) children
         let node = Node {context=algorithm.defaultContext; children=children; state=None}
         let eval, treeOpt = expand (depth-1) grammar algorithm result formula node
         eval, treeOpt
   | Leaf [] -> // a terminal leaf (this branch is fully explored)
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      eval, None

//-----------------------------------------------------------------------------
// CHECK

/// counts the number of nodes in a tree
let rec countNodes tree =
   match tree with 
   | Leaf _ -> 0
   | Node node -> 1 + Array.sumBy countNodes node.children

//-----------------------------------------------------------------------------
// SEARCH

/// search for a given number of iterations using the given expand function, context and grammar
let search depth result algorithm grammar maxIterations =
   let mutable tree = Leaf ([grammar.rootState])
   while result.nbEvaluation < maxIterations do 
      match expand depth grammar algorithm result [] tree with 
      | _, Some newTree -> tree <- newTree
      | _ -> ()
   result

//-----------------------------------------------------------------------------
// DATA COLLECTION

/// search that exports (steps, value) to a given csv
let dataCollectionSearch depth algorithm grammar maxIterations (csv:Csv) =
   let mutable tree = Leaf ([grammar.rootState])
   let result = Result.Single.create false
   while result.nbEvaluation < maxIterations do 
      match expand depth grammar algorithm result [] tree with 
      | _, Some newTree -> tree <- newTree
                           csv.addLine result.nbEvaluation (result.best () |> snd)
      | _ -> ()
   printfn "nb nodes: %d" (countNodes tree)

// performs nbDatapoints search and document them in the given csv file for later analysis
let dataCollection depth algorithm grammar maxIterations nbDatapoints outFile =
   let csv = Csv()
   for p = 1 to nbDatapoints do
      dataCollectionSearch depth algorithm grammar maxIterations csv
   csv.write outFile
