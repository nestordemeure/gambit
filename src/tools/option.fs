module Option

/// computes the sum of the context of the options
let inline sum o1 o2 =
   match o1, o2 with 
   | Some x1, Some x2 -> Some (x1 + x2)
   | None, o | o, None -> o 