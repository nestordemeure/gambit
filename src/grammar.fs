module Grammar

//-----------------------------------------------------------------------------
// GRAMMAR

/// represents a legal list of states
type Rule<'State> = 'State list

/// represents a grammar
/// 
/// roostate is the original state
/// expand takes a state and returns all the possible expansions
/// eval takes a rule and returns a score (the target is to maximize the score)
/// 
/// WARNING:
/// expand should return the rule from the closest to a terminal state to the further
/// a terminal should return an empty array (since it has no expansion)
type Grammar<'State> =
   {
      rootState:'State
      expand: 'State -> Rule<'State> array
      evalFunction: Rule<'State> -> float
   }
   
   /// evaluate a rule and encapsulate it so that invalid result becomes None
   member this.eval rule =
      let result = this.evalFunction rule 
      if result = -infinity || System.Double.IsNaN result then None else Some result

   /// returns true if a state is terminal
   /// WARNING: this fucntion is as expensive as a call to grammar expand
   member this.isTerminal state =
      state |> this.expand |> Array.isEmpty
   
   /// builds a random formula
   member this.randomFormula maxlength =
      let rec expand maxlength result formula =
         match formula with 
         | [] -> 
            List.rev result
         | state::formula ->
            match this.expand state with 
            | [||] -> // terminal state
               expand maxlength (state::result) formula
            | rules -> // non terminal state
               let newStates = if maxlength <= 0 then rules.[0] else Random.element rules
               let maxlength = maxlength + 1 - List.length newStates
               expand maxlength result (newStates @ formula)
      expand maxlength [] [this.rootState]

//-----------------------------------------------------------------------------
// FUNCTIONS

/// takes an evaluation function, some [|variables, targets|] and produces the sum of squared error
/// this can be used as an evaluation function when doing regression
let regression evaluation data formula : float =
   // norm2 error
   let error2 (output, inputs) =
      let result = evaluation inputs formula
      abs (result - output) // norm1
      //(result - output)*(result - output) // norm2
   // compute the sum of norm two errors
   - (Array.sumBy error2 data) / (float data.Length)

/// takes a constructor and an array of values and output an array or rules
/// useful to write a grammar quicker
let rules constructor values =
   Array.map (fun v -> [constructor v]) values