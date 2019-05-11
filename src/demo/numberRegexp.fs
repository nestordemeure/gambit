module RegexpNumber

open Grammar
open Regexp

//-------------------------------------------------------------------------------------------------
// DATA

let matching = ["1";"11,1";"1,11";"+11";"-11";"-11,1"]
let nonMatching = ["a";"1,";"1,,1";"-,1";"-";"1-";"1,-1";",-1";"1,-";"1,1,1";"";",";"1,1-";"11-11"]

//-------------------------------------------------------------------------------------------------
// GRAMMAR

/// building block to make a number matching regexp
let expand state =
   match state with
   | Expression ->
      [|
         [Symbol] // a single expression
         [Operator '&'; Symbol; Expression] // a sequence of expression
      |]
   | Symbol ->
      [|
         [Operator '+'; Digit] // digits
         [Char ','] // comma
         [Operator '|'; Char '+'; Char '-'] // sign
         [Operator '?'; Expression] // optionnal expression
      |]
   | _ -> [||] // terminal state, no expansion

/// tests a potential regexp against a bank of valid and invalid examples
let evaluate formula =
   Regexp.evaluate matching nonMatching formula |> float

/// displays the output of a given formula on out test cases
let display formula =
   Regexp.printMatching (matching @ nonMatching) formula

/// grammar to search for a number matching regexp
let grammar = 
   {
      rootState = Expression
      expand = expand
      evalFunction = evaluate
   }