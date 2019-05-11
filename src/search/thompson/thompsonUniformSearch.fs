module ThompsonUSearch

open Search

//-----------------------------------------------------------------------------
// TYPE

/// encapsulate the informations needed to sample from the distribution
type Distribution =
   { nbVisit:int; nbValue:int; minValue:float; maxValue:float}
   
   member this.sample () =
      Random.interval this.minValue this.maxValue

//-----------------------------------------------------------------------------
// ALGORITHM

/// adds an evaluation
let update data eval =
   match eval with 
   | None -> {data with nbVisit = data.nbVisit+1} 
   | Some eval -> {nbVisit=data.nbVisit+1; nbValue=data.nbValue+1; maxValue=max data.maxValue eval; minValue=min eval data.minValue}

/// returns the score of the node
let score childData (fatherData:Distribution) =
   if childData.nbVisit = 0 then infinity else 
      let probaValid = float (childData.nbValue + 1) / float (childData.nbVisit+2) // laplacian smoothed
      match Random.boolean probaValid with 
      | false -> - infinity
      | true when childData.nbValue = 0 -> fatherData.sample ()
      | true -> childData.sample ()

/// returns a normalized value
let normalize (context:Distribution) score =
   let mean = (context.maxValue - context.minValue) / 2.
   (score - mean) / (context.maxValue - mean )

//-----------------------------------------------------------------------------
// CONTEXT

/// context that produces the score
let context =
   {
      defaultContext = {nbVisit=0; nbValue=0; maxValue= -infinity; minValue=infinity}
      update = update
      score = score
      detScore = fun data -> data.maxValue
      normalize = normalize
   }
