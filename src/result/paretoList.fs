module Result.Pareto.List

open Grammar
open Result.Single

//-------------------------------------------------------------------------------------------------
// TYPES

/// represents an individual result store in the pareto front
type ParetoElement<'State> = {formula:Rule<'State>; score:float; cost:int}

/// represents the pareto front using a list
type ParetoFront<'State> = ParetoElement<'State> list

//-------------------------------------------------------------------------------------------------
// FUNCTIONS

/// adds a formula with a given score and cost to the result
/// we want to maximize score and minimize cost 
let rec update result formula score cost = 
   match result with 
   | [] -> [{formula=formula; score=score; cost=cost}]
   | r :: result when r.score <= score && r.cost >= cost -> // we pareto dominate this result
      update result formula score cost 
   | r :: result when r.score < score -> // we are better but more expensive
      {formula=formula; score=score; cost=cost} :: r :: result 
   | r :: result when r.cost > cost -> // we are worst but cheaper
      r :: update result formula score cost
   | _ -> result // we are pareto dominated

/// returns true if the given score is an improvement on the best known result
let isImprovement result score = 
   match result with 
   | r :: _ when r.score >= score -> false 
   | _ -> true 

/// returns the best element in the result
let best result =
   let element = List.head result
   (element.formula, element.score)

/// outputs a string that represents the result
let toString result formulaPrinter =
   let toString singleResult = 
      sprintf "score: %f\tcost: %d\tformula: %s\n" singleResult.score singleResult.cost (formulaPrinter singleResult.formula)
   List.fold (fun acc r -> acc + toString r) "" result

//-------------------------------------------------------------------------------------------------
// RESULT

/// stores a pareto front in a list
let create shouldPrint =
   {
      result = []
      nbEvaluation = 0
      shouldPrint = shouldPrint
      updateFunction = fun result formula score -> update result formula score (List.length formula), isImprovement result score
      bestFunction = best
      toStringFunction = toString
   }