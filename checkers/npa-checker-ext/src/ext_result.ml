let schema = "npa.independent-checker.checker_raw_result.v1"

let checker_id = "npa-checker-ext"

let checker_version = "0.1.0"

let checker_build_material =
  String.concat "\000"
    [
      checker_id;
      checker_version;
      "format:NPA-CERT-0.1";
      "core:NPA-Core-0.1";
      Ext_hash.vendored_sha256_source_identity;
    ]

let checker_build_hash = Ext_hash.sha256_prefixed_hex_of_string checker_build_material

type checker_error = {
  kind : string;
  reason_code : string option;
  section : string option;
  offset : int option;
}

let json_escape text =
  let buffer = Buffer.create (String.length text) in
  String.iter
    (fun ch ->
      match ch with
      | '"' -> Buffer.add_string buffer "\\\""
      | '\\' -> Buffer.add_string buffer "\\\\"
      | '\b' -> Buffer.add_string buffer "\\b"
      | '\012' -> Buffer.add_string buffer "\\f"
      | '\n' -> Buffer.add_string buffer "\\n"
      | '\r' -> Buffer.add_string buffer "\\r"
      | '\t' -> Buffer.add_string buffer "\\t"
      | _ ->
          let code = Char.code ch in
          if code < 0x20 then Buffer.add_string buffer (Printf.sprintf "\\u%04x" code)
          else Buffer.add_char buffer ch)
    text;
  Buffer.contents buffer

let json_string text = "\"" ^ json_escape text ^ "\""

let render_error error =
  let fields =
    [ "\"kind\": " ^ json_string error.kind ]
    @ (match error.reason_code with
      | None -> []
      | Some reason -> [ "\"reason_code\": " ^ json_string reason ])
    @ (match error.section with
      | None -> []
      | Some section -> [ "\"section\": " ^ json_string section ])
    @
    (match error.offset with
    | None -> []
    | Some offset -> [ "\"offset\": " ^ string_of_int offset ])
  in
  "{\n    " ^ String.concat ",\n    " fields ^ "\n  }"

let render_failed error =
  "{\n"
  ^ "  \"schema\": " ^ json_string schema ^ ",\n"
  ^ "  \"checker_id\": " ^ json_string checker_id ^ ",\n"
  ^ "  \"checker_version\": " ^ json_string checker_version ^ ",\n"
  ^ "  \"checker_build_hash\": " ^ json_string checker_build_hash ^ ",\n"
  ^ "  \"status\": \"failed\",\n"
  ^ "  \"error\": " ^ render_error error ^ "\n"
  ^ "}\n"

let skeleton_failure () =
  render_failed
    {
      kind = "checker_internal_error";
      reason_code = Some "checker_reported_internal_error";
      section = Some "skeleton";
      offset = Some 0;
    }
