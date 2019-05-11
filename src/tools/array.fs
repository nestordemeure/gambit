module Array

/// removes an index from an array by rebuilding it
/// NOTE: could be made more efficient with a resizable array
let remove i tab =
   Array.init (Array.length tab - 1) (fun j -> if j < i then tab.[j] else tab.[j+1])

/// returns the element with the largest score
/// NOTE: desambiguate equals randomly
let argmax score (tab : 'a array) =
   let score i = score tab.[i], Random.integer (10*tab.Length)
   {0..tab.Length-1} |> Seq.maxBy score 

/// returns the element with the smalest score
/// NOTE: desambiguate equals randomly
let argmin score (tab : 'a array) =
   let score i = score tab.[i], Random.integer (10*tab.Length)
   {0..tab.Length-1} |> Seq.minBy score 