module Concrete

open Grammar
open Arithmetic

//-------------------------------------------------------------------------------------------------
// DATA

/// we want to express period of each planet as a function of their distance to the sun
let path = "src/demo/input/Concrete_Data.csv"
let variables, data = CSV.readRegression path "ConcreteCompressiveStrength"

//-------------------------------------------------------------------------------------------------
// GRAMMAR

/// how many bits are used to represent numbers
let resolution = 5

/// expands a state using a limited set of operations and a single variable
let expand nbVariables state =
   match state with
   | Expression ->
      [|
         [Base]
         [Function; Expression]
         [Operator; Expression; Expression]
         [O '^'; Expression; Number]
      |]
   | Base ->
      [|
         [Variable]
         //bitList resolution // a number between 0 and 1
         //Interval :: N 0. :: N 100. :: bitList resolution // a number in the given interval
      |]
   | Bits -> [| [Bit0]; [Bit1] |]
   | Operator -> rules O [|'+';'-';'*';'/'|]
   | Function -> rules F [|"log";"sqrt";"exp"|]
   | Number -> rules N [|2.;3.;4.|]
   | Variable -> Array.init nbVariables (fun i -> [V i])
   | _ -> [||] // terminal state, no expansion

/// outputs a formula as a string
let print formula =
   let formula = Arithmetic.print variables formula
   sprintf "CompressiveStrength = %s" formula

/// grammar to search for keplers third law
let grammar = 
   {
      rootState = Expression
      expand = expand variables.Length
      evalFunction = Grammar.regression Arithmetic.evaluate data
   }
