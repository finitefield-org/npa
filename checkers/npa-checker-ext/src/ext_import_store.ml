type public_export = {
  public_export_name : Ext_name.t;
  public_export_kind : Ext_cert.export_kind;
  public_decl_interface_hash : Ext_hash.digest;
  public_axiom_dependencies : Ext_cert.axiom_ref list;
  public_universe_params : Ext_name.t list;
  public_ty : Ext_term.t;
  public_body : Ext_term.t option;
}

type public_environment = {
  public_imports : Ext_import.entry list;
  public_exports : public_export list;
  public_module_axioms : Ext_cert.axiom_ref list;
  public_core_features : Ext_feature.feature_report_entry list;
}

type module_entry = {
  import_entry : Ext_import.entry;
  axiom_report_hash : Ext_hash.digest;
  public_environment : public_environment;
  checked_by_ext_checker : bool;
}

type store = module_entry list

type hash_mismatch = {
  hash_mismatch_kind : string;
  hash_mismatch_section : string;
  hash_mismatch_offset : int;
}

type load_error =
  | Import_dir_unavailable
  | Source_or_replay_input_rejected
  | Certificate_decode_error of Ext_bytes.decode_error
  | Certificate_hash_mismatch of hash_mismatch
  | Duplicate_import_binding of {
      duplicate_module_name : Ext_name.t;
      duplicate_export_hash : Ext_hash.digest;
      duplicate_offset : int;
    }

type resolve_error_reason =
  | Missing_import
  | Import_export_hash_mismatch
  | Import_certificate_hash_mismatch
  | Duplicate_import

type resolve_error = {
  resolve_reason : resolve_error_reason;
  resolve_offset : int;
}

let empty = []

let entries store = store

let bind result f =
  match result with
  | Error err -> Error err
  | Ok value -> f value

let has_suffix text suffix =
  let text_len = String.length text in
  let suffix_len = String.length suffix in
  text_len >= suffix_len
  && String.sub text (text_len - suffix_len) suffix_len = suffix

let contains_substring text needle =
  let text_len = String.length text in
  let needle_len = String.length needle in
  let rec loop index =
    if needle_len = 0 then true
    else if index + needle_len > text_len then false
    else if String.sub text index needle_len = needle then true
    else loop (index + 1)
  in
  loop 0

let is_source_or_replay_path path =
  has_suffix path ".npa" || contains_substring path ".npa/"
  || contains_substring path ".npa\\" || has_suffix path "replay.json"
  || contains_substring path "/replay.json" || contains_substring path "\\replay.json"

let is_npcert_path path = has_suffix path ".npcert"

let sorted_unique paths =
  let rec loop remaining previous unique =
    match remaining with
    | [] -> List.rev unique
    | path :: rest ->
        if previous = Some path then loop rest previous unique
        else loop rest (Some path) (path :: unique)
  in
  loop (List.sort String.compare paths) None []

let is_directory path =
  try Sys.is_directory path with Sys_error _ -> false

let collect_cert_paths import_dir =
  if is_source_or_replay_path import_dir then Error Source_or_replay_input_rejected
  else if not (is_directory import_dir) then Error Import_dir_unavailable
  else
    let rec collect_dir dir paths =
      let entries =
        try Ok (Array.to_list (Sys.readdir dir))
        with Sys_error _ -> Error Import_dir_unavailable
      in
      bind entries (fun entries ->
          let rec loop remaining paths =
            match remaining with
            | [] -> Ok paths
            | name :: rest ->
                let path = Filename.concat dir name in
                if is_source_or_replay_path path then loop rest paths
                else if is_directory path then
                  bind (collect_dir path paths) (fun paths -> loop rest paths)
                else if is_npcert_path path then loop rest (path :: paths)
                else loop rest paths
          in
          loop entries paths)
    in
    bind (collect_dir import_dir []) (fun paths -> Ok (sorted_unique paths))

let read_binary_file path =
  try
    let channel = open_in_bin path in
    let length = in_channel_length channel in
    let contents = really_input_string channel length in
    close_in channel;
    Ok contents
  with Sys_error _ -> Error Import_dir_unavailable

let import_hash_mismatch kind section offset =
  { hash_mismatch_kind = kind; hash_mismatch_section = section; hash_mismatch_offset = offset }

let declaration_hash_error mismatch =
  Certificate_hash_mismatch
    (import_hash_mismatch
       (Ext_canonical.declaration_hash_mismatch_kind_code
          mismatch.Ext_canonical.mismatch_kind)
       "declarations" mismatch.Ext_canonical.mismatch_offset)

let module_hash_error mismatch =
  Certificate_hash_mismatch
    (import_hash_mismatch
       (Ext_canonical.module_hash_role_kind_code
          mismatch.Ext_canonical.module_mismatch_role)
       "hashes" mismatch.Ext_canonical.module_mismatch_offset)

let public_export_of_export export =
  {
    public_export_name = export.Ext_cert.export_name;
    public_export_kind = export.Ext_cert.export_kind;
    public_decl_interface_hash = export.Ext_cert.export_decl_interface_hash;
    public_axiom_dependencies = export.Ext_cert.export_axiom_dependencies;
    public_universe_params = export.Ext_cert.export_universe_params;
    public_ty = export.Ext_cert.export_ty;
    public_body = export.Ext_cert.export_body;
  }

let public_environment_of_decoded decoded =
  {
    public_imports =
      List.map (fun import -> import.Ext_cert.import_entry) decoded.Ext_cert.imports;
    public_exports = List.map public_export_of_export decoded.Ext_cert.export_block;
    public_module_axioms = decoded.Ext_cert.axiom_report.Ext_cert.module_axioms;
    public_core_features = decoded.Ext_cert.axiom_report.Ext_cert.core_features;
  }

let module_entry_of_decoded decoded =
  {
    import_entry =
      {
        Ext_import.module_name = decoded.Ext_cert.header.Ext_cert.module_name;
        export_hash = decoded.Ext_cert.hashes.Ext_cert.export_hash;
        certificate_hash = Some decoded.Ext_cert.hashes.Ext_cert.certificate_hash;
      };
    axiom_report_hash = decoded.Ext_cert.hashes.Ext_cert.axiom_report_hash;
    public_environment = public_environment_of_decoded decoded;
    checked_by_ext_checker = false;
  }

let module_entry_from_source_free_certificate bytes =
  match Ext_cert.read_module (Ext_bytes.of_string bytes) with
  | Error err -> Error (Certificate_decode_error err)
  | Ok (decoded, _next) -> (
      match Ext_canonical.verify_declaration_hashes decoded with
      | Error err -> Error (Certificate_decode_error err)
      | Ok Ext_canonical.Declaration_hashes_ok -> (
          match Ext_canonical.verify_module_hashes bytes decoded with
          | Error err -> Error (Certificate_decode_error err)
          | Ok Ext_canonical.Module_hashes_ok -> Ok (module_entry_of_decoded decoded)
          | Ok (Ext_canonical.Module_hash_mismatch mismatch) ->
              Error (module_hash_error mismatch))
      | Ok (Ext_canonical.Declaration_hash_mismatch mismatch) ->
          Error (declaration_hash_error mismatch))

let duplicate_binding first second offset =
  if
    Ext_name.equal first.import_entry.Ext_import.module_name
      second.import_entry.Ext_import.module_name
    && first.import_entry.Ext_import.export_hash = second.import_entry.Ext_import.export_hash
  then
    Some
      (Duplicate_import_binding
         {
           duplicate_module_name = second.import_entry.Ext_import.module_name;
           duplicate_export_hash = second.import_entry.Ext_import.export_hash;
           duplicate_offset = offset;
         })
  else None

let validate_unique entries =
  let rec outer index seen remaining =
    match remaining with
    | [] -> Ok entries
    | entry :: rest -> (
        let rec inner prior =
          match prior with
          | [] -> Ok ()
          | existing :: prior_rest -> (
              match duplicate_binding existing entry index with
              | Some err -> Error err
              | None -> inner prior_rest)
        in
        match inner seen with
        | Error err -> Error err
        | Ok () -> outer (index + 1) (entry :: seen) rest)
  in
  outer 0 [] entries

let from_source_free_certificates certificates =
  let rec loop remaining decoded =
    match remaining with
    | [] -> validate_unique (List.rev decoded)
    | bytes :: rest ->
        bind (module_entry_from_source_free_certificate bytes) (fun entry ->
            loop rest (entry :: decoded))
  in
  loop certificates []

let load_import_dir import_dir =
  bind (collect_cert_paths import_dir) (fun paths ->
      let rec read_all remaining bytes =
        match remaining with
        | [] -> from_source_free_certificates (List.rev bytes)
        | path :: rest ->
            bind (read_binary_file path) (fun contents ->
                read_all rest (contents :: bytes))
      in
      read_all paths [])

let same_module entry requested =
  Ext_name.equal entry.import_entry.Ext_import.module_name requested.Ext_import.module_name

let same_export entry requested =
  entry.import_entry.Ext_import.export_hash = requested.Ext_import.export_hash

let resolve_error_kind error =
  match error.resolve_reason with
  | Missing_import | Duplicate_import -> "import_not_found"
  | Import_export_hash_mismatch
  | Import_certificate_hash_mismatch ->
      "import_hash_mismatch"

let resolve_error_reason_code reason =
  match reason with
  | Missing_import -> "missing_import"
  | Import_export_hash_mismatch -> "import_export_hash_mismatch"
  | Import_certificate_hash_mismatch -> "import_certificate_hash_mismatch"
  | Duplicate_import -> "duplicate_import"

let resolve_normal ?(offset = 0) store requested =
  let same_module_entries = List.filter (fun entry -> same_module entry requested) store in
  match same_module_entries with
  | [] -> Error { resolve_reason = Missing_import; resolve_offset = offset }
  | _ -> (
      let same_export_entries =
        List.filter (fun entry -> same_export entry requested) same_module_entries
      in
      match same_export_entries with
      | [] -> Error { resolve_reason = Import_export_hash_mismatch; resolve_offset = offset }
      | [ entry ] -> (
          match requested.Ext_import.certificate_hash with
          | None -> Ok entry
          | Some certificate_hash -> (
              match entry.import_entry.Ext_import.certificate_hash with
              | Some actual when actual = certificate_hash -> Ok entry
              | _ ->
                  Error
                    {
                      resolve_reason = Import_certificate_hash_mismatch;
                      resolve_offset = offset;
                    }))
      | _ -> Error { resolve_reason = Duplicate_import; resolve_offset = offset })
