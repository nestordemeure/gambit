module RecyclingExpand

open Grammar
open Search
open Result.Single
open CSV

//-----------------------------------------------------------------------------
// TYPES

/// represents an index in the FIFO
type Index = int 

/// encapsulate a child (that might not have been expanded into a node)
type Child<'State> =
   {
      state:'State
      stack:'State list
      mutable index:Index Option
   }

/// a node, its distribution, its father and its childrens
type Node<'State, 'Distribution> =
   {
      mutable father:Index
      distribution:'Distribution
      children:Child<'State> array
   }

//-----------------------------------------------------------------------------
// RECYCLING MEMORY

/// stores a fixed number of node
/// recycles nodes that have not been visited since a long time
type Memory<'State,'Distribution>(size, defaultDistribution) =
   /// memory containning all existing nodes
   let memory = Array.init size (fun i -> {father=i; distribution=defaultDistribution; children=[||]})
   let mutable indexOldest = 0;
   
   /// returns the node with the given index
   member this.Item index : Node<'State,'Distribution> =
      memory.[index]
   
   /// builds a rootnode
   member this.rootNode grammar =
      let rules = grammar.expand grammar.rootState 
      let children = Array.map (fun rule -> {state=List.head rule; stack=List.tail rule; index=None}) rules
      let root = {father= -1; distribution=defaultDistribution; children=children}
      let index = this.addNode root 
      root.father <- index // the node is self referencial
      root

   /// put a node in the memory (destroying an old node)
   /// returns the index of the node
   member this.addNode node =
      // remove the index of the old node from its father's list of childrens
      let fatherIndex = memory.[indexOldest].father
      memory.[fatherIndex].children
      |> Array.tryFind (fun c -> c.index = Some indexOldest)
      |> Option.iter (fun c -> c.index <- None)
      // put the new node at the index
      let index = indexOldest
      memory.[index] <- node
      // update the index of the new node in its childrens
      for child in node.children do 
         child.index |> Option.iter (fun i -> memory.[i].father <- index)
      // update the position of the oldest node
      indexOldest <- (indexOldest + 1) % size
      // returns the index of the new node
      index

//-----------------------------------------------------------------------------
// FUNCTIONS

/// returns the score of a formula and update the result using the score
/// the formula is taken reversed and might include non terminal states
let evaluateFormula (grammar:Grammar<'State>) revformula (result:Result<_,_>) =
   /// reverse a formula and removes non terminal states
   let rec revKeepTerminals revformula formula =
      match revformula with 
      | state::revformula when grammar.isTerminal state -> revKeepTerminals revformula (state::formula)
      | _::revformula -> revKeepTerminals revformula formula 
      | [] -> 
         let score = grammar.eval formula 
         result.update formula score
         score
   revKeepTerminals revformula [] 

/// returns the index of the child with the highest score, breaks ties randomly
/// returns the first index if there is no depth left
/// NOTE: this code could be simpler, it as been written to minimize the calls to random and score
let selectChild algorithm (nodes:Memory<_,_>) node depthLeft =
   if depthLeft <= 0 then 0 else
      /// returns the index of the best leaf
      let bestLeaf bestIndex =
         let mutable bestIndex = bestIndex
         for i = bestIndex+1 to node.children.Length-1 do 
            if (node.children.[i].index = None) && (Random.equalBoolean ()) then bestIndex <- i 
         bestIndex
      /// returns the index of the best child
      let bestChild () =
         let mutable bestIndex = 0 
         let mutable bestScore = -infinity 
         for i = 0 to node.children.Length-1 do 
            let index = Option.get node.children.[i].index 
            let score = algorithm.score nodes.[index].distribution node.distribution
            if (score > bestScore) || ((score = bestScore) && (Random.equalBoolean ())) then
               bestIndex <- i 
               bestScore <- score
         bestIndex
      /// calls bestLeaf if there is a leaf and bestChild otherwise
      match Array.tryFindIndex (fun c -> c.index = None) node.children  with 
      | Some i -> bestLeaf i
      | None -> bestChild ()

/// takes an expanded rule and the current stack in order to build a new child
let childOfRule stack rule =
   match rule with 
   | state::rule -> {state=state; stack=rule@stack; index=None}
   | [] -> failwith "Error: An empty rule was passed to the 'childOfRule' function."

/// returns the node associated with the child
/// creates a new node (not storred in memory yet) if needed
/// consume depth if it has to create a new node
let nodeOfChild (nodes:Memory<_,_>) (grammar:Grammar<_>) algorithm child depthLeft =
   match child.index with 
   | Some index -> // alreaddy existing node
      nodes.[index], depthLeft 
   | None -> // creates a new node (currently not stored in memory)
      match grammar.expand child.state, child.stack with 
      | [||], state::stack -> // terminal
         let children = Array.singleton {state=state; stack=stack; index=None}
         {father= -1; distribution=algorithm.defaultContext; children=children}, depthLeft-1
      | rules, stack -> // non terminal
         let children = Array.map (childOfRule stack) rules
         {father= -1; distribution=algorithm.defaultContext; children=children}, depthLeft-1

//-----------------------------------------------------------------------------
// EXPAND

/// takes a node and returns (score, index of the node)
let rec expand depthLeft grammar algorithm (nodes:Memory<_,_>) result node formula =
   match Array.isEmpty node.children with 
   | true -> // there is no child, the formula is complete
      let score = evaluateFormula grammar formula result
      score, None
   | false -> // expands a child
      let childIndex = selectChild algorithm nodes node depthLeft
      let child = node.children.[childIndex]
      let childNode, depthLeft = nodeOfChild nodes grammar algorithm child depthLeft
      let formula = child.state::formula
      match expand depthLeft grammar algorithm nodes result childNode formula with 
      | score, None when Array.length node.children <= 1 -> // the branch is fully explored
         score, None
      | score, None -> // the branch is now fully explored but there are other branches
         let children = Array.remove childIndex node.children 
         let node = {node with children=children; distribution=algorithm.update node.distribution score}
         let nodeIndex = nodes.addNode node 
         score, Some nodeIndex
      | score, Some childNodeIndex -> // the branch is not fully explored
         child.index <- Some childNodeIndex // update children
         let node = {node with distribution=algorithm.update node.distribution score}
         let nodeIndex = nodes.addNode node 
         score, Some nodeIndex

//-----------------------------------------------------------------------------
// SEARCH

/// search using a constrained memory
let search result algorithm grammar memorysize maxDepth maxIterations =
   let memory = Memory(memorysize, algorithm.defaultContext)
   let root = memory.rootNode grammar
   let mutable percent = 0
   let rec search tree =
      match expand maxDepth grammar algorithm memory result tree [] with 
      | _, Some treeIndex when result.nbEvaluation < maxIterations -> 
         let newPercent = (result.nbEvaluation*100) / maxIterations
         if newPercent > percent then 
            printfn "progress: %d iterations." result.nbEvaluation
            percent <- newPercent
         search memory.[treeIndex]
      | _ -> result
   search root

/// search that exports (steps, value) to a given csv
let dataCollectionSearch algorithm grammar memorysize maxDepth maxIterations (csv:Csv) =
   let memory = Memory(memorysize, algorithm.defaultContext)
   let result = Result.Single.create false
   let root = memory.rootNode grammar
   let rec search tree =
      match expand maxDepth grammar algorithm memory result tree [] with 
      | _, Some treeIndex when result.nbEvaluation < maxIterations -> 
         csv.addLine result.nbEvaluation (result.best () |> snd)
         search memory.[treeIndex]
      | _ -> ()
   search root

//-----------------------------------------------------------------------------
// DATA COLLECTION

// performs nbDatapoints search and document them in the given csv file for later analysis
let dataCollection algorithm grammar memorysize maxDepth maxIterations nbDatapoints outFile =
   let csv = Csv()
   for p = 1 to nbDatapoints do
      dataCollectionSearch algorithm grammar memorysize maxDepth maxIterations csv
   csv.write outFile

// are we revisiting nodes that we already know but have forgotten ?
// in this case, is it better to use another strategy ?

// is there a simpler algorithm to kill cold nodes and keep memory consumption constrained
// maybe a percolation approach where visited node increase their index in a table

// can we profile the code to speed it up