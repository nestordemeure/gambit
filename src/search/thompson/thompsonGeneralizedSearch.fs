module ThompsonGSearch

open Search

//-----------------------------------------------------------------------------
// TYPE

/// encapsulate the informations needed to sample from the distribution
type Distribution =
   { nbVisit:int; nbSucess:int; maxValue:float ref; minValue:float ref}

   /// updates the bounds
   /// NOTE: the bounds are shared by all instances of the distribution
   member this.updateBounds eval =
      if eval < !this.minValue then this.minValue := eval
      if eval > !this.maxValue then this.maxValue := eval

   /// estimate of a maximum value
   member this.sup () =
      if !this.maxValue < 0. then 0. else !this.maxValue
   
   /// estimate of a minimum value
   member this.inf () = 
      if !this.minValue > 0. then 0. else !this.minValue

   /// samples a beta distribution (with laplacian smoothing)
   member this.sample () =
      let a = 2 + this.nbSucess
      let b = 2 + this.nbVisit - this.nbSucess
      Random.beta (float a) (float b)

//-----------------------------------------------------------------------------
// ALGORITHM

/// adds an evaluation
let update data eval =
   match eval with 
   | None -> {data with nbVisit = data.nbVisit+1} 
   | Some eval -> 
      data.updateBounds eval
      let inf = data.inf()
      let sup = data.sup()
      let normalizedEval = (eval - inf) / (sup - inf)
      let sucess = if Random.boolean normalizedEval then 1 else 0
      {data with nbVisit=data.nbVisit+1; nbSucess=data.nbSucess+sucess}

/// returns the score of the node
let score childData (fatherData:Distribution) =
   if childData.nbVisit = 0 then infinity else childData.sample ()

//-----------------------------------------------------------------------------
// CONTEXT

/// context that produces the score
let context () =
   {
      defaultContext = {nbVisit=0; nbSucess=0; maxValue=ref (-infinity); minValue=ref (infinity)}
      update = update
      score = score
      detScore = fun data -> (float data.nbSucess) / (float data.nbVisit)
      normalize = fun _ x -> x
   }
