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

type header = {
  format : string;
  core_spec : string;
  module_name : Ext_name.t;
}

type located_name = {
  name : Ext_name.t;
  offset : Ext_bytes.offset;
}

let expected_format = "NPA-CERT-0.1"

let expected_core_spec = "NPA-Core-0.1"

let find_dot_offset component =
  let rec loop index =
    if index >= String.length component then None
    else if component.[index] = '.' then Some index
    else loop (index + 1)
  in
  loop 0

let read_name section reader =
  let name_offset = Ext_bytes.offset reader in
  match Ext_bytes.read_usize section reader with
  | Error err -> Error err
  | Ok (component_count, after_count) ->
      if component_count = 0 then
        Ext_bytes.error section name_offset Ext_bytes.Empty_name
      else
        let rec loop remaining current components =
          if remaining = 0 then
            match Ext_name.of_components (List.rev components) with
            | None -> Ext_bytes.error section name_offset Ext_bytes.Empty_name
            | Some name -> Ok (name, current)
          else
            let component_offset = Ext_bytes.offset current in
            match Ext_bytes.read_string_with_offset section current with
            | Error err -> Error err
            | Ok ((component, component_content_offset), next) ->
                if component = "" then
                  Ext_bytes.error section component_offset Ext_bytes.Empty_name_component
                else
                  match find_dot_offset component with
                  | Some dot_offset ->
                      Ext_bytes.error section (component_content_offset + dot_offset)
                        Ext_bytes.Dotted_name_component
                  | None -> loop (remaining - 1) next (component :: components)
        in
        loop component_count after_count []

let read_header reader =
  match Ext_bytes.read_string Ext_bytes.Header_format reader with
  | Error err -> Error err
  | Ok (format, after_format) ->
      if format <> expected_format then
        Ext_bytes.error Ext_bytes.Header_format (Ext_bytes.offset after_format)
          Ext_bytes.Format_mismatch
      else (
        match Ext_bytes.read_string Ext_bytes.Header_core_spec after_format with
        | Error err -> Error err
        | Ok (core_spec, after_core_spec) ->
            if core_spec <> expected_core_spec then
              Ext_bytes.error Ext_bytes.Header_core_spec (Ext_bytes.offset after_core_spec)
                Ext_bytes.Core_spec_mismatch
            else (
              match read_name Ext_bytes.Header_module after_core_spec with
              | Error err -> Error err
              | Ok (module_name, next) ->
                  Ok ({ format; core_spec; module_name }, next)))

let read_name_table reader =
  match Ext_bytes.read_usize Ext_bytes.Name_table reader with
  | Error err -> Error err
  | Ok (name_count, after_count) ->
      let rec loop remaining current names =
        if remaining = 0 then Ok (List.rev names, current)
        else
          let entry_offset = Ext_bytes.offset current in
          match read_name Ext_bytes.Name_table current with
          | Error err -> Error err
          | Ok (name, next) ->
              if List.exists (fun entry -> Ext_name.equal entry.name name) names then
                Ext_bytes.error Ext_bytes.Name_table entry_offset Ext_bytes.Duplicate_name
              else loop (remaining - 1) next ({ name; offset = entry_offset } :: names)
      in
      loop name_count after_count []
