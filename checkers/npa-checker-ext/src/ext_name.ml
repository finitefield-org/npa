type t = string list

let contains_dot component =
  let rec loop index =
    if index >= String.length component then false
    else if component.[index] = '.' then true
    else loop (index + 1)
  in
  loop 0

let of_components components =
  if
    components = []
    || List.exists (fun component -> component = "" || contains_dot component) components
  then None
  else Some components

let to_string name = String.concat "." name

let equal left right = left = right
