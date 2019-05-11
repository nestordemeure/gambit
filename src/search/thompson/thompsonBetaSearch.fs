module ThompsonBSearch

open Search

//-----------------------------------------------------------------------------
// TYPE

/// encapsulate the informations needed to sample from the distribution
type Distribution =
   { nbVisit:int; nbValue:int; minValue:float; maxValue:float; sumValues:float}
   
   member this.mean() =
      this.sumValues / (float this.nbValue)
      
   member this.sample () =
      let inf = this.minValue
      let sup = this.maxValue
      let m = (this.mean() - inf) / (sup - inf)
      let v = float this.nbValue // could be nbvisit which would let us get rid of the test in score
      let a = m*v 
      let b = v - a
      let unscaledResult = Random.beta a b
      inf + unscaledResult*(sup - inf)

//-----------------------------------------------------------------------------
// ALGORITHM

/// adds an evaluation
let update data eval =
   match eval with 
   | None -> {data with nbVisit = data.nbVisit+1} 
   | Some eval -> {nbVisit=data.nbVisit+1; nbValue=data.nbValue+1; 
                   maxValue=max data.maxValue eval; minValue=min eval data.minValue;
                   sumValues=data.sumValues + eval}

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
   let mean = context.mean()
   (score - mean) / (context.maxValue - mean )

//-----------------------------------------------------------------------------
// CONTEXT

/// context that produces the score
let context =
   {
      defaultContext = {nbVisit=0; nbValue=0; maxValue= -infinity; minValue=infinity; sumValues=0.}
      update = update
      score = score
      detScore = fun data -> data.maxValue
      normalize = normalize
   }

