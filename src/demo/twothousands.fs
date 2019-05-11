module Thousands

open Grammar
open IntegerArithmetic

//-------------------------------------------------------------------------------------------------
// GRAMMAR

/// this formula avoids wasting time in multiplication by one
/// its helps a lot in reducing te search space
let expand state =
   match state with 
   | Expression ->
      [|
         [N 1]
         [O '+'; Expression; Expression]
         [O '*'; Base; Base]
      |]
   | Base -> // cannot multiply by one
      [|
         [O '+'; Expression; Expression]
         [O '*'; Base; Base]
      |]
   | _ -> [||]

/// returns the distance to 2019
let evaluate formula =
   let value = IntegerArithmetic.evaluate [||] formula
   2019 - value |> abs |> (~-) |> float

/// grammar to search for keplers third law
let grammar = 
   {
      rootState = Expression
      expand = expand
      evalFunction = evaluate
   }