module CSV 

//-----------------------------------------------------------------------------
// OUTPUT

/// encapsulate a csv that is build line by line
type Csv() =
   let mutable lines = []
   /// adds a line to the csv
   member this.addLine index value = 
      let line = sprintf "%d,\t%f" index value
      lines <- line::lines
   /// writes all lines to a given file
   member this.write filename =
      System.IO.File.WriteAllLines(filename, lines)
      printfn "file '%s' has been written." filename

//-----------------------------------------------------------------------------
// INPUT

/// an array of (output,inputs)
type Dataset<'Target> = ('Target * float[]) array

/// splits a line at a given char
let split (sep:char) (line:string) =
   line.Split sep

/// reads a csv of and outputs variables, data (all full of strings)
let readRaw filePath =
   let lines = System.IO.File.ReadAllLines filePath
   let variables = split ',' lines.[0]
   let data = 
      lines
      |> Seq.skip 1
      |> Seq.map (split ',')
      |> Seq.filter (Array.isEmpty >> not)
   variables, data

/// reads a csv and outputs (variables,[|(inputs,output)|])
let readRegression filePath targetVar : (string[] * Dataset<float>) =
   let variables, data = readRaw filePath
   let indexTarget = Array.findIndexBack ((=) targetVar) variables
   let data = 
      data 
      |> Seq.map (Array.map float)
      |> Seq.map (fun row -> row.[indexTarget], Array.remove indexTarget row)
      |> Seq.toArray
   Array.remove indexTarget variables, data

/// reads a csv and outputs (variables,[|(inputs,class)|])
let readClassification filePath targetVar : (string[] * Dataset<string>) =
   let variables, data = readRaw filePath
   let indexTarget = Array.findIndexBack ((=) targetVar) variables
   let data = 
      data 
      |> Seq.map (fun row -> row.[indexTarget], (row |> Array.remove indexTarget |> Array.map float))
      |> Seq.toArray
   Array.remove indexTarget variables, data