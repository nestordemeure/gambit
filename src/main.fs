
printfn "Starting..."

//-----------------------------------------------------------------------------
// PROBLEM STATEMENT

(*
   kepler : 
   maxIter = 500
   maxDepth = 50
   
   thousands :
   maxIter = 2000
   maxDepth = 20
*)

let maxIter = 1000
let maxDepth = 4
let minVisit = 4 // minimum number of visit needed before we accept to expand childrens of a node
let maxFormulaSize = 23 // used for the stacked search
let nbDatapoints = 100 // used for the datacollection
let memorysize = 100_000 // used for the memoryconstrained version, 1_000_000 seems like a good maximum

//let example = "kepler"
//let grammar = Kepler.grammar
//let print = Kepler.print

//let example = "2019"
//let grammar = Thousands.grammar
//let print = IntegerArithmetic.print [||]

//let example = "prime"
//let grammar = Prime.grammar
//let print = IntegerArithmetic.print [|"x"|]

//let example = "regexp"
//let grammar = RegexpNumber.grammar
//let print = Regexp.print

//let example = "iris"
//let grammar = IrisRepresentation.grammar
//let print = IrisRepresentation.print

//let example = "vertebra"
//let grammar = VertebraRepresentation.grammar
//let print = VertebraRepresentation.print

let example = "concrete"
let grammar = Concrete.grammar
let print = Concrete.print

//-----------------------------------------------------------------------------
// SEARCH

/// calls the garbage colelction to insure that we start from a clean slate
let cleanMemory () =
   System.GC.Collect()
   System.GC.WaitForPendingFinalizers()

(*
printfn "\nThompson Search"
let thompsonResult = Expand.search (Result.Pareto.List.create true) (Expand.expandDepth maxDepth) ThompsonMSearch.context grammar 100_000
printfn "ThompsonSearch: %s" (thompsonResult.toString print)
//printfn "number of misclassified points : %d" (VertebraRepresentation.nbMissedPoints (fst (thompsonResult.best())))
cleanMemory ()
*)

(*
printfn "\nThompson Search memory limited"
let thompsonResultno = ExpandNo.search maxDepth ThompsonMSearch.context Prime.grammar 1_000_000 1_000_000
printfn "ThompsonSearch: %s" (thompsonResultno.toString (IntegerArithmetic.print [|"x"|]))
cleanMemory ()
*)


//-----------------------------------------------------------------------------
// DATA COLLECTION

let test grammar example maxDepth maxIter maxFormulaSize nbDatapoints =
   let outfile contextName = "data/" + example + "/" + contextName + ".csv"
   let nbIterPerStep = maxIter / maxFormulaSize
   let memorysize = maxIter / minVisit

   // exploration tests
   Expand.dataCollection Expand.dataCollectionSearch (Expand.expandDepth maxDepth) Expand.randomSearch grammar maxIter nbDatapoints (outfile "random")
   Expand.dataCollection (Expand.dataCollectionStackedSearch nbIterPerStep) (Expand.expandDepth maxDepth) Expand.randomSearch grammar maxIter nbDatapoints (outfile "stackrandom")
   Expand.dataCollection Expand.dataCollectionSearch (Expand.expandDepth maxDepth) UCBSearch.context grammar maxIter nbDatapoints (outfile "ucb")
   Expand.dataCollection Expand.dataCollectionSearch (Expand.expandDepth maxDepth) UCBESearch.context grammar maxIter nbDatapoints (outfile "ucbtuned")
   Expand.dataCollection Expand.dataCollectionSearch (Expand.expandDepth maxDepth) ThompsonMSearch.context grammar maxIter nbDatapoints (outfile "thompsonMax")

   // memory test
   //ExpandNo.dataCollection maxDepth ThompsonMSearch.context grammar maxIter memorysize nbDatapoints (outfile "thompsonNo")

   // hypothesis test
   //DataExpand.dataCollection maxDepth ThompsonMSearch.context grammar maxIter nbDatapoints (outfile "ThompsonDepth")

//test Kepler.grammar "kepler" 4 3000 5 200
//test Thousands.grammar "2019" 4 3000 (2*23) 200
//test Prime.grammar "prime" 4 3000 (5*5) 200
//test RegexpNumber.grammar "regexp" 4 3000 20 200
//test IrisRepresentation.grammar "iris" 4 3000 10 200
//test VertebraRepresentation.grammar "vertebra" 4 3000 10 200
test Concrete.grammar "concrete" 4 3000 15 100