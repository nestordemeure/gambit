module ExpandKill

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

/// returns the node (and not a leaf or contracted) with the lowest score
let argminScore algorithm children =
   let mutable minScore = infinity 
   let mutable minIndex = 0 
   for i = 0 to Array.length children - 1 do 
      match children.[i] with 
      | Leaf _ -> ()
      | Node node -> 
         let score = algorithm.detScore node.context
         if (minScore = infinity) || (score < minScore) then 
            minScore <- score 
            minIndex <- i
   minIndex

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
   | Leaf (state::stack) -> // an unexpanded leaf
      match grammar.expand state with 
      | [||] -> // terminal state 
         let children = [| Leaf stack |]
         let node = Node {context=algorithm.defaultContext; children=children; state=Some state}
         decr nbAvailableNodes
         let eval, treeOpt = expand (depth-1) grammar algorithm result nbAvailableNodes formula node
         eval, treeOpt
      | children -> // non terminal state
         let children = Array.map (fun rule -> Leaf (rule@stack)) children
         let node = Node {context=algorithm.defaultContext; children=children; state=None}
         decr nbAvailableNodes
         let eval, treeOpt = expand (depth-1) grammar algorithm result nbAvailableNodes formula node
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

let checkCount memory nbAvailableNodes tree isContract =
   let trueCount = countNodes tree 
   let theoricalCount = memory - !nbAvailableNodes
   if trueCount <> theoricalCount then 
      let section = if isContract then "contract" else "expand"
      printfn "expected:%d but obtained:%d due to %s" theoricalCount trueCount section

//-----------------------------------------------------------------------------
// CONTRACT

/// tries to increase the number of available nodes
/// returns true if the current node should be deleted
let rec contract algorithm nbAvailableNodes tree =
   match tree with 
   | Leaf _ -> failwith "unable to delete a node"
   | Node node when Array.exists (isNode >> not) node.children -> 
      // there is at least a leaf, no need to search for a node, we can cut here
      nbAvailableNodes := !nbAvailableNodes + countNodes tree - 1
      true
   | Node node ->
      let indexChild = argminScore algorithm node.children
      let child = node.children.[indexChild]
      match child with 
      | Leaf _ -> failwith "should never happen"
      | Node _ ->
         if contract algorithm nbAvailableNodes child then 
            node.children <- Array.remove indexChild node.children
            incr nbAvailableNodes
            not (Array.exists isNode node.children) // if there are justs leafs, we might has well kill the node
            //Array.isEmpty node.children
         else false

//-----------------------------------------------------------------------------
// SEARCH

/// search for a given number of iterations using the given expand function, context and grammar
let search depth result algorithm grammar maxIterations memorysize =
   let mutable tree = Leaf ([grammar.rootState])
   let nbAvailableNodes = ref memorysize
   while result.nbEvaluation < maxIterations do 
      if !nbAvailableNodes <= 0 then 
         contract algorithm nbAvailableNodes tree |> ignore
      else 
         match expand depth grammar algorithm result nbAvailableNodes [] tree with 
         | _, Some newTree -> tree <- newTree
         | _ -> ()
   result

//-----------------------------------------------------------------------------
// DATA COLLECTION

/// search that exports (steps, value) to a given csv
let dataCollectionSearch depth algorithm grammar maxIterations memorysize (csv:Csv) =
   let mutable tree = Leaf ([grammar.rootState])
   let result = Result.Single.create false
   let nbAvailableNodes = ref memorysize
   let mutable keepGoing = true
   while keepGoing && result.nbEvaluation < maxIterations do 
      if !nbAvailableNodes <= 0 then 
         //let previous = !nbAvailableNodes
         keepGoing <- not (contract algorithm nbAvailableNodes tree)
         //checkCount memorysize nbAvailableNodes tree true
         //printfn "decreased by %d" (!nbAvailableNodes - previous)
      else 
         let previous = !nbAvailableNodes
         match expand depth grammar algorithm result nbAvailableNodes [] tree with 
         | _, Some newTree -> tree <- newTree
                              csv.addLine result.nbEvaluation (result.best () |> snd)
                              //checkCount memorysize nbAvailableNodes tree false
                              //printfn "increased by %d" (previous - !nbAvailableNodes)
         | _ -> ()
   if result.nbEvaluation < maxIterations then printfn "stopped prematurelay at %d/%d" result.nbEvaluation maxIterations

// performs nbDatapoints search and document them in the given csv file for later analysis
let dataCollection depth algorithm grammar maxIterations memorysize nbDatapoints outFile =
   let csv = Csv()
   for p = 1 to nbDatapoints do
      dataCollectionSearch depth algorithm grammar maxIterations memorysize csv
   csv.write outFile
