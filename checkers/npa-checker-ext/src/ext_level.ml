type t =
  | Zero
  | Succ of t
  | Max of t * t
  | Imax of t * t
  | Param of string

let zero = Zero
