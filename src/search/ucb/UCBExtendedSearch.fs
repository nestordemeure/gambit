module UCBESearch

open Search

//-----------------------------------------------------------------------------
// TYPE

/// encapsulate the informations needed for the UCB computation
type Distribution =
   { nbVisit:int; nbValue:int; sumValue:float; sumSquare:float}
   
   /// mean value
   member this.mean () =
      if this.nbValue = 0 then infinity else this.sumValue / (float this.nbValue)
   
   /// variance
   member this.var () =
      if this.nbValue < 2 then 0. else this.sumSquare / (float this.nbValue - 1.) + this.mean() ** 2.

   // standard deviation
   /// NOTE: this can be approximated as random (max - mean)
   /// which could maybe be done without the random as (max-mean)/2
   member this.std () =
      sqrt (this.var ())

//-----------------------------------------------------------------------------
// UCB

/// adds an evaluation
let update ucb eval =
   match eval with 
   | None -> {ucb with nbVisit = ucb.nbVisit+1} 
   | Some eval -> {nbVisit=ucb.nbVisit+1; nbValue=ucb.nbValue+1; 
                   sumValue=ucb.sumValue+eval; sumSquare=ucb.sumSquare+eval*eval}

/// returns the UCB score of the node
let score (childUCB:Distribution) (fatherUCB:Distribution) =
   let fatherNbVisit = float fatherUCB.nbVisit
   let c = childUCB.var() + sqrt(2. * (log fatherNbVisit) / (float childUCB.nbVisit))
   match childUCB.nbValue, childUCB.nbVisit with 
   | _, 0 -> // never visited, max score
      infinity 
   | 0, _ -> // never produced a valid output, minimum score
      sqrt(c * (log fatherNbVisit) / (float childUCB.nbVisit))
   | _ -> // classical UCB score
      childUCB.mean() + sqrt(c * (log fatherNbVisit) / (float childUCB.nbVisit)) 

/// returns a normalized value
let normalize (context:Distribution) score =
   let mean = context.mean()
   (score - mean) / (context.std()*3. - mean )

//-----------------------------------------------------------------------------
// CONTEXT

/// context that produces the UCB score
let context =
   {
      defaultContext = {nbVisit=0; nbValue=0; sumValue=0.; sumSquare=0.}
      update = update
      score = score
      detScore = fun data -> data.mean()
      normalize = normalize
   }
