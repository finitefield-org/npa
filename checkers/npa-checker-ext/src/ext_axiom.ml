type policy = {
  deny_sorry : bool;
  deny_custom_axioms : bool;
  allowed_axioms : Ext_name.t list;
}

let default_policy =
  { deny_sorry = true; deny_custom_axioms = true; allowed_axioms = [] }
