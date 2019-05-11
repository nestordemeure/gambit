module ExpandHash

open Grammar
open Search
open Result.Single
open CSV

//-----------------------------------------------------------------------------
// TYPES

/// a child with its rule and context
type Child<'State,'Context> = {mutable context:'Context; rule:'State list}

/// a node with the maximum depth of a formula that has visited him and its children
type Node<'State,'Context> = { children: Child<'State,'Context> array }

/// an array of state -> node
type Tree<'State, 'Context when 'State : comparison> = (Map<'State, Node<'State,'Context>>) array

//-----------------------------------------------------------------------------
// FUNCTIONS

/// go to the node at the given position
/// expands it if necessary
let getNode grammar algorithm (tree:Tree<_,_>) formula state stack =
   // we hash (formula, stack) in order to have a hash as specific as possible
   // no need to hash state since we deal with that using a proper Map
   let index = (formula, stack) |> hash |> abs |> fun h -> h % tree.Length
   match Map.tryFind state tree.[index] with 
   | Some node -> node
   | None -> 
      let children = Array.map (fun rule -> {context=algorithm.defaultContext; rule=rule}) (grammar.expand state)
      let node = {children=children}
      tree.[index] <- Map.add state node tree.[index]
      node

/// returns the node with the highest score
let argmaxScore algorithm fathersContext children =
   let score child = algorithm.score child.context fathersContext
   Array.argmax score children // argmax will randomly deambiguate equal solutions

//-----------------------------------------------------------------------------
// EXPAND

let rec expand maxAdditionalDepth grammar (algorithm:Algorithm<_>) (result:Result<_,_>) tree depth nodeContext (formula, stack) = 
   match stack with 
   | state::stack ->
      let node = getNode grammar algorithm tree formula state stack
      // the number (3) might be the fraction of the tree that we want to visit for a given depth
      // it might be dependent of the mean branching for a given problem
      // we could use depth > mean_formula_length_so_far - maxAdditionalDepth as a criteria
      let nodeDepth = log(1. + float (result.nbEvaluation)) * 3.
      let maxAdditionalDepth = if (float depth) > nodeDepth then maxAdditionalDepth-1 else maxAdditionalDepth
      match node.children with 
      | [||] -> // terminal node, we consume an element in the stack
         expand maxAdditionalDepth grammar algorithm result tree depth nodeContext (state::formula, stack)
      | children -> // non terminal node, we apply a rule
         let indexChild = if maxAdditionalDepth <= 0 then 0 else argmaxScore algorithm nodeContext node.children
         let child = children.[indexChild]
         let eval = expand maxAdditionalDepth grammar algorithm result tree (depth+1) child.context (formula, child.rule@stack)
         child.context <- algorithm.update child.context eval
         eval
   | [] -> // no more stack, we can evaluate the formula
      let formula = List.rev formula 
      let eval = grammar.eval formula
      result.update formula eval
      eval

//-----------------------------------------------------------------------------
// SEARCH

/// search for a given number of iterations using the given algorithm and grammar
let search depth result algorithm grammar memorySize maxIterations =
   let tree = Array.create memorySize Map.empty
   let mutable rootContext = algorithm.defaultContext
   while result.nbEvaluation < maxIterations do 
      let rootFormula = ([], [grammar.rootState])
      let eval = expand depth grammar algorithm result tree 0 rootContext rootFormula
      rootContext <- algorithm.update rootContext eval
   result

/// search that exports (steps, value) to a given csv
let dataCollectionSearch depth algorithm grammar memorySize maxIterations (csv:Csv) =
   let tree = Array.create memorySize Map.empty
   let mutable rootContext = algorithm.defaultContext
   let result = Result.Single.create false
   while result.nbEvaluation < maxIterations do 
      let rootFormula = ([], [grammar.rootState])
      let eval = expand depth grammar algorithm result tree 0 rootContext rootFormula
      csv.addLine result.nbEvaluation (result.best () |> snd)
      rootContext <- algorithm.update rootContext eval

//-----------------------------------------------------------------------------
// DATA COLLECTION

// performs nbDatapoints search and document them in the given csv file for later analysis
let dataCollection depth algorithm grammar maxIterations memorySize nbDatapoints outFile =
   let csv = Csv()
   for p = 1 to nbDatapoints do
      dataCollectionSearch depth algorithm grammar memorySize maxIterations csv
   csv.write outFile

// note : works best with normal prior on the distribution