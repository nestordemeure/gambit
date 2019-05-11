module Result.Single

open Grammar

//-------------------------------------------------------------------------------------------------
// RESULT

/// encapsulate a result
type Result<'Result,'State> = 
   {
      mutable result:'Result
      mutable nbEvaluation: int
      shouldPrint : bool // are we displaying a message when we have an improvement
      updateFunction : 'Result -> Rule<'State> -> float -> ('Result * bool)
      bestFunction : 'Result -> Rule<'State> * float
      toStringFunction : 'Result -> (Rule<'State> -> string) -> string 
   }
   
   /// stores the result if it is better than the best formula so far
   member this.update formula eval =
      this.nbEvaluation <- this.nbEvaluation + 1
      match eval with 
      | Some eval -> 
         let (newResult, improvement) = this.updateFunction this.result formula eval
         if improvement && this.shouldPrint then printfn "Evaluation %d, the score is now %f." this.nbEvaluation eval
         this.result <- newResult 
      | _ -> ()
      
   /// returns the best result so far
   member this.best () =
      this.bestFunction this.result
      
   /// given a function that can print a formula, outputs a string representation of the result
   member this.toString formulaPrinter = 
      let str = this.toStringFunction this.result formulaPrinter
      sprintf "result obtained in %d iterations:\n%s" this.nbEvaluation str

//-------------------------------------------------------------------------------------------------
// SINGLE RESULT

/// stores the best result so far
let create shouldPrint =
   {
      result = ([], -infinity)
      nbEvaluation = 0
      shouldPrint = shouldPrint
      updateFunction = fun (formula,evaluation) f e -> if e < evaluation then (formula,evaluation), true else (f,e), false
      bestFunction = id
      toStringFunction = fun (formula,evaluation) printer -> sprintf "score: %f\tformula: %s" evaluation (printer formula)
   }
