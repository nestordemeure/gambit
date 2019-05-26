
// grammar V1
grammar!
{
   State(Variable):
   #Expr => #Base
   #Expr => #Function(#Expr)
   #Expr => #Operator(#Expr, #Expr)
   #Base => Variable
   #Base => #Number
   #Base => Variable^#Number
   #Operator => +
   #Operator => -
   #Operator => /
   #Number => 1
   #Number => 2
   #Number => 3
   #Number => 4
   #Function => log
   #Function => sqrt
}

// grammar V2
grammar!
{
   State(Variable):
   #Expr => #Base | #Function(#Expr) | #Operator(#Expr, #Expr)
   #Base => Variable | #Number | Variable^#Number
   #Operator => + | - | /
   #Number => 1 | 2 | 3 | 4
   #Function => log | sqrt
}

// we should ignore comments inside grammar
// 1..4 would be a useful syntax
// a syntax to define a rule as one of the elements from a vector would be useful
// what are the separators between the rules ? a coma ?

// TODO implement macro that derive grammar from simple representation
// example : https://github.com/dtolnay/syn/tree/master/examples/lazy-static
/*

the macro should expand into :
State
rootstate()
to_string()
expand()
interpret(&[State])

the user need to implement:
evaluate

the macro has the form :

#Expr => #Base
#Expr => #Function(#Expr)
#Expr => #Expr #Operator #Expr
#Base => #Variable | #Number | #Variable^#Number
#Operator => + | - | /
#Number => 1 | 1 | 3 | 4
#Function => cos | sin | log | sqrt

the expand function converts sequence of terminals into proper states :
Expr => Base
Expr => Function, S(, Expr, S)
Expr => Expr, Operator, Expr
Base => Variable
Base => Number
Base => Variable, S^, Number
Operator => S+
Operator => S-
...

State is just all possible states (termnals and non terminals)

root_state is just the state that comes with the first rule

to_string converts :
- terminals into strings using our previous method
- recurce on non terminals

interpret ?
having a rule state in front of each rule would make the task fairly easy (i.e.: RExpr1 in front of the first rule)
and should not impact the current implementation in term of speed
we could have one interpreter per non terminal
(the main one being the one of the root state)

TODO simulate the code that would be generated for kepler
(in particular the interpreter)
*/
