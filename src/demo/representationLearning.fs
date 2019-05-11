module RepresentationLearning

open Grammar

//-----------------------------------------------------------------------------
// State

/// represents the AST of an arithmetic operation
type State =
   | Coordinates
   | Expression
   | Operator
   | O of char // a given operator
   | Function
   | F of string // a given function
   | Base
   | Number
   | N of float // a given number
   | Variable
   | V of int // index of a given variable

/// roostate of an expression
let rooState = Coordinates

//-----------------------------------------------------------------------------
// Expand

/// nbvariables is the number of input variables
/// nbFeatures is the number of output variables
let expand nbVariables nbFeatures state =
   match state with
   | Coordinates -> 
      [|
         List.create nbFeatures Expression
      |]
   | Expression ->
      [|
         [Variable]
         [Operator; Expression; Expression]
         [Function; Expression]
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
         [F "sqrt"]
         [F "exp"]
         [F "log"]
      |]
   | Variable -> 
      Array.init nbVariables (fun i -> [V i])
   | _ -> [||] // terminal state, no expansion

//-----------------------------------------------------------------------------
// Interpret

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
   | "square" -> fun x -> x*x
   | "sin" -> sin 
   | "cos" -> cos 
   | _ -> failwith "ERROR: unknown function."

/// takes a number of coordinates and a formula and build an array of function that each computes the formula from the variables
/// uses the futamura projection
let makeFunction nbCoordinates formula =
   let rec eval formula =
      match formula with 
      | (O op)::formula ->
         let op = evalOperator op
         let (e1, formula) = eval formula 
         let (e2, formula) = eval formula 
         let f = fun variables -> op (e1 variables) (e2 variables)
         f, formula
      | (F f)::formula -> 
         let f = evalFunction f 
         let (e,formula) = eval formula 
         let f = e >> f
         f, formula
      | (V x)::formula ->
         let f = fun (variables:float []) -> variables.[x]
         f, formula
      | (N n)::formula ->
         let f = fun variables -> n
         f, formula
      | formula ->
         sprintf "ERROR: '%A' cannot be evaluated" formula |> failwith
   // extracts all the coordinates
   let result = Array.zeroCreate nbCoordinates
   let mutable partialFormula = formula
   for i = 0 to nbCoordinates-1 do 
      if List.isEmpty partialFormula then printfn "%A" formula; failwith "Not enough coordinates."
      let f, newFormula = eval partialFormula 
      result.[i] <- f
      partialFormula <- newFormula
   if not (List.isEmpty partialFormula) then printfn "%A -> %A" formula partialFormula; failwith "There is some leftover formula"
   result

//-----------------------------------------------------------------------------
// Evaluate

/// takes an array of coordinate formula, variables and output new coordinates
let applyCoordinates f variables =
   Array.map (fun f -> f variables) f

/// returns the mean coordinates of a dataset
let meanCoordinate (data:CSV.Dataset<string>) =
   let nbCoordinates = (snd data.[0]).Length
   let result = Array.zeroCreate nbCoordinates
   for (_,coord) in data do 
      for i = 0 to nbCoordinates-1 do 
         result.[i] <- result.[i] + coord.[i]
   let nbSample = float data.Length
   Array.map (fun c -> c / nbSample) result 

/// computes the distance between two sets of coordinates
let distance c1 c2 =
   Seq.map2 (fun x1 x2 -> (x1-x2)*(x1-x2)) c1 c2 |> Seq.sum

/// returns 1 if the points label is strictly closer than any other label
let score labelMeans (slabel, scoordinate) =
   let mutable dClosest = infinity
   let mutable labelClosest = ""
   for (label,coordinate) in labelMeans do 
      let dist = distance coordinate scoordinate
      // if it is strictly better, we update
      if dist < dClosest then 
         dClosest <- dist
         labelClosest <- label
      // if it is equal but the label is not sample.label, we update
      else if (dist = dClosest) && (label <> slabel) then 
         dClosest <- dist
         labelClosest <- label
   if labelClosest = slabel then 0. else -1.

/// given a dataset and a formula, evaluate the formula
let evaluate (data:CSV.Dataset<string>) nbFeatures formula =
   let f = makeFunction nbFeatures formula
   // data grouped by label
   let dataGrouped =
      data
      |> Array.map (fun (label,coord) -> label, applyCoordinates f coord)
      |> Array.groupBy fst
   // mean of each label
   let labelMeans = Array.map (fun (label, data) -> label, meanCoordinate data) dataGrouped
   // sum of score normalized by number of elements per label
   Array.sumBy (fun (_,data) -> (Array.sumBy (score labelMeans) data) / (float data.Length) ) dataGrouped

/// counts the number of labels that are misclassified
let nbMissedPoints (data:CSV.Dataset<string>) nbFeatures formula =
   let f = makeFunction nbFeatures formula
   let data = Array.map (fun (label,coord) -> label, applyCoordinates f coord) data
   let labelMeans =
      data
      |> Array.groupBy fst
      |> Array.map (fun (label, data) -> label, meanCoordinate data)
   Array.sumBy (score labelMeans) data |> abs |> int

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
      | Function::formula -> 
         let (e,formula) = print formula 
         let result = sprintf "Function(%s)" e
         result, formula
      | (F "square")::formula -> 
         let (e,formula) = print formula 
         let result = sprintf "(%s)^2" e
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
   /// takes a formula and displays a set of coordinates
   let rec assembleCoordinates result formula =
      match print formula with 
      | coordinate, [] -> result + ", " + coordinate + ")"
      | coordinate, formula when result = "(" -> assembleCoordinates (result + coordinate) formula
      | coordinate, formula -> assembleCoordinates (result + ", " + coordinate) formula
   assembleCoordinates "(" formula

//-----------------------------------------------------------------------------
// Grammar
