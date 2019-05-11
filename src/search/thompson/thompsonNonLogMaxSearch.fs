module ThompsonNLMSearch

open Search
open CSV

//-----------------------------------------------------------------------------
// TYPE

/// encapsulate the informations needed to sample from the distribution
type Distribution =
   { nbVisit:int; nbValue:int; maxValue:float; sumValues:float; }
   
   member this.mean () =
      this.sumValues / float this.nbValue
   
   /// log(this.nbValues) is an approximation of the height of the tree
   /// high tree will make better decisions and return better values
   /// 
   /// the approximation of the height should be improvable by dividing with the log of the branching factor
   /// and correcting the factor 0.5 introduced by Random
   /// but it is not a quantity easily computable
   member this.sample () =
      let mi = this.mean()
      let ma = this.maxValue
      Random.interval mi ma

//-----------------------------------------------------------------------------
// ALGORITHM

/// adds an evaluation
let update data eval =
   match eval with 
   | None -> {data with nbVisit = data.nbVisit+1} 
   | Some eval -> 
         {nbVisit=data.nbVisit+1; nbValue=data.nbValue+1; 
         maxValue=max data.maxValue eval; sumValues=data.sumValues + eval;}

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
   let maximum = context.maxValue * log(float context.nbValue)
   //let sign x = if x > 0. then 1. else -1.
   (score - mean) / (maximum - mean)

//-----------------------------------------------------------------------------
// CONTEXT

/// context that produces the score
let context =
   {
      defaultContext = {nbVisit=0; nbValue=0; maxValue= -infinity; sumValues=0.;}
      update = update
      score = score
      detScore = fun data -> data.mean()
      normalize = normalize
   }
