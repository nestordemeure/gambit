module Arithmetic

open Grammar

//-----------------------------------------------------------------------------
// State

/// represents the AST of an arithmetic operation
type State =
   | Expression
   | Operator
   | O of char // a given operator
   | Function
   | F of string // a given function
   | Base
   | Number
   | Interval
   | N of float // a given number
   | Bits | Bit0 | Bit1 | EndBit // used to encode a number between 0 and 1
   | Variable
   | V of int // index of a given variable

/// roostate of an expression
let rooState = Expression

//-----------------------------------------------------------------------------
// Expand

let expand nbVariables state =
   match state with
   | Expression ->
      [|
         [Base]
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
         [F "cos"]
         [F "sin"]
         [F "log"]
         [F "exp"]
         [F "sqrt"]
      |]
   | Base ->
      [|
         [Variable]
         [Number]
         [O '^'; Variable; Number]
      |]
   | Number ->
      [|
         [N 1.]
         [N 2.]
         [N 3.]
         [N 4.]
      |]
   | Bits -> 
      [|
         [Bit0]
         [Bit1]
      |]
   | Variable -> 
      Array.init nbVariables (fun i -> [V i])
   | _ -> [||] // terminal state, no expansion

/// creates a list of bits of the given size
let bitList nbBits =
   let rec add nbBits result = if nbBits <= 0 then result else add (nbBits-1) (Bits::result)
   add nbBits [EndBit]

/// returns the sum of bits, the number of bits and the leftover formula
/// the sum of bits is in the interval ]0;2^nbBits]
/// it can never be 0 since that would not be a very useful number
let evalBits formula =
   let rec eval formula result maxResult =
      match formula with 
      | Bit0::formula -> eval formula (result*2) (maxResult*2+1)
      | Bit1::formula -> eval formula (result*2 + 1) (maxResult*2+1)
      | EndBit::formula -> (1+result, 1+maxResult, formula)
      | _ -> failwith "no EndBit was detected"
   eval formula 0 0

//-----------------------------------------------------------------------------
// Evaluate

let evalOperator op =
   match op with 
   | '+' -> (+)
   | '-' -> (-)
   | '*' -> (*)
   | '/' -> (/)
   | '^' -> ( ** )
   |  _  -> failwith "ERROR: unknown operator."

let evalFunction f =
   match f with 
   | "sqrt" -> sqrt
   | "exp" -> exp
   | "log" -> log
   | "sin" -> sin 
   | "cos" -> cos 
   | _ -> failwith "ERROR: unknown function."

let evaluate (variables : float array) formula =
   let rec eval formula =
      match formula with 
      | (O op)::formula ->
         let op = evalOperator op
         let (e1, formula) = eval formula 
         let (e2, formula) = eval formula 
         (op e1 e2), formula
      | (F f)::formula -> 
         let f = evalFunction f 
         let (e,formula) = eval formula 
         (f e), formula
      | (V x)::formula ->
         let x = variables.[x]
         x, formula
      | (N n)::formula ->
         n, formula
      | Bit0::_ | Bit1::_ ->
         let (sumBits, maxSum, formula) = evalBits formula 
         (float sumBits) / (float maxSum), formula
      | Interval :: formula -> 
         let inf, formula = eval formula 
         let sup, formula = eval formula
         let value, formula = eval formula
         inf + (sup-inf)*value, formula
      | formula ->
         sprintf "ERROR: '%A' cannot be evaluated" formula |> failwith
   match eval formula with 
   | result, [] -> result 
   | _, leftover -> sprintf "ERROR: '%A' has not been evaluated" formula |> failwith

//-----------------------------------------------------------------------------
// Print

let print (variablesNames:string array) formula =
   let rec print formula =
      match formula with 
      | Operator::formula ->
         let (e1, formula) = print formula 
         let (e2, formula) = print formula 
         let result = sprintf "(%s Operator %s)" e1 e2
         result, formula
      | (O op)::formula ->
         let (e1, formula) = print formula 
         let (e2, formula) = print formula 
         let result = sprintf "(%s %c %s)" e1 op e2
         result, formula
      | Bit0::_ | Bit1::_ ->
         let (sumBits, maxSum, formula) = evalBits formula 
         let value = (float sumBits) / (float maxSum)
         let result = sprintf "%f" value
         result, formula
      | Interval :: N inf :: N sup :: formula -> 
         let (sumBits, maxSum, formula) = evalBits formula 
         let value = (float sumBits) / (float maxSum)
         let result = sprintf "%f" (inf + (sup-inf)*value)
         result, formula
      | Function::formula -> 
         let (e,formula) = print formula 
         let result = sprintf "Function(%s)" e
         result, formula
      | (F f)::formula -> 
         let (e,formula) = print formula 
         let result = sprintf "%s(%s)" f e
         result, formula
      | (V x)::formula ->
         variablesNames.[x], formula
      | (N n)::formula ->
         (string n), formula
      | unexpandedBase::formula -> 
         let result = sprintf "%A" unexpandedBase
         result, formula
      | [] ->
         failwith "ERROR: trying to print the empty formula"
   match print formula with 
   | result, [] -> result 
   | _, leftover -> sprintf "ERROR: '%A' has not been printed" formula |> failwith

//-----------------------------------------------------------------------------
// Grammar

/// produces a grammar that contains arithmetic expresison including the given variables
let grammar variables =
   {
      rootState = rooState
      expand = expand (Array.length variables)
      evalFunction = evaluate variables
   }
