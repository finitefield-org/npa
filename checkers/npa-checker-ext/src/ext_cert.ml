type declaration_kind =
  | Axiom
  | Definition
  | Theorem
  | Inductive
  | Constrained

type declaration = {
  name : Ext_name.t;
  kind : declaration_kind;
}

type t = {
  module_name : Ext_name.t option;
  declarations : declaration list;
}

let empty = { module_name = None; declarations = [] }
