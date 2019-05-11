module IntegerPolynomial

open Grammar 
open IntegerArithmetic

//-----------------------------------------------------------------------------
// Expand

/// a polynomial with only one variable and number of at most nbBits bits
let expand nbBits state =
   match state with
   | Expression ->
      [|
         [V 0]
         [Number]
         [O '+'; Expression; Expression]
         [O '*'; Expression; Expression]
      |]
   | Number ->
      [|
         bitList nbBits
         (O '~') :: bitList nbBits
      |]
   | Bits ->
      [|
         [Bit0]
         [Bit1]
      |]
   | _ -> [||] // terminal state, no expansion