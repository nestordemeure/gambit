module DepthSearch

open Expand

/// depth first search, the search is done incrementing the depth one by one
/// if the maximum number of iterations is reached AFTER finishing a layer, then the search will halt
let search result grammar maxIterations =
   search result (expandAll 1) randomSearch grammar maxIterations
