module IntegerArithmetic

open Grammar

//-----------------------------------------------------------------------------
// State

/// represents the AST of an arithmetic operation
type State =
   | Expression
   | Base
   | Operator
   | O of char // a given operator
   | Number
   | RelativNumber
   | N of int // a given number
   | Bits
   | Bit0 | Bit1 | EndBit // the bit representation of a number
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
         [Operator; Expression; Expression]
      |]
   | Operator ->
      [|
         [O '+']
         [O '-']
         [O '*']
      |]
   | Base ->
      [|
         [Variable]
         [Number]
         [O '^'; Variable; Number]
      |]
   | Number ->
      [|
         [N 1]
         [N 2]
         [N 3]
         [N 4]
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

//-----------------------------------------------------------------------------
// Evaluate

let evalOperator op =
   match op with 
   | '+' -> (+)
   | '-' -> (-)
   | '*' -> (*)
   | '^' -> pown
   |  _  -> failwith "ERROR: unknown operator."

let evalBits formula =
   let rec eval formula result =
      match formula with 
      | Bit0::formula -> eval formula (result*2)
      | Bit1::formula -> eval formula (result*2 + 1)
      | EndBit::formula -> (result, formula)
      | _ -> failwith "no EndBit was detected"
   eval formula 0

let evaluate (variables : int array) formula =
   let rec eval formula =
      match formula with 
      | (O '~')::formula -> 
         let (e,formula) = eval formula 
         (-e), formula
      | (O op)::formula ->
         let op = evalOperator op
         let (e1, formula) = eval formula 
         let (e2, formula) = eval formula 
         (op e1 e2), formula
      | (V x)::formula ->
         let x = variables.[x]
         x, formula
      | (N n)::formula ->
         n, formula
      | Bit0::_ | Bit1::_ ->
         let (n, formula) = evalBits formula 
         n, formula
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
      | (O '~')::formula ->
         let (e, formula) = print formula
         let result = "-" + e 
         result, formula
      | (O op)::formula ->
         let (e1, formula) = print formula 
         let (e2, formula) = print formula 
         let result = sprintf "(%s %c %s)" e1 op e2
         result, formula
      | (V x)::formula ->
         variablesNames.[x], formula
      | (N n)::formula ->
         (string n), formula
      | Bit0::_ | Bit1::_ -> 
         let (n, formula) = evalBits formula 
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
      evalFunction = (evaluate variables) >> float
   }
