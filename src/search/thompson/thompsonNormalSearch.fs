module ThompsonNSearch

open Search

//-----------------------------------------------------------------------------
// TYPE

/// encapsulate the informations needed to sample from the distribution
type Distribution =
   { nbVisit:int; nbValue:int; sumValue:float; sumSquare:float }
   
   /// mean value
   member this.mean () =
      if this.nbValue = 0 then infinity else this.sumValue / (float this.nbValue)
   
   /// variance
   member this.var () =
      if this.nbValue < 2 then 0. else this.sumSquare / (float this.nbValue - 1.) + this.mean() ** 2.

   /// standard deviation
   member this.std () =
      sqrt (this.var ())
   
   member this.sampling () =
      Random.normal (this.mean()) (this.std())
      
   member this.RejectionSampling () =
      let m = this.mean()
      let s = this.std()
      let mutable n = Random.normal m s
      while n < m + s do // use rejection sampling to find a good value
         n <- Random.normal m s
      n

//-----------------------------------------------------------------------------
// ALGORITHM

/// adds an evaluation
let update ucb eval =
   match eval with 
   | None -> {ucb with nbVisit = ucb.nbVisit+1} 
   | Some eval -> {nbVisit=ucb.nbVisit+1; nbValue=ucb.nbValue+1; sumValue=ucb.sumValue+eval; sumSquare=ucb.sumSquare+eval*eval}

/// returns the score of the node
let score childData (fatherData:Distribution) =
   if childData.nbVisit = 0 then infinity else 
      let probaValid = float (childData.nbValue + 1) / float (childData.nbVisit+2) // laplacian smoothed
      match Random.boolean probaValid with 
      | false -> - infinity
      | true when childData.nbValue = 0 -> fatherData.sampling()
      | true -> childData.sampling()

/// returns a normalized value
let normalize (context:Distribution) score =
   let mean = context.mean()
   (score - mean) / (context.std()*3. - mean )

//-----------------------------------------------------------------------------
// CONTEXT

/// context that produces the score
let context =
   {
      defaultContext = {nbVisit=0; nbValue=0; sumValue=0.; sumSquare=0.}
      update = update
      score = score
      detScore = fun data -> data.mean()
      normalize = normalize
   }
