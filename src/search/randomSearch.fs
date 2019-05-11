module RandomSearch

open Expand

/// search until we used our number of iterations
let search result grammar maxDepth maxIterations =
   search result (expandDepth maxDepth) randomSearch grammar maxIterations

/// search that collects data
let dataCollectionSearch grammar maxDepth maxIterations =
   dataCollectionSearch (expandDepth maxDepth) randomSearch grammar maxIterations
