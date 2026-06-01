type t =
  | Sort of Ext_level.t
  | BVar of int
  | Const of Ext_name.t
  | App of t * t
  | Lam of Ext_name.t option * t * t
  | Pi of Ext_name.t option * t * t
  | Let of Ext_name.t option * t * t * t

let sort_zero = Sort Ext_level.zero
