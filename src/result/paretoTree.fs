module Result.Pareto.Tree

open Grammar
open Result.Single

//-------------------------------------------------------------------------------------------------
// TYPES

/// represents an individual node of the pareto front
type ParetoElement<'State> = {formula:Rule<'State>; score:float; cost:int; smaller:ParetoFront<'State>; bigger:ParetoFront<'State>}

/// represents the pareto front using a (unbalanced) binary tree
and ParetoFront<'State> =
   | Empty 
   | Node of ParetoElement<'State>

//-------------------------------------------------------------------------------------------------
// FUNCTIONS

/// removes all the element worst than a given score
let rec removeWorst result score =
   match result with 
   | Empty -> result 
   | Node element when element.score <= score -> removeWorst element.bigger score
   | Node element -> Node {element with smaller = removeWorst element.smaller score}

/// removes all th elements more costly than a given score
let rec removeBigger result cost =
   match result with 
   | Empty -> result 
   | Node element when element.cost >= cost -> removeBigger element.smaller cost 
   | Node element -> Node {element with bigger = removeBigger element.bigger cost}

/// adds a formula with a given score and cost to the result
/// we want to maximize score and minimize cost 
let rec update result formula score cost = 
   match result with 
   | Empty -> Node {formula=formula; score=score; cost=cost; smaller=Empty; bigger=Empty}
   | Node element when element.score <= score && element.cost >= cost -> // we pareto dominate this element
      let bigger = removeWorst element.bigger score
      let smaller = removeBigger element.smaller cost
      Node {formula=formula; score=score; cost=cost; smaller=smaller; bigger=bigger}
   | Node element when element.score < score -> // we are better but more expensive
      Node {element with bigger = update element.bigger formula score cost}
   | Node element when element.cost > cost -> // we are worst but cheaper
      Node {element with smaller = update element.smaller formula score cost}
   | _ -> result // we are pareto dominated

/// returns true if the score is better than the best score so far
let rec isImprovement result score =
   match result with 
   | Empty -> true
   | Node element when element.score <= score -> isImprovement element.bigger score 
   | Node _ -> false

/// returns the best element in the result
let rec best result =
   match result with 
   | Node element when element.bigger = Empty -> (element.formula, element.score)
   | Node element -> best element.bigger
   | Empty -> failwith "No result."

/// outputs a string that represents the result
let rec toString result formulaPrinter =
   match result with 
   | Empty -> ""
   | Node {formula=formula; score=score; cost=cost; smaller=smaller; bigger=bigger} -> 
      let smaller = toString smaller formulaPrinter
      let bigger = toString bigger formulaPrinter
      let element = sprintf "score: %f\tcost: %d\tformula: %s\n" score cost (formulaPrinter formula)
      bigger + element + smaller

//-------------------------------------------------------------------------------------------------
// RESULT

/// stores a pareto front in a list
let create shouldPrint =
   {
      result = Empty
      nbEvaluation = 0
      shouldPrint = shouldPrint
      updateFunction = fun result formula score -> update result formula score (List.length formula), isImprovement result score
      bestFunction = best
      toStringFunction = toString
   }