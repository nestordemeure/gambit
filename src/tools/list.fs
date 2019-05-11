module List

/// creates a ilst of the given size and full of the given value
let rec create size value =
   if size <= 0 then [] else value :: create (size-1) value