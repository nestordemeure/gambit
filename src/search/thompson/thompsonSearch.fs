module ThompsonSearch

open Search

//-----------------------------------------------------------------------------
// TYPE

/// encapsulate the informations needed to sample from the distribution
type Distribution =
   { nbVisit:int; nbValue:int; sumValue:float; minValue:float }
   
   /// mean value
   member this.mean () =
      if this.nbValue = 0 then 0. else this.sumValue / (float this.nbValue)
   
   /// uniform sample in [fatherDistribution.minValue;this.mean]
   /// equivalent to returning a uniform number with esperance this.min()
   member this.sample fatherDistribution =
      Random.interval fatherDistribution.minValue (this.mean())

//-----------------------------------------------------------------------------
// ALGORITHM

/// adds an evaluation
let update data eval =
   match eval with 
   | None -> {data with nbVisit = data.nbVisit+1} 
   | Some eval -> {nbVisit=data.nbVisit+1; nbValue=data.nbValue+1; sumValue=data.sumValue+eval; minValue=min eval data.minValue}

/// returns the score of the node
let score childData (fatherData:Distribution) =
   if childData.nbVisit = 0 then infinity else 
      let probaValid = float (childData.nbValue + 1) / float (childData.nbVisit+2) // laplacian smoothed
      match Random.boolean probaValid with 
      | false -> - infinity
      | true when childData.nbValue = 0 -> fatherData.sample fatherData
      | true -> childData.sample fatherData

/// returns a normalized value
let normalize (context:Distribution) score =
   let mean = context.mean()
   (score - mean) / (mean - context.minValue)

//-----------------------------------------------------------------------------
// CONTEXT

/// context that produces the score
let context =
   {
      defaultContext = {nbVisit=0; nbValue=0; sumValue=0.; minValue=infinity}
      update = update
      score = score
      detScore = fun data -> data.mean()
      normalize = normalize
   }
