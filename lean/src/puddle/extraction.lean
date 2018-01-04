import .syntax
import .py

namespace puddle
namespace extraction

open syntax

/-- This module defines a compiler from our droplet language to Python, and extracts
    a set of Python functions given a call the compiler.
-/

def compiler_st := (nat × list py.stmt)
def compiler_st.initial : compiler_st :=
(0, [])

def compiler_m (a : Type) : Type := state_t compiler_st option a

instance compiler_m.monad : monad compiler_m :=
begin
    intros,
    delta compiler_m,
    apply_instance,
end

def compiler_m.run (action : compiler_m py.stmt) : py.stmt :=
match action compiler_st.initial with
| none := sorry
| some (v, (_, bs)) := py.stmt.seq (bs.reverse ++ [v])
end

meta def fresh_name : compiler_m string :=
do (c, bs) ← state_t.read,
   state_t.write (c + 1, bs),
   return ("d" ++ to_string c)

namespace puddle.runtime

def input := ["puddle", "input"]
def output := ["puddle", "output"]
def mix := ["puddle", "mix"]

end puddle.runtime

meta mutual def mk_binding, compile_body, compile
with mk_binding : term → compiler_m string
| t :=
   match t with
   | (term.var x) := return x
   | _ :=
     do res ← fresh_name,
        stmt ← compile_body res t,
        (c, bs) ← state_t.read,
        state_t.write (c, stmt :: bs),
        return res
   end
with compile_body : string → term → compiler_m py.stmt
| res (term.input _) := pure $ py.stmt.assign res (py.mk_call puddle.runtime.input [])
| res (term.output d) :=
    do out_drop ← mk_binding d,
       pure $ py.stmt.expr $ py.mk_call puddle.runtime.output [py.expr.var out_drop]
| res (term.bind x ty m1 m2) :=
  do v ← mk_binding m1,
     compile_body res m2
| res (term.var x) := pure $ py.stmt.assign res (py.expr.var x)
| res (term.mix d1 d2) :=
    do d1 ← mk_binding d1,
       d2 ← mk_binding d2,
       pure $ py.stmt.assign res (py.expr.call puddle.runtime.mix [py.expr.var d1, py.expr.var d2])
| res (term.unit) := pure py.stmt.empty
with compile : term → py.fn
| t := py.fn.mk "compiled_fn" [] (do n ← fresh_name, compile_body n t).run

end extraction
end puddle

