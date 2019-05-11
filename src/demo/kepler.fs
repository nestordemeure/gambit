module Kepler

open Grammar
open Arithmetic

//-------------------------------------------------------------------------------------------------
// DATA

/// we want to express period of each planet as a function of their distance to the sun
let keplerPath = "src/demo/input/kepler.csv"
let variables, data = CSV.readRegression keplerPath "period"

//-------------------------------------------------------------------------------------------------
// GRAMMAR

/// expands a state using a limited set of operations and a single variable
let expand state =
   match state with
   | Expression ->
      [|
         [Base]
         [Function; Expression]
         [Operator; Expression; Expression]
      |]
   | Base ->
      [|
         [Variable]
         [Number]
         [O '^'; Variable; Number]
      |]
   | Operator -> rules O [|'+';'-';'/'|]
   | Function -> rules F [|"cos";"sin";"log";"sqrt"|]
   | Number -> rules N [|1.;2.;3.;4.|]
   | Variable -> [| [V 0] |] // only one variable
   | _ -> [||] // terminal state, no expansion

/// outputs a formula as a string
let print formula =
   let formula = Arithmetic.print variables formula
   sprintf "period = %s" formula

/// grammar to search for keplers third law
let grammar = 
   {
      rootState = Expression
      expand = expand
      evalFunction = Grammar.regression Arithmetic.evaluate data
   }
