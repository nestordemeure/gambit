module VertebraRepresentation 

open Grammar
open RepresentationLearning

/// the iris dataset
let path = "src/demo/input/vertebral_column_3C.csv"
let variables, dataSet = CSV.readClassification path "class"
let nbFeatures = 2

/// displays a set of coordinates
let print = RepresentationLearning.print variables

/// counts the number of missed points
let nbMissedPoints = RepresentationLearning.nbMissedPoints dataSet nbFeatures

/// grammar to try and find a good feature space for the Iris dataset
let grammar = 
   {
      rootState = Coordinates
      expand = RepresentationLearning.expand variables.Length nbFeatures
      evalFunction = RepresentationLearning.evaluate dataSet nbFeatures
   }