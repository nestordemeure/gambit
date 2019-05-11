module UCBSearch

open Search

//-----------------------------------------------------------------------------
// UCB

/// encapsulate the informations needed for the UCB computation
type Distribution =
   { nbVisit:int; nbValue:int; sumValue:float }
   
   /// mean value
   member this.mean () =
      if this.nbValue = 0 then infinity else this.sumValue / (float this.nbValue)

//-----------------------------------------------------------------------------
// UCB

/// this constant can have a deep effect on the efficiency of the algorithm, cross valiation would be good
let c = (sqrt 2.) * 4.

/// adds an evaluation
let update ucb eval =
   match eval with 
   | None -> {ucb with nbVisit = ucb.nbVisit+1} 
   | Some eval -> {nbVisit=ucb.nbVisit+1; nbValue=ucb.nbValue+1; sumValue=ucb.sumValue+eval}

/// returns the UCB score of the node
let score childUCB (fatherUCB:Distribution) =
   let fatherNbVisit = float fatherUCB.nbVisit
   match childUCB.nbValue, childUCB.nbVisit with 
   | _, 0 -> // never visited, max score
      infinity 
   | 0, _ -> // never produced a valid output, minimum score
      c * sqrt((log fatherNbVisit) / (float childUCB.nbVisit))
   | _ -> // classical UCB score
      let fatherMean = fatherUCB.mean() |> abs
      let normalizedMean = childUCB.mean() / fatherMean // normalise so that is land between 0 and 1
      normalizedMean + c * sqrt((log fatherNbVisit) / (float childUCB.nbVisit)) 

//-----------------------------------------------------------------------------
// CONTEXT

/// context that produces the UCB score
let context =
   {
      defaultContext = {nbVisit=0; nbValue=0; sumValue=0.}
      update = update
      score = score
      detScore = fun data -> data.mean()
      normalize = fun _ x -> x
   }
