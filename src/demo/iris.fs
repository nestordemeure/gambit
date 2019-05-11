module IrisRepresentation 

open Grammar
open RepresentationLearning

//-----------------------------------------------------------------------------
// Expand

let expand nbVariables state =
   match state with
   | Coordinates -> 
      [|
         [Expression; Expression] // two coordinates
      |]
   | Expression ->
      [|
         [Variable]
         [Function; Expression]
         [Operator; Expression; Expression]
      |]
   | Operator ->
      [|
         [O '+']
         [O '-']
         [O '*']
         [O '/']
      |]
   | Function ->
      [|
         [F "log"]
         [F "exp"]
         [F "sqrt"]
      |]
   | Variable -> 
      Array.init nbVariables (fun i -> [V i])
   | _ -> [||] // terminal state, no expansion

//-----------------------------------------------------------------------------
// GRAMMAR

/// the iris dataset
let irisPath = "src/demo/input/iris.csv"
let variables, irisDataSet = CSV.readClassification irisPath "class"
let nbFeatures = 2

/// displays a set of coordinates
let print = RepresentationLearning.print variables

/// counts the number of missed points
let nbMissedPoints = RepresentationLearning.nbMissedPoints irisDataSet nbFeatures

/// grammar to try and find a good feature space for the Iris dataset
let grammar = 
   {
      rootState = Coordinates
      expand = expand variables.Length
      evalFunction = RepresentationLearning.evaluate irisDataSet nbFeatures
   }