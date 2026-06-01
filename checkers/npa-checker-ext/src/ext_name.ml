type t = string list

let of_components components =
  if components = [] || List.exists (fun component -> component = "") components then None
  else Some components

let to_string name = String.concat "." name
