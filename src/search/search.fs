module Search

open Grammar

/// used to describe the algorithm that choose the branches
type Algorithm<'Context> =
   {
      defaultContext: 'Context
      update: 'Context -> float Option -> 'Context
      score: 'Context -> 'Context -> float // childData -> fatherData -> score
      detScore: 'Context -> float // childData -> fatherData -> score
      normalize: 'Context -> float -> float // childData -> score -> score
   }

/// childrens will be chosen at random due to the constant score
let randomSearch = 
   { 
      defaultContext = (0, 0.)
      update = fun (n,x) evalopt -> match evalopt with None -> (n,x) | Some y -> (n+1, x+y)
      score = fun _ _ -> 0.
      detScore = fun (n,x) -> x / float n
      normalize = fun _ x -> x
   }
