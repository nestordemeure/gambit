module Random

//-----------------------------------------------------------------------------
// GENERATOR

/// xorshift pseudo random number generator
/// see: http://www.fssnip.net/7Rb/title/xorshift128plus-PRNG-that-mimics-SystemRandom
type XorshiftPRNG(seed) =
   let mutable s0 = 0UL 
   let mutable s1 = uint64 seed
   /// creates a new generator using the system tickcount as seed
   new()=XorshiftPRNG(System.Environment.TickCount)
   /// produce a random number between 0UL and System.UInt64.MaxValue
   member x.Next() =
      let mutable x = s0
      let y = s1
      s0 <- y
      x <- x ^^^ (x <<< 23)
      s1 <- x ^^^ y ^^^ (x >>> 17) ^^^ (y >>> 26)
      s1 + y // has a slim chance of being System.UInt64.MaxValue
   /// produces a random double between 0. and 1.
   member x.NextDouble() = (float (x.Next())) / float System.UInt64.MaxValue
   /// produces a random integer between 0 and max excluded
   /// NOTE: this operation is more expensive than producing a random double
   member x.Next(max) = x.NextDouble() * (float max) |> int

//-----------------------------------------------------------------------------
// OPERATIONS

/// pseudo random number generator
let generator = XorshiftPRNG() //System.Random()

/// returns an integer inferior to the given born sup
let inline integer sup = generator.Next(sup)

/// produces a random number in the given interval
let inline interval inf sup = inf + generator.NextDouble()*(sup - inf)

/// returns a real inferior to the given born sup
let inline real sup = generator.NextDouble() * sup

/// returns a random element from an array
let inline element a = 
   let index = Array.length a
   a.[integer index]

/// returns a boolean with the given probability
let inline boolean proba =
   generator.NextDouble() < proba

/// returns true with probability 1/2
//let equalBoolean () = generator.Next 2 = 0
let equalBoolean =
   let mutable bits = generator.Next()
   let mutable nbBit = 64
   fun () -> 
      bits <- bits / 2UL
      nbBit <- nbBit - 1
      if nbBit = 0 then 
         bits <- generator.Next()
         nbBit <- 64
      bits % 2UL = 0UL

/// sample from a normal law described as follow
/// NOTE: this waste a random number at each call
let normal mean std =
   let u1, u2 = generator.NextDouble(), generator.NextDouble()
   let n = sqrt(-2. * log u1) * cos ( 2.* System.Math.PI * u2 )
   mean + n*std

/// adapted from : https://github.com/CSBiology/FSharp.Stats/blob/master/src/FSharp.Stats/Distributions/Continuous.fs#L330
let gamma a b =
   let mutable a = a
   // Fix when alpha is less than one.
   let alphafix = if a >= 1.0 then 1. else (a <- a + 1.0; generator.NextDouble() ** (1.0 / a))
   let d = a - 1.0 / 3.0
   let c = 1.0 / sqrt(9.0 * d)
   let rec gamma_sample () =
      let mutable x = normal 0.0 1.0
      let mutable v = 1.0 + c * x
      while v <= 0.0 do
         x <- normal 0.0 1.0
         v <- 1.0 + c * x
      v <- v * v * v
      let u = generator.NextDouble()
      x <- x * x
      if u < 1.0 - 0.0331 * x * x then d * v
      elif (log u) < 0.5 * x + d * (1.0 - v + (log v)) then d * v
      else gamma_sample()
   alphafix * gamma_sample() / b

/// sample from a beta distribution
let beta a b =
   let x = gamma a 1.0
   let y = gamma b 1.0
   x / (x + y)