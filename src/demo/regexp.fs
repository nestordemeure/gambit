module Regexp 

open Grammar

//-----------------------------------------------------------------------------
// State

/// represents the AST of the regexps
type State =
   | Expression
   | Symbol
   | Operator of char 
   | Any 
   | Digit 
   | Letter 
   | Char of char 

/// roostate of an expression
let rooState = Expression

//-----------------------------------------------------------------------------
// Expand

let expand state =
   match state with
   | Expression ->
      [|
         [Symbol]
         [Operator '+'; Expression] // at least once
         [Operator '*'; Expression] // 0 or more times
         [Operator '?'; Expression] // maybe
         [Operator '|'; Expression; Expression] // or
         [Operator '&'; Expression; Expression] // concat operator
      |]
   | Symbol ->
      [|
         [Any]
         [Digit]
         [Letter]
      |]
   | Digit -> rules Char [|'0'; '1'; '2'; '3'; '4'; '5'; '6'; '7'; '8'; '9'|]
   | _ -> [||] // terminal state, no expansion

//-----------------------------------------------------------------------------
// Evaluate

/// represents a string that is being matched
type MatchedStr = 
   {position:int; str:string}
   member this.head () =
      if this.position >= this.str.Length then None 
      else Some this.str.[this.position]
   member this.incr () =
      {this with position = this.position + 1}

/// returns true if the given char is a digit
let isDigit c = c >= '0' && c <= '9'

/// returns true if the given char is a letter
let isLetter c = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')

/// applies a matcher as many time as needed
let rec many matcher (str:MatchedStr) =
   match matcher str with 
   | Some str2 when str2.position > str.position -> many matcher str2
   | _ -> Some str 

/// takes a formula and outputs a function that matches a regexp (using the futamura projection)
let makeRegexp formula =
   /// returns a function that matches a string and a leftover formula
   let rec eval formula =
      match formula with 
      | Any :: formula -> 
         let matcher = fun (str:MatchedStr) -> str.head() |> Option.map (fun _ -> str.incr ())
         matcher, formula
      | Digit :: formula -> 
         let matcher = fun (str:MatchedStr) -> str.head() |> Option.filter isDigit |> Option.map (fun _ -> str.incr ())
         matcher, formula
      | Letter :: formula -> 
         let matcher = fun (str:MatchedStr) -> str.head() |> Option.filter isLetter |> Option.map (fun _ -> str.incr ())
         matcher, formula
      | Char c :: formula ->
         let matcher = fun (str:MatchedStr) -> str.head() |> Option.filter ((=) c) |> Option.map (fun _ -> str.incr ())
         matcher, formula
      | Operator op :: formula -> 
         let matcher1, formula = eval formula
         match op with 
         | '?' -> 
            let matcher = fun (str:MatchedStr) -> matcher1 str |> Option.defaultValue str |> Some
            matcher, formula
         | '&' -> 
            let matcher2, formula = eval formula 
            let matcher = fun (str:MatchedStr) -> let strOpt = matcher1 str in if strOpt = None then None else matcher2 (Option.get strOpt)
            matcher, formula
         | '|' -> 
            let matcher2, formula = eval formula 
            let matcher = fun (str:MatchedStr) -> let strOpt = matcher1 str in if strOpt = None then matcher2 str else strOpt 
            matcher, formula
         | '*' -> 
            let matcher = many matcher1
            matcher, formula
         | '+' -> 
            let matcher = fun (str:MatchedStr) -> let strOpt = many matcher1 str in if strOpt = Some str then None else strOpt
            matcher, formula
         | _ -> failwith "unrecognized operator"
      | Expression :: _ | Symbol :: _ -> failwith "the formula was only partially expanded"
      | [] -> failwith "unable to match the empty formula"
   match eval formula with 
   | matcher, [] -> 
      fun str -> match matcher {position=0; str=str} with 
                 | Some strM when strM.position >= str.Length -> true 
                 | _ -> false
   | _ -> failwith "there is some leftover formula"

/// returns a score that is the number of valid formula matched and the number of invalid formula not matched
let evaluate valid invalid formula =
   let regexp = makeRegexp formula
   let valid = Seq.sumBy (fun s -> if regexp s then 0 else -1) valid
   let invalid = Seq.sumBy (fun s -> if regexp s then -1 else 0) invalid
   valid + invalid

//-----------------------------------------------------------------------------
// Print

/// takes a formula and produces a string representation
let print formula =
   let rec print formula =
      match formula with 
      | Any :: formula -> ".", formula
      | Digit :: formula -> "0-9", formula
      | Letter :: formula -> "a-z", formula
      | Char c :: formula -> string c, formula
      | Operator op :: formula -> 
         let regexp1, formula = print formula
         match op with 
         | '?' -> sprintf "[%s]?" regexp1, formula
         | '&' -> let regexp2, formula = print formula 
                  regexp1 + regexp2, formula
         | '|' -> let regexp2, formula = print formula 
                  sprintf "[%s|%s]" regexp1 regexp2, formula
         | '*' -> sprintf "[%s]*" regexp1, formula
         | '+' -> sprintf "[%s]+" regexp1, formula
         | _ -> failwith "unrecognized operator"
      | Expression :: _ | Symbol :: _ -> failwith "the formula was only partially expanded"
      | [] -> failwith "unable to match the empty formula"
   match print formula with 
   | str, [] -> str
   | _ -> failwith "there is some leftover formula"

/// displays all test cases with either matching or non matching
let printMatching testCases formula =
   printfn "formula: %s" (print formula)
   let regexp = makeRegexp formula 
   for str in testCases do 
      if regexp str then printfn "  valid: '%s'" str 
      else printfn "invalid: '%s'" str