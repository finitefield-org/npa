let assert_equal label expected actual =
  if expected <> actual then
    failwith
      (label ^ ": expected " ^ String.escaped expected ^ " but got "
     ^ String.escaped actual)

let assert_int_equal label expected actual =
  if expected <> actual then
    failwith
      (label ^ ": expected " ^ string_of_int expected ^ " but got " ^ string_of_int actual)

let assert_int64_equal label expected actual =
  if expected <> actual then
    failwith
      (label ^ ": expected " ^ Int64.to_string expected ^ " but got " ^ Int64.to_string actual)

let assert_bool label value = if not value then failwith (label ^ ": expected true")

let assert_ok label result =
  match result with
  | Ok value -> value
  | Error error ->
      failwith
        (label ^ ": unexpected error " ^ Ext_bytes.reason_code error.Ext_bytes.reason ^ " at "
       ^ Ext_bytes.section_name error.Ext_bytes.section ^ ":"
       ^ string_of_int error.Ext_bytes.offset)

let contains text needle =
  let text_len = String.length text in
  let needle_len = String.length needle in
  let rec loop index =
    if index + needle_len > text_len then false
    else if String.sub text index needle_len = needle then true
    else loop (index + 1)
  in
  needle_len = 0 || loop 0

let assert_contains label needle text =
  if not (contains text needle) then
    failwith (label ^ ": missing " ^ String.escaped needle ^ " in " ^ String.escaped text)

let assert_cli_error label expected args =
  let result = Ext_cli.run args in
  assert_int_equal (label ^ " exit") 2 result.code;
  assert_equal (label ^ " stdout") "" result.stdout;
  assert_equal (label ^ " stderr") ("npa-checker-ext: " ^ expected ^ "\n") result.stderr

let bytes_of_codes codes =
  let bytes = Bytes.create (List.length codes) in
  List.iteri (fun index code -> Bytes.set bytes index (Char.chr code)) codes;
  bytes

let string_of_codes codes = Bytes.to_string (bytes_of_codes codes)

let mutate_byte text offset =
  if offset < 0 || offset >= String.length text then
    failwith ("cannot mutate byte at offset " ^ string_of_int offset);
  let bytes = Bytes.of_string text in
  let original = Char.code (Bytes.get bytes offset) in
  Bytes.set bytes offset (Char.chr (original lxor 0x01));
  Bytes.to_string bytes

let split_tabs line =
  let length = String.length line in
  let rec loop start fields =
    try
      let index = String.index_from line start '\t' in
      loop (index + 1) (String.sub line start (index - start) :: fields)
    with Not_found -> List.rev (String.sub line start (length - start) :: fields)
  in
  loop 0 []

let root_dir () =
  try Sys.getenv "NPA_CHECKER_EXT_ROOT"
  with Not_found -> Filename.concat (Sys.getcwd ()) "checkers/npa-checker-ext"

let boundary_input length =
  let bytes = Bytes.create length in
  for index = 0 to length - 1 do
    Bytes.set bytes index (Char.chr (((index * 17) + 31) land 0xff))
  done;
  bytes

let vector_input source label length =
  match (source, label) with
  | "standard", "empty" -> Bytes.empty
  | "standard", "abc" -> Bytes.of_string "abc"
  | "standard", "long-standard" ->
      Bytes.of_string "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
  | "standard", "million-a" -> Bytes.make 1_000_000 'a'
  | "boundary", _ -> boundary_input length
  | "rust-sha2", "build-identity-domain" ->
      Bytes.of_string
        "npa-checker-ext\000checker-build\000vendored-sha256-source:v1\000NPA-CERT-0.1\000NPA-Core-0.1"
  | "rust-sha2", "level-zero-domain" -> Bytes.of_string "npa.hash.domain\000level\000zero"
  | "rust-sha2", "term-sort-zero-domain" ->
      Bytes.of_string "npa.hash.domain\000term\000sort\000zero"
  | "rust-sha2", "binary-all-bytes" ->
      let bytes = Bytes.create 256 in
      for index = 0 to 255 do
        Bytes.set bytes index (Char.chr index)
      done;
      bytes
  | "rust-sha2", "newline-path-bytes" ->
      Bytes.of_string "npa-checker-ext\000newline-bytes\000path/with/backslash\\name\nline\r\n"
  | _ -> failwith ("unknown sha256 vector " ^ source ^ ":" ^ label)

let chunk_sizes = [| 1; 2; 3; 5; 8; 13; 21; 34; 55; 64 |]

let digest_streaming bytes =
  let state = Ext_sha256.create () in
  let offset = ref 0 in
  let chunk_index = ref 0 in
  while !offset < Bytes.length bytes do
    let chunk_size = chunk_sizes.(!chunk_index mod Array.length chunk_sizes) in
    let take = min chunk_size (Bytes.length bytes - !offset) in
    Ext_sha256.update_subbytes state bytes !offset take;
    offset := !offset + take;
    incr chunk_index
  done;
  Ext_sha256.finalize state

let assert_sha256 label bytes expected_hex =
  let digest = Ext_sha256.digest_bytes bytes in
  assert_int_equal (label ^ " raw length") 32 (Bytes.length digest);
  assert_equal (label ^ " one-shot hex") expected_hex (Ext_sha256.to_hex digest);
  assert_equal (label ^ " prefixed hex") ("sha256:" ^ expected_hex)
    (Ext_hash.sha256_prefixed_hex_of_bytes bytes);
  assert_equal (label ^ " streaming hex") expected_hex
    (Ext_sha256.to_hex (digest_streaming bytes))

let run_sha256_tests () =
  let path = Filename.concat (root_dir ()) "test/golden/sha256_vectors.tsv" in
  let channel = open_in path in
  let count = ref 0 in
  (try
     while true do
       let line = input_line channel in
       if String.length line > 0 && line.[0] <> '#' then
         match split_tabs line with
         | [ source; label; length_text; expected_hex ] ->
             let length = int_of_string length_text in
             let bytes = vector_input source label length in
             assert_int_equal (label ^ " vector length") length (Bytes.length bytes);
             assert_sha256 (source ^ ":" ^ label) bytes expected_hex;
             incr count
         | _ -> failwith ("malformed sha256 vector line: " ^ line)
     done
   with End_of_file -> close_in channel);
  assert_int_equal "sha256 vector count" 18 !count;
  let expected_build_hash =
    Ext_result.checker_build_hash_for_sha256_source_identity Ext_sha256.source_identity
  in
  assert_equal "checker build hash uses vendored sha256 source identity" expected_build_hash
    Ext_result.checker_build_hash;
  assert_bool "checker build hash is not placeholder"
    (Ext_result.checker_build_hash
    <> "sha256:0000000000000000000000000000000000000000000000000000000000000000");
  assert_bool "checker build hash changes with vendored sha256 identity"
    (Ext_result.checker_build_hash
    <> Ext_result.checker_build_hash_for_sha256_source_identity
         "vendored-sha256-source:test-change")

let run_cli_tests () =
  let version = Ext_cli.run [ "--version" ] in
  assert_int_equal "version exit" 0 version.code;
  assert_contains "version checker id" "npa-checker-ext 0.1.0\n" version.stdout;
  assert_contains "version build hash" ("checker_build_hash " ^ Ext_result.checker_build_hash)
    version.stdout;
  assert_contains "version certificate format" "certificate_format NPA-CERT-0.1\n"
    version.stdout;
  assert_contains "version core spec" "core_spec NPA-Core-0.1\n" version.stdout;
  assert_contains "version implementation profile" "implementation_profile ocaml-clean-room\n"
    version.stdout;
  assert_contains "version feature policy contract"
    "feature_policy_contract m0-05:first-release-empty-core-feature-set\n" version.stdout;
  assert_contains "version source identity"
    ("vendored_sha256_source_identity " ^ Ext_sha256.source_identity ^ "\n")
    version.stdout;
  assert_contains "version manifest signature"
    "checker_identity_manifest_signature_required false\n" version.stdout;
  assert_equal "version stderr" "" version.stderr;

  assert_cli_error "no args" "missing required --cert" [];
  assert_cli_error "version mixed" "--version must be used alone" [ "--version"; "--output"; "json" ];
  assert_cli_error "source cert path" "--cert must not point to .npa source"
    [ "--cert"; "example.npa"; "--import-dir"; "imports"; "--policy"; "policy.toml"; "--output"; "json" ];
  assert_cli_error "source policy path" "--policy must not point to .npa source"
    [ "--cert"; "example.npcert"; "--import-dir"; "imports"; "--policy"; "policy.npa"; "--output"; "json" ];
  assert_cli_error "source import dir" "--import-dir must not point to .npa source"
    [ "--cert"; "example.npcert"; "--import-dir"; "src/module.npa/imports"; "--policy"; "policy.toml"; "--output"; "json" ];
  assert_cli_error "bad output" "--output must be json"
    [ "--cert"; "example.npcert"; "--import-dir"; "imports"; "--policy"; "policy.toml"; "--output"; "pretty" ];
  assert_cli_error "duplicate cert" "duplicate --cert"
    [
      "--cert";
      "a.npcert";
      "--cert";
      "b.npcert";
      "--import-dir";
      "imports";
      "--policy";
      "policy.toml";
      "--output";
      "json";
    ];
  assert_cli_error "missing cert value" "missing value for --cert"
    [ "--cert"; "--import-dir"; "imports"; "--policy"; "policy.toml"; "--output"; "json" ];
  assert_cli_error "missing output value" "missing value for --output"
    [ "--cert"; "example.npcert"; "--import-dir"; "imports"; "--policy"; "policy.toml"; "--output"; "--policy" ];
  assert_cli_error "unknown flag" "unknown flag --audit-bundle" [ "--audit-bundle"; "bundle" ];
  assert_cli_error "positional source" "positional .npa source input is forbidden" [ "example.npa" ];
  assert_cli_error "positional input" "positional input is forbidden" [ "example.npcert" ];

  let check_shape =
    Ext_cli.run
      [
        "--cert";
        "example.npcert";
        "--import-dir";
        "imports";
        "--policy";
        "policy.toml";
        "--output";
        "json";
      ]
  in
  assert_int_equal "check shape exit" 0 check_shape.code;
  assert_equal "check shape stderr" "" check_shape.stderr;
  assert_contains "check shape schema" "\"schema\": \"npa.independent-checker.checker_raw_result.v1\""
    check_shape.stdout;
  assert_contains "check shape status" "\"status\": \"failed\"" check_shape.stdout;
  assert_contains "check shape error" "\"kind\": \"checker_internal_error\""
    check_shape.stdout

let assert_feature_policy_rejects_quotient feature offset expected_kind =
  assert_bool (feature ^ " is a quotient feature profile")
    (Ext_feature.is_quotient_feature_profile feature);
  assert_bool (feature ^ " is not supported in first release")
    (not (Ext_feature.is_supported_first_release feature));
  assert_equal (feature ^ " fixture expected kind") "unsupported_core_feature" expected_kind;
  let report = [ { Ext_feature.feature; offset = Some offset } ] in
  match Ext_feature.raw_result_for_first_release_report report with
  | None -> failwith (feature ^ ": expected unsupported_core_feature raw result")
  | Some raw ->
      assert_contains (feature ^ " failed status") "\"status\": \"failed\"" raw;
      assert_contains (feature ^ " unsupported kind")
        ("\"kind\": \"" ^ expected_kind ^ "\"") raw;
      assert_contains (feature ^ " unsupported reason")
        ("\"reason_code\": \"" ^ expected_kind ^ "\"") raw;
      assert_contains (feature ^ " section") "\"section\": \"core_features\"" raw;
      assert_contains (feature ^ " offset") ("\"offset\": " ^ string_of_int offset) raw

let run_feature_policy_fixture_tests () =
  let path = Filename.concat (root_dir ()) "test/fixtures/feature_policy.tsv" in
  let channel = open_in path in
  let count = ref 0 in
  (try
     while true do
       let line = input_line channel in
       if String.length line > 0 && line.[0] <> '#' then
         match split_tabs line with
         | [ feature; offset_text; expected_kind ] ->
             assert_feature_policy_rejects_quotient feature (int_of_string offset_text)
               expected_kind;
             incr count
         | _ -> failwith ("malformed feature policy fixture line: " ^ line)
     done
   with End_of_file -> close_in channel);
  assert_int_equal "feature policy fixture count" 3 !count

let run_feature_policy_tests () =
  assert_equal "feature policy input shape"
    "canonical-certificate-feature-report-only" Ext_feature.policy_input_shape;
  assert_bool "first-release supported core features are empty"
    (Ext_feature.supported_core_features = []);
  (match Ext_feature.check_first_release_report [] with
  | Ext_feature.Feature_policy_ok -> ()
  | Ext_feature.Unsupported_core_feature _ ->
      failwith "empty MVP feature report must not be rejected");
  assert_bool "empty MVP report has no raw failure"
    (Ext_feature.raw_result_for_first_release_report [] = None);
  run_feature_policy_fixture_tests ()

let decode_error_raw_result error =
  Ext_result.decode_error error

let assert_decode_error label expected_kind expected_reason expected_section expected_offset result =
  match result with
  | Ok _ -> failwith (label ^ ": expected decode error")
  | Error error ->
      assert_equal (label ^ " stable kind") expected_kind (Ext_result.decode_error_kind error);
      assert_equal (label ^ " reason") (Ext_bytes.reason_code expected_reason)
        (Ext_bytes.reason_code error.Ext_bytes.reason);
      assert_equal (label ^ " section") (Ext_bytes.section_name expected_section)
        (Ext_bytes.section_name error.Ext_bytes.section);
      assert_int_equal (label ^ " offset") expected_offset error.Ext_bytes.offset;
      let raw = decode_error_raw_result error in
      assert_contains (label ^ " raw kind") ("\"kind\": \"" ^ expected_kind ^ "\"") raw;
      assert_contains (label ^ " raw reason")
        ("\"reason_code\": \"" ^ Ext_bytes.reason_code expected_reason ^ "\"")
        raw;
      assert_contains (label ^ " raw section")
        ("\"section\": \"" ^ Ext_bytes.section_name expected_section ^ "\"")
        raw;
      assert_contains (label ^ " raw offset") ("\"offset\": " ^ string_of_int expected_offset)
        raw

let assert_read_uvar label codes expected offset =
  let reader = Ext_bytes.of_bytes (bytes_of_codes codes) in
  match Ext_bytes.read_uvar Ext_bytes.Imports reader with
  | Error error ->
      failwith
        (label ^ ": unexpected decode error " ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (actual, next) ->
      assert_int64_equal (label ^ " value") expected actual;
      assert_int_equal (label ^ " offset") offset (Ext_bytes.offset next);
      assert_int_equal (label ^ " original offset") 0 (Ext_bytes.offset reader)

let run_decoder_bytes_tests () =
  let mutable_input = Bytes.of_string "ab" in
  let reader = Ext_bytes.of_bytes mutable_input in
  Bytes.set mutable_input 0 'z';
  (match Ext_bytes.read_byte Ext_bytes.Full_certificate reader with
  | Error error ->
      failwith
        ("immutable reader byte: unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (byte, next) ->
      assert_int_equal "immutable reader copied input" (Char.code 'a') byte;
      assert_int_equal "immutable reader original offset" 0 (Ext_bytes.offset reader);
      assert_int_equal "immutable reader next offset" 1 (Ext_bytes.offset next));

  (match Ext_bytes.take Ext_bytes.Full_certificate 2 (Ext_bytes.of_string "abcd") with
  | Error error ->
      failwith ("take: unexpected decode error " ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (taken, next) ->
      assert_equal "take bytes" "ab" taken;
      assert_int_equal "take offset" 2 (Ext_bytes.offset next);
      assert_int_equal "take remaining" 2 (Ext_bytes.remaining next));

  assert_read_uvar "uvar zero" [ 0x00 ] 0L 1;
  assert_read_uvar "uvar one-byte max" [ 0x7f ] 127L 1;
  assert_read_uvar "uvar 128" [ 0x80; 0x01 ] 128L 2;
  assert_read_uvar "uvar 300" [ 0xac; 0x02 ] 300L 2;
  assert_read_uvar "uvar u64 max"
    [ 0xff; 0xff; 0xff; 0xff; 0xff; 0xff; 0xff; 0xff; 0xff; 0x01 ]
    Int64.minus_one 10;

  assert_decode_error "empty input" "certificate_decode_error"
    Ext_bytes.Unexpected_eof Ext_bytes.Full_certificate 0
    (Ext_bytes.read_byte Ext_bytes.Full_certificate Ext_bytes.empty);
  assert_decode_error "noncanonical zero" "noncanonical_encoding"
    Ext_bytes.Noncanonical_uvar Ext_bytes.Imports 1
    (Ext_bytes.read_uvar Ext_bytes.Imports (Ext_bytes.of_bytes (bytes_of_codes [ 0x80; 0x00 ])));
  assert_decode_error "overlong one" "noncanonical_encoding"
    Ext_bytes.Noncanonical_uvar Ext_bytes.Imports 1
    (Ext_bytes.read_uvar Ext_bytes.Imports (Ext_bytes.of_bytes (bytes_of_codes [ 0x81; 0x00 ])));
  assert_decode_error "uvar eof after continuation" "certificate_decode_error"
    Ext_bytes.Unexpected_eof Ext_bytes.Imports 1
    (Ext_bytes.read_uvar Ext_bytes.Imports (Ext_bytes.of_bytes (bytes_of_codes [ 0x80 ])));
  assert_decode_error "take eof" "certificate_decode_error" Ext_bytes.Unexpected_eof
    Ext_bytes.Full_certificate 1
    (Ext_bytes.take Ext_bytes.Full_certificate 2 (Ext_bytes.of_string "a"));
  assert_decode_error "uvar overflow" "certificate_decode_error" Ext_bytes.Uvar_overflow
    Ext_bytes.Imports 9
    (Ext_bytes.read_uvar Ext_bytes.Imports
       (Ext_bytes.of_bytes
          (bytes_of_codes
             [ 0xff; 0xff; 0xff; 0xff; 0xff; 0xff; 0xff; 0xff; 0xff; 0x02 ])));
  let usize_overflow = Ext_bytes.encode_uvar (Int64.add (Int64.of_int max_int) 1L) in
  assert_decode_error "usize overflow" "certificate_decode_error" Ext_bytes.Length_overflow
    Ext_bytes.Imports (String.length usize_overflow - 1)
    (Ext_bytes.read_usize Ext_bytes.Imports (Ext_bytes.of_string usize_overflow))

let encode_uvar_int value = Ext_bytes.encode_uvar (Int64.of_int value)

let encode_string text = encode_uvar_int (String.length text) ^ text

let encode_raw_string text = encode_uvar_int (String.length text) ^ text

let encode_name components =
  encode_uvar_int (List.length components) ^ String.concat "" (List.map encode_string components)

let make_name components =
  match Ext_name.of_components components with
  | None -> failwith "test fixture constructed an invalid name"
  | Some name -> name

let one_byte code = String.make 1 (Char.chr code)

let hash_bytes fill = String.make 32 (Char.chr fill)

let encode_level_zero = one_byte 0x00

let encode_level_succ inner = one_byte 0x01 ^ encode_uvar_int inner

let encode_level_max lhs rhs = one_byte 0x02 ^ encode_uvar_int lhs ^ encode_uvar_int rhs

let encode_level_imax lhs rhs = one_byte 0x03 ^ encode_uvar_int lhs ^ encode_uvar_int rhs

let encode_level_param name_id = one_byte 0x04 ^ encode_uvar_int name_id

let encode_term_sort level_id = one_byte 0x00 ^ encode_uvar_int level_id

let encode_term_bvar index = one_byte 0x01 ^ encode_uvar_int index

let encode_term_const global_ref levels =
  one_byte 0x02 ^ global_ref ^ encode_uvar_int (List.length levels)
  ^ String.concat "" (List.map encode_uvar_int levels)

let encode_term_app fn arg = one_byte 0x03 ^ encode_uvar_int fn ^ encode_uvar_int arg

let encode_term_lam ty body = one_byte 0x04 ^ encode_uvar_int ty ^ encode_uvar_int body

let encode_term_pi ty body = one_byte 0x05 ^ encode_uvar_int ty ^ encode_uvar_int body

let encode_term_let ty value body =
  one_byte 0x06 ^ encode_uvar_int ty ^ encode_uvar_int value ^ encode_uvar_int body

let encode_global_builtin name_id hash = one_byte 0x03 ^ encode_uvar_int name_id ^ hash

let encode_global_imported import_index name_id hash =
  one_byte 0x00 ^ encode_uvar_int import_index ^ encode_uvar_int name_id ^ hash

let encode_global_local decl_index = one_byte 0x01 ^ encode_uvar_int decl_index

let encode_usize_vec values =
  encode_uvar_int (List.length values) ^ String.concat "" (List.map encode_uvar_int values)

let encode_option payload =
  match payload with
  | None -> one_byte 0x00
  | Some value -> one_byte 0x01 ^ value

let encode_option_usize value =
  match value with
  | None -> encode_option None
  | Some value -> encode_option (Some (encode_uvar_int value))

let encode_option_hash value = encode_option value

let encode_reducibility reducibility =
  match reducibility with
  | `Reducible -> one_byte 0x00
  | `Opaque -> one_byte 0x01

let encode_opacity_opaque = one_byte 0x00

let encode_imports imports =
  encode_uvar_int (List.length imports)
  ^ String.concat ""
      (List.map
         (fun (module_components, export_hash, certificate_hash) ->
           encode_name module_components ^ export_hash ^ encode_option_hash certificate_hash)
         imports)

let encode_name_table names =
  encode_uvar_int (List.length names) ^ String.concat "" (List.map encode_name names)

let encode_level_table entries = encode_uvar_int (List.length entries) ^ String.concat "" entries

let encode_term_table entries = encode_uvar_int (List.length entries) ^ String.concat "" entries

let encode_dependency_entries entries =
  encode_uvar_int (List.length entries)
  ^ String.concat ""
      (List.map
         (fun (global_ref, decl_interface_hash) -> global_ref ^ decl_interface_hash)
         entries)

let encode_axiom_refs refs =
  encode_uvar_int (List.length refs)
  ^ String.concat ""
      (List.map
         (fun (global_ref, name_id, decl_interface_hash) ->
           global_ref ^ encode_uvar_int name_id ^ decl_interface_hash)
         refs)

let encode_axiom_decl_payload name_id universe_params ty =
  one_byte 0x00 ^ encode_uvar_int name_id ^ encode_usize_vec universe_params
  ^ encode_uvar_int ty

let encode_universe_constraints constraints =
  encode_uvar_int (List.length constraints)
  ^ String.concat ""
      (List.map
         (fun (lhs, relation_tag, rhs) ->
           encode_uvar_int lhs ^ one_byte relation_tag ^ encode_uvar_int rhs)
         constraints)

let encode_constrained_axiom_decl_payload name_id universe_params constraints ty =
  one_byte 0x10 ^ encode_uvar_int name_id ^ encode_usize_vec universe_params
  ^ encode_universe_constraints constraints ^ encode_uvar_int ty

let encode_def_decl_payload tag name_id universe_params ?(constraints = None) ty value
    reducibility =
  one_byte tag ^ encode_uvar_int name_id ^ encode_usize_vec universe_params
  ^ (match constraints with
    | None -> ""
    | Some constraints -> encode_universe_constraints constraints)
  ^ encode_uvar_int ty ^ encode_uvar_int value ^ encode_reducibility reducibility

let encode_theorem_decl_payload tag name_id universe_params ?(constraints = None) ty proof =
  one_byte tag ^ encode_uvar_int name_id ^ encode_usize_vec universe_params
  ^ (match constraints with
    | None -> ""
    | Some constraints -> encode_universe_constraints constraints)
  ^ encode_uvar_int ty ^ encode_uvar_int proof ^ encode_opacity_opaque

let encode_binder_types term_ids =
  encode_uvar_int (List.length term_ids) ^ String.concat "" (List.map encode_uvar_int term_ids)

let encode_constructor_specs constructors =
  encode_uvar_int (List.length constructors)
  ^ String.concat ""
      (List.map
         (fun (name_id, ty) -> encode_uvar_int name_id ^ encode_uvar_int ty)
         constructors)

let encode_recursor_spec spec =
  match spec with
  | None -> one_byte 0x00
  | Some (name_id, universe_params, ty, minor_start, major_index) ->
      one_byte 0x01 ^ encode_uvar_int name_id ^ encode_usize_vec universe_params
      ^ encode_uvar_int ty ^ encode_uvar_int minor_start ^ encode_uvar_int major_index

let encode_inductive_decl_payload tag name_id universe_params ?(constraints = None) params
    indices sort constructors recursor =
  one_byte tag ^ encode_uvar_int name_id ^ encode_usize_vec universe_params
  ^ (match constraints with
    | None -> ""
    | Some constraints -> encode_universe_constraints constraints)
  ^ encode_binder_types params ^ encode_binder_types indices ^ encode_uvar_int sort
  ^ encode_constructor_specs constructors ^ encode_recursor_spec recursor

let encode_mutual_inductive_spec name_id params indices sort constructors recursor =
  encode_uvar_int name_id ^ encode_binder_types params ^ encode_binder_types indices
  ^ encode_uvar_int sort ^ encode_constructor_specs constructors ^ encode_recursor_spec recursor

let encode_mutual_inductive_block_payload name_id universe_params constraints inductives =
  one_byte 0x04 ^ encode_uvar_int name_id ^ encode_usize_vec universe_params
  ^ encode_universe_constraints constraints ^ encode_uvar_int (List.length inductives)
  ^ String.concat "" inductives

let encode_decl_cert payload dependencies axiom_dependencies interface_hash certificate_hash =
  payload ^ encode_dependency_entries dependencies ^ encode_axiom_refs axiom_dependencies
  ^ interface_hash ^ certificate_hash

let encode_declarations entries =
  encode_uvar_int (List.length entries) ^ String.concat "" entries

let encode_export_kind tag = one_byte tag

let encode_export_entry_prefix name_id kind_tag universe_params ty body =
  encode_uvar_int name_id ^ encode_export_kind kind_tag ^ encode_usize_vec universe_params
  ^ encode_uvar_int ty ^ encode_option_usize body ^ hash_bytes 0x31 ^ encode_option_hash None
  ^ encode_option None ^ encode_option None ^ hash_bytes 0x32

let encode_export_entry name_id kind_tag universe_params ty body axiom_dependencies =
  encode_export_entry_prefix name_id kind_tag universe_params ty body
  ^ encode_axiom_refs axiom_dependencies

let encode_export_block entries =
  encode_uvar_int (List.length entries) ^ String.concat "" entries

let encode_axiom_report per_declaration module_axioms =
  encode_uvar_int (List.length per_declaration)
  ^ String.concat ""
      (List.map
         (fun (decl_index, direct_axioms, transitive_axioms) ->
           encode_uvar_int decl_index ^ encode_axiom_refs direct_axioms
           ^ encode_axiom_refs transitive_axioms)
         per_declaration)
  ^ encode_axiom_refs module_axioms

let encode_core_features features =
  encode_string "core_features" ^ encode_uvar_int (List.length features)
  ^ String.concat "" (List.map encode_string features)

let encode_hashes = hash_bytes 0xa1 ^ hash_bytes 0xa2 ^ hash_bytes 0xa3

let encode_header ?(format = Ext_cert.expected_format)
    ?(core_spec = Ext_cert.expected_core_spec) module_components =
  encode_string format ^ encode_string core_spec ^ encode_name module_components

let read_binary_file path =
  let channel = open_in_bin path in
  let length = in_channel_length channel in
  let contents = really_input_string channel length in
  close_in channel;
  contents

type golden_hash_fixture = {
  golden_byte_len : int;
  golden_export_hash : string;
  golden_axiom_report_hash : string;
  golden_certificate_hash : string;
}

let golden_hash_fixture label =
  let path =
    Filename.concat (root_dir ()) "../../crates/npa-cert/tests/fixtures/golden_hashes.tsv"
  in
  let contents = read_binary_file path in
  let rec loop lines =
    match lines with
    | [] -> failwith ("missing golden hash fixture " ^ label)
    | line :: rest ->
        if line = "" || contains line "label\t" then loop rest
        else (
          match split_tabs line with
          | [ current; byte_len; export_hash; axiom_report_hash; certificate_hash ]
            when current = label ->
              {
                golden_byte_len = int_of_string byte_len;
                golden_export_hash = export_hash;
                golden_axiom_report_hash = axiom_report_hash;
                golden_certificate_hash = certificate_hash;
              }
          | _ -> loop rest)
  in
  loop (String.split_on_char '\n' contents)

let hex_of_raw_hash hash = Ext_sha256.to_hex (Bytes.of_string hash)

let decode_module_bytes label bytes =
  match Ext_cert.read_module (Ext_bytes.of_string bytes) with
  | Ok (decoded, next) ->
      assert_int_equal (label ^ " offset") (String.length bytes) (Ext_bytes.offset next);
      decoded
  | Error error ->
      failwith
        (label ^ ": unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason ^ " at "
       ^ Ext_bytes.section_name error.Ext_bytes.section ^ ":"
       ^ string_of_int error.Ext_bytes.offset)

let assert_header label expected_module header =
  assert_equal (label ^ " format") Ext_cert.expected_format header.Ext_cert.format;
  assert_equal (label ^ " core spec") Ext_cert.expected_core_spec header.Ext_cert.core_spec;
  assert_equal (label ^ " module") expected_module (Ext_name.to_string header.Ext_cert.module_name)

let run_decoder_header_tests () =
  let golden_path =
    Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Nat/Basic/certificate.npcert"
  in
  let golden = read_binary_file golden_path in
  (match Ext_cert.read_header (Ext_bytes.of_string golden) with
  | Error error ->
      failwith ("golden header: unexpected decode error " ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (header, next) ->
      assert_equal "golden header format" Ext_cert.expected_format header.Ext_cert.format;
      assert_equal "golden header core spec" Ext_cert.expected_core_spec header.Ext_cert.core_spec;
      assert_bool "golden header module is structured"
        (String.length (Ext_name.to_string header.Ext_cert.module_name) > 0);
      assert_bool "golden header advances reader" (Ext_bytes.offset next > 0));

  let valid_header = encode_header [ "Std"; "Nat" ] in
  (match Ext_cert.read_header (Ext_bytes.of_string valid_header) with
  | Error error ->
      failwith ("valid header: unexpected decode error " ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (header, next) ->
      assert_header "valid header" "Std.Nat" header;
      assert_int_equal "valid header offset" (String.length valid_header) (Ext_bytes.offset next));

  let bad_format = encode_header ~format:"BAD-CERT" [ "Std"; "Nat" ] in
  assert_decode_error "format mismatch" "certificate_decode_error" Ext_bytes.Format_mismatch
    Ext_bytes.Header_format (String.length (encode_string "BAD-CERT"))
    (Ext_cert.read_header (Ext_bytes.of_string bad_format));

  let core_prefix = encode_string Ext_cert.expected_format ^ encode_string "NPA-Core-X" in
  let bad_core = core_prefix ^ encode_name [ "Std"; "Nat" ] in
  assert_decode_error "core spec mismatch" "certificate_decode_error"
    Ext_bytes.Core_spec_mismatch Ext_bytes.Header_core_spec (String.length core_prefix)
    (Ext_cert.read_header (Ext_bytes.of_string bad_core));

  let invalid_utf8 = encode_raw_string (string_of_codes [ 0xff ]) in
  assert_decode_error "invalid utf8 header" "noncanonical_encoding" Ext_bytes.Invalid_utf8
    Ext_bytes.Header_format 1
    (Ext_cert.read_header (Ext_bytes.of_string invalid_utf8));

  let empty_module_prefix =
    encode_string Ext_cert.expected_format ^ encode_string Ext_cert.expected_core_spec
  in
  let empty_module = empty_module_prefix ^ encode_uvar_int 0 in
  assert_decode_error "empty module name" "noncanonical_encoding" Ext_bytes.Empty_name
    Ext_bytes.Header_module (String.length empty_module_prefix)
    (Ext_cert.read_header (Ext_bytes.of_string empty_module));

  let empty_component_prefix = empty_module_prefix ^ encode_uvar_int 1 in
  let empty_component = empty_component_prefix ^ encode_string "" in
  assert_decode_error "empty name component" "noncanonical_encoding"
    Ext_bytes.Empty_name_component Ext_bytes.Header_module (String.length empty_component_prefix)
    (Ext_cert.read_header (Ext_bytes.of_string empty_component));

  let dotted_component_prefix = empty_module_prefix ^ encode_uvar_int 1 ^ encode_uvar_int 7 in
  let dotted_component = dotted_component_prefix ^ "Std.Nat" in
  assert_decode_error "dotted name component" "noncanonical_encoding"
    Ext_bytes.Dotted_name_component Ext_bytes.Header_module
    (String.length dotted_component_prefix + 3)
    (Ext_cert.read_header (Ext_bytes.of_string dotted_component));

  let name_table = encode_uvar_int 2 ^ encode_name [ "A" ] ^ encode_name [ "Std"; "Nat" ] in
  (match Ext_cert.read_name_table (Ext_bytes.of_string name_table) with
  | Error error ->
      failwith ("name table: unexpected decode error " ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (entries, next) ->
      assert_int_equal "name table length" 2 (List.length entries);
      assert_equal "name table first name" "A" (Ext_name.to_string (List.hd entries).Ext_cert.name);
      assert_int_equal "name table offset" (String.length name_table) (Ext_bytes.offset next));

  let duplicate_entry = encode_name [ "A" ] in
  let duplicate_name_table = encode_uvar_int 2 ^ duplicate_entry ^ duplicate_entry in
  assert_decode_error "duplicate name table entry" "noncanonical_encoding" Ext_bytes.Duplicate_name
    Ext_bytes.Name_table (String.length (encode_uvar_int 2 ^ duplicate_entry))
    (Ext_cert.read_name_table (Ext_bytes.of_string duplicate_name_table))

let level_value (entry : Ext_level.located) = entry.level

let term_value (entry : Ext_term.located) = entry.term

let run_decoder_tables_tests () =
  let universe_name = make_name [ "u" ] in
  let nat_name = make_name [ "Nat" ] in
  let names = [ universe_name; nat_name ] in
  let valid_level_table =
    encode_uvar_int 3 ^ encode_level_zero ^ encode_level_param 0 ^ encode_level_succ 0
  in
  let levels =
    match Ext_level.read_table names (Ext_bytes.of_string valid_level_table) with
    | Error error ->
        failwith
          ("valid level table: unexpected decode error "
         ^ Ext_bytes.reason_code error.Ext_bytes.reason)
    | Ok (levels, next) ->
        assert_int_equal "valid level table offset" (String.length valid_level_table)
          (Ext_bytes.offset next);
        assert_int_equal "valid level table length" 3 (List.length levels);
        levels
  in
  (match List.map level_value levels with
  | [ Ext_level.Zero; Ext_level.Param name; Ext_level.Succ Ext_level.Zero ] ->
      assert_equal "valid level param name" "u" (Ext_name.to_string name)
  | _ -> failwith "valid level table did not decode into structured level AST");

  let builtin_nat = encode_global_builtin 1 (hash_bytes 0x42) in
  let valid_term_table =
    encode_uvar_int 7 ^ encode_term_sort 0 ^ encode_term_bvar 0
    ^ encode_term_const builtin_nat [ 0; 1 ]
    ^ encode_term_app 2 1 ^ encode_term_lam 0 3 ^ encode_term_pi 0 4
    ^ encode_term_let 0 1 5
  in
  let terms =
    match Ext_term.read_table names levels (Ext_bytes.of_string valid_term_table) with
    | Error error ->
        failwith
          ("valid term table: unexpected decode error "
         ^ Ext_bytes.reason_code error.Ext_bytes.reason)
    | Ok (terms, next) ->
        assert_int_equal "valid term table offset" (String.length valid_term_table)
          (Ext_bytes.offset next);
        assert_int_equal "valid term table length" 7 (List.length terms);
        terms
  in
  (match List.map term_value terms with
  | [
   Ext_term.Sort Ext_level.Zero;
   Ext_term.BVar 0;
   Ext_term.Const
     (Ext_term.Builtin { name; decl_interface_hash }, [ Ext_level.Zero; Ext_level.Param _ ]);
   Ext_term.App (_, _);
   Ext_term.Lam (_, _);
   Ext_term.Pi (_, _);
   Ext_term.Let (_, _, _);
  ] ->
      assert_equal "valid term const builtin name" "Nat" (Ext_name.to_string name);
      assert_int_equal "valid term const hash length" 32 (String.length decl_interface_hash)
  | _ -> failwith "valid term table did not decode into structured term AST");

  assert_decode_error "unknown level tag" "certificate_decode_error"
    (Ext_bytes.Unknown_tag 0xff) Ext_bytes.Level_table 1
    (Ext_level.read_table names (Ext_bytes.of_string (encode_uvar_int 1 ^ one_byte 0xff)));
  assert_decode_error "level table length exceeds payload" "certificate_decode_error"
    Ext_bytes.Unexpected_eof Ext_bytes.Level_table 1
    (Ext_level.read_table names (Ext_bytes.of_string (encode_uvar_int 2 ^ encode_level_zero)));
  assert_decode_error "dangling level self reference" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Level_table 1
    (Ext_level.read_table names (Ext_bytes.of_string (encode_uvar_int 1 ^ encode_level_succ 0)));
  assert_decode_error "dangling level name reference" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Level_table 1
    (Ext_level.read_table [ universe_name ]
       (Ext_bytes.of_string (encode_uvar_int 1 ^ encode_level_param 1)));
  assert_decode_error "non-normalized max zero" "noncanonical_encoding"
    Ext_bytes.Non_normalized_level Ext_bytes.Level_table 4
    (Ext_level.read_table [ universe_name ]
       (Ext_bytes.of_string
          (encode_uvar_int 3 ^ encode_level_zero ^ encode_level_param 0
         ^ encode_level_max 0 1)));
  assert_decode_error "duplicate level entry" "noncanonical_encoding"
    Ext_bytes.Noncanonical_order Ext_bytes.Level_table 2
    (Ext_level.read_table names
       (Ext_bytes.of_string (encode_uvar_int 2 ^ encode_level_zero ^ encode_level_zero)));
  assert_decode_error "unresolved universe metavariable" "certificate_decode_error"
    Ext_bytes.Unresolved_metavariable Ext_bytes.Level_table 1
    (Ext_level.read_table [ make_name [ "z?meta" ] ]
       (Ext_bytes.of_string (encode_uvar_int 1 ^ encode_level_param 0)));
  assert_decode_error "unresolved human universe metavariable" "certificate_decode_error"
    Ext_bytes.Unresolved_metavariable Ext_bytes.Level_table 1
    (Ext_level.read_table [ make_name [ "__npa_internal_human_universe_meta#0" ] ]
       (Ext_bytes.of_string (encode_uvar_int 1 ^ encode_level_param 0)));

  assert_decode_error "unknown term tag" "certificate_decode_error"
    (Ext_bytes.Unknown_tag 0xff) Ext_bytes.Term_table 1
    (Ext_term.read_table names levels (Ext_bytes.of_string (encode_uvar_int 1 ^ one_byte 0xff)));
  assert_decode_error "term table length exceeds payload" "certificate_decode_error"
    Ext_bytes.Unexpected_eof Ext_bytes.Term_table 1
    (Ext_term.read_table names levels
       (Ext_bytes.of_string (encode_uvar_int 2 ^ one_byte 0x01)));
  assert_decode_error "dangling term level reference" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Term_table 1
    (Ext_term.read_table names [] (Ext_bytes.of_string (encode_uvar_int 1 ^ encode_term_sort 0)));
  assert_decode_error "dangling term self reference" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Term_table 1
    (Ext_term.read_table names levels
       (Ext_bytes.of_string (encode_uvar_int 1 ^ encode_term_app 0 0)));
  assert_decode_error "unknown global ref tag" "certificate_decode_error"
    (Ext_bytes.Unknown_tag 0xfe) Ext_bytes.Term_table 2
    (Ext_term.read_table names levels
       (Ext_bytes.of_string (encode_uvar_int 1 ^ one_byte 0x02 ^ one_byte 0xfe)));
  assert_decode_error "dangling global ref name" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Term_table 1
    (Ext_term.read_table names levels
       (Ext_bytes.of_string
          (encode_uvar_int 1 ^ encode_term_const (encode_global_builtin 9 (hash_bytes 0x01)) [])));
  assert_decode_error "duplicate term entry" "noncanonical_encoding"
    Ext_bytes.Non_normalized_term Ext_bytes.Term_table 3
    (Ext_term.read_table names levels
       (Ext_bytes.of_string (encode_uvar_int 2 ^ encode_term_sort 0 ^ encode_term_sort 0)))

let simple_level_table = [ { Ext_level.level = Ext_level.Zero; offset = 0 } ]

let simple_term_table = [ { Ext_term.term = Ext_term.Sort Ext_level.Zero; offset = 0 } ]

let encode_module ?(core_features = []) ?(axiom_report = encode_axiom_report [] [])
    ?(module_name = [ "M" ]) ?(imports = []) name_entries level_entries term_entries
    declarations export_entries =
  encode_header module_name ^ encode_imports imports ^ encode_name_table name_entries
  ^ encode_level_table level_entries ^ encode_term_table term_entries
  ^ encode_declarations declarations ^ encode_export_block export_entries ^ axiom_report
  ^ (if core_features = [] then "" else encode_core_features core_features)
  ^ encode_hashes

let encode_minimal_module ?(core_features = []) ?(axiom_report = encode_axiom_report [] [])
    declarations export_entries =
  encode_module ~core_features ~axiom_report [ [ "A" ] ] [ encode_level_zero ]
    [ encode_term_sort 0 ] declarations export_entries

let minimal_axiom_decl =
  encode_decl_cert (encode_axiom_decl_payload 0 [] 0) [] [] (hash_bytes 0x11) (hash_bytes 0x12)

let minimal_export_entry = encode_export_entry 0 0x00 [] 0 None []

let assert_decoded_minimal label decoded expected_feature_count =
  assert_equal (label ^ " module") "M"
    (Ext_name.to_string decoded.Ext_cert.header.Ext_cert.module_name);
  assert_int_equal (label ^ " imports") 0 (List.length decoded.Ext_cert.imports);
  assert_int_equal (label ^ " names") 1 (List.length decoded.Ext_cert.name_table);
  assert_int_equal (label ^ " levels") 1 (List.length decoded.Ext_cert.level_table);
  assert_int_equal (label ^ " terms") 1 (List.length decoded.Ext_cert.term_table);
  assert_int_equal (label ^ " declarations") 1
    (List.length decoded.Ext_cert.declaration_table);
  assert_int_equal (label ^ " exports") 1 (List.length decoded.Ext_cert.export_block);
  assert_int_equal (label ^ " axiom report mismatch preserved") 0
    (List.length decoded.Ext_cert.axiom_report.Ext_cert.per_declaration);
  assert_int_equal (label ^ " feature count") expected_feature_count
    (List.length decoded.Ext_cert.axiom_report.Ext_cert.core_features);
  assert_int_equal (label ^ " export hash length") 32
    (String.length decoded.Ext_cert.hashes.Ext_cert.export_hash);
  assert_int_equal (label ^ " axiom report hash length") 32
    (String.length decoded.Ext_cert.hashes.Ext_cert.axiom_report_hash);
  assert_int_equal (label ^ " certificate hash length") 32
    (String.length decoded.Ext_cert.hashes.Ext_cert.certificate_hash)

let run_decoder_declarations_tests () =
  let golden_path =
    Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Nat/Basic/certificate.npcert"
  in
  let golden = read_binary_file golden_path in
  (match Ext_cert.read_module (Ext_bytes.of_string golden) with
  | Error error ->
      failwith
        ("golden module: unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason ^ " at "
       ^ Ext_bytes.section_name error.Ext_bytes.section ^ ":"
       ^ string_of_int error.Ext_bytes.offset)
  | Ok (decoded, next) ->
      assert_bool "golden module has declarations"
        (List.length decoded.Ext_cert.declaration_table > 0);
      assert_bool "golden module has exports" (List.length decoded.Ext_cert.export_block > 0);
      assert_int_equal "golden module offset" (String.length golden) (Ext_bytes.offset next));

  let minimal = encode_minimal_module [ minimal_axiom_decl ] [ minimal_export_entry ] in
  (match Ext_cert.read_module (Ext_bytes.of_string minimal) with
  | Error error ->
      failwith
        ("minimal module: unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (decoded, next) ->
      assert_decoded_minimal "minimal module" decoded 0;
      assert_int_equal "minimal module offset" (String.length minimal) (Ext_bytes.offset next));

  let feature_module =
    encode_minimal_module ~core_features:[ "quotient_v1" ] [ minimal_axiom_decl ]
      [ minimal_export_entry ]
  in
  (match Ext_cert.read_module (Ext_bytes.of_string feature_module) with
  | Error error ->
      failwith
        ("feature module: unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (decoded, next) ->
      assert_decoded_minimal "feature module" decoded 1;
      assert_equal "feature name" "quotient_v1"
        (List.hd decoded.Ext_cert.axiom_report.Ext_cert.core_features).Ext_feature.feature;
      assert_int_equal "feature module offset" (String.length feature_module)
        (Ext_bytes.offset next));

  let variant_names =
    List.map
      (fun name -> make_name [ name ])
      [ "A0"; "A1"; "D0"; "D1"; "T0"; "T1"; "I0"; "I1"; "M0"; "C"; "R" ]
  in
  let constraints = [ (0, 0x00, 0) ] in
  let constructor = [ (9, 0) ] in
  let recursor = Some (10, [], 0, 0, 0) in
  let variant_payloads =
    [
      encode_axiom_decl_payload 0 [] 0;
      encode_constrained_axiom_decl_payload 1 [] constraints 0;
      encode_def_decl_payload 0x01 2 [] 0 0 `Reducible;
      encode_def_decl_payload 0x11 3 [] ~constraints:(Some constraints) 0 0 `Opaque;
      encode_theorem_decl_payload 0x02 4 [] 0 0;
      encode_theorem_decl_payload 0x12 5 [] ~constraints:(Some constraints) 0 0;
      encode_inductive_decl_payload 0x03 6 [] [] [] 0 constructor recursor;
      encode_inductive_decl_payload 0x13 7 [] ~constraints:(Some constraints) [] [] 0
        constructor recursor;
      encode_mutual_inductive_block_payload 8 [] constraints
        [ encode_mutual_inductive_spec 6 [] [] 0 constructor recursor ];
    ]
  in
  let variant_declarations =
    encode_declarations
      (List.mapi
         (fun index payload ->
           encode_decl_cert payload [] [] (hash_bytes (0x60 + index)) (hash_bytes (0x70 + index)))
         variant_payloads)
  in
  (match
     Ext_cert.read_declarations 0 variant_names simple_level_table simple_term_table
       (Ext_bytes.of_string variant_declarations)
   with
  | Error error ->
      failwith
        ("variant declarations: unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (declarations, next) ->
      assert_int_equal "variant declaration count" 9 (List.length declarations);
      assert_int_equal "variant declaration offset" (String.length variant_declarations)
        (Ext_bytes.offset next);
      assert_bool "variant declarations include mutual block"
        (List.exists
           (fun declaration -> declaration.Ext_cert.kind = Ext_cert.Mutual_inductive)
           declarations));

  let duplicate_declarations =
    encode_declarations [ minimal_axiom_decl; minimal_axiom_decl ]
  in
  assert_decode_error "duplicate declaration name" "noncanonical_encoding"
    Ext_bytes.Duplicate_declaration Ext_bytes.Declarations
    (String.length (encode_uvar_int 2 ^ minimal_axiom_decl))
    (Ext_cert.read_declarations 0 [ make_name [ "A" ] ] simple_level_table simple_term_table
       (Ext_bytes.of_string duplicate_declarations));

  let dangling_term_export =
    encode_uvar_int 1 ^ encode_uvar_int 0 ^ encode_export_kind 0x00 ^ encode_usize_vec []
    ^ encode_uvar_int 1
  in
  assert_decode_error "export dangling term" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Export_block 4
    (Ext_cert.read_export_block 0
       (Array.of_list [ make_name [ "A" ] ])
       (Array.of_list simple_term_table) 1
       (Ext_bytes.of_string dangling_term_export));

  let export_prefix = encode_export_entry_prefix 0 0x00 [] 0 None in
  let axiom_ref_len = encode_uvar_int 1 in
  let dangling_decl_offset = String.length (encode_uvar_int 1 ^ export_prefix ^ axiom_ref_len) in
  let dangling_decl_export =
    encode_uvar_int 1 ^ export_prefix ^ axiom_ref_len ^ encode_global_local 99
    ^ encode_uvar_int 0 ^ hash_bytes 0x51
  in
  assert_decode_error "export dangling declaration" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Export_block dangling_decl_offset
    (Ext_cert.read_export_block 0
       (Array.of_list [ make_name [ "A" ] ])
       (Array.of_list simple_term_table) 1
       (Ext_bytes.of_string dangling_decl_export))

let run_decoder_reachability_tests () =
  let golden_path =
    Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Nat/Basic/certificate.npcert"
  in
  let golden = read_binary_file golden_path in
  (match Ext_cert.read_module (Ext_bytes.of_string golden) with
  | Error error ->
      failwith
        ("reachability golden module: unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason ^ " at "
       ^ Ext_bytes.section_name error.Ext_bytes.section ^ ":"
       ^ string_of_int error.Ext_bytes.offset)
  | Ok (_, next) ->
      assert_int_equal "reachability golden offset" (String.length golden)
        (Ext_bytes.offset next));

  let minimal = encode_minimal_module [ minimal_axiom_decl ] [ minimal_export_entry ] in
  (match Ext_cert.read_module (Ext_bytes.of_string minimal) with
  | Error error ->
      failwith
        ("reachability minimal module: unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (_, next) ->
      assert_int_equal "reachability minimal offset" (String.length minimal)
        (Ext_bytes.offset next));

  let axiom_report_root =
    encode_module ~axiom_report:(encode_axiom_report [] [ (encode_global_local 0, 1, hash_bytes 0x44) ])
      [ [ "A" ]; [ "B" ] ] [ encode_level_zero ] [ encode_term_sort 0 ]
      [ minimal_axiom_decl ] [ minimal_export_entry ]
  in
  (match Ext_cert.read_module (Ext_bytes.of_string axiom_report_root) with
  | Error error ->
      failwith
        ("axiom report root module: unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason)
  | Ok (_, next) ->
      assert_int_equal "axiom report root offset" (String.length axiom_report_root)
        (Ext_bytes.offset next));

  let unused_name_prefix =
    encode_header [ "M" ] ^ encode_imports [] ^ encode_uvar_int 2 ^ encode_name [ "A" ]
  in
  let unused_name =
    encode_module [ [ "A" ]; [ "Z" ] ] [ encode_level_zero ] [ encode_term_sort 0 ]
      [ minimal_axiom_decl ] [ minimal_export_entry ]
  in
  assert_decode_error "unused name table entry" "noncanonical_encoding"
    Ext_bytes.Unused_table_entry Ext_bytes.Name_table (String.length unused_name_prefix)
    (Ext_cert.read_module (Ext_bytes.of_string unused_name));

  let reordered_name_prefix =
    encode_header [ "M" ] ^ encode_imports [] ^ encode_uvar_int 2 ^ encode_name [ "Z" ]
  in
  let reordered_name_decl =
    encode_decl_cert (encode_axiom_decl_payload 1 [] 0) [] [] (hash_bytes 0x19) (hash_bytes 0x1a)
  in
  let reordered_name_export = encode_export_entry 1 0x00 [] 0 None [] in
  let reordered_name =
    encode_module [ [ "Z" ]; [ "A" ] ] [ encode_level_zero ] [ encode_term_sort 0 ]
      [ reordered_name_decl ] [ reordered_name_export ]
  in
  assert_decode_error "reordered name table" "noncanonical_encoding"
    Ext_bytes.Noncanonical_order Ext_bytes.Name_table (String.length reordered_name_prefix)
    (Ext_cert.read_module (Ext_bytes.of_string reordered_name));

  let unused_level_prefix =
    encode_header [ "M" ] ^ encode_imports [] ^ encode_name_table [ [ "A" ] ]
    ^ encode_uvar_int 2 ^ encode_level_zero
  in
  let unused_level =
    encode_module [ [ "A" ] ] [ encode_level_zero; encode_level_param 0 ] [ encode_term_sort 0 ]
      [ minimal_axiom_decl ] [ minimal_export_entry ]
  in
  assert_decode_error "unused level table entry" "noncanonical_encoding"
    Ext_bytes.Unused_table_entry Ext_bytes.Level_table (String.length unused_level_prefix)
    (Ext_cert.read_module (Ext_bytes.of_string unused_level));

  let unused_term_prefix =
    encode_header [ "M" ] ^ encode_imports [] ^ encode_name_table [ [ "A" ] ]
    ^ encode_level_table [ encode_level_zero ] ^ encode_uvar_int 2 ^ encode_term_sort 0
  in
  let unused_term =
    encode_module [ [ "A" ] ] [ encode_level_zero ] [ encode_term_sort 0; encode_term_bvar 0 ]
      [ minimal_axiom_decl ] [ minimal_export_entry ]
  in
  assert_decode_error "unused term table entry" "noncanonical_encoding"
    Ext_bytes.Unused_table_entry Ext_bytes.Term_table (String.length unused_term_prefix)
    (Ext_cert.read_module (Ext_bytes.of_string unused_term));

  let reordered_level_prefix =
    encode_header [ "M" ] ^ encode_imports [] ^ encode_name_table [ [ "A" ] ]
    ^ encode_uvar_int 2 ^ encode_level_param 0
  in
  let reordered_level_decl =
    encode_decl_cert (encode_axiom_decl_payload 0 [] 0) [] [] (hash_bytes 0x21) (hash_bytes 0x22)
  in
  let reordered_level_export = encode_export_entry 0 0x00 [] 0 None [] in
  let reordered_level =
    encode_module [ [ "A" ] ] [ encode_level_param 0; encode_level_zero ] [ encode_term_sort 1 ]
      [ reordered_level_decl ] [ reordered_level_export ]
  in
  assert_decode_error "reordered level table" "noncanonical_encoding"
    Ext_bytes.Noncanonical_order Ext_bytes.Level_table (String.length reordered_level_prefix)
    (Ext_cert.read_module (Ext_bytes.of_string reordered_level));

  let reordered_term_prefix =
    encode_header [ "M" ] ^ encode_imports [] ^ encode_name_table [ [ "A" ] ]
    ^ encode_level_table [ encode_level_zero ] ^ encode_uvar_int 2 ^ encode_term_bvar 0
  in
  let reordered_term_decl =
    encode_decl_cert (encode_axiom_decl_payload 0 [] 1) [] [] (hash_bytes 0x23) (hash_bytes 0x24)
  in
  let reordered_term_export = encode_export_entry 0 0x00 [] 1 None [] in
  let reordered_term =
    encode_module [ [ "A" ] ] [ encode_level_zero ] [ encode_term_bvar 0; encode_term_sort 0 ]
      [ reordered_term_decl ] [ reordered_term_export ]
  in
  assert_decode_error "reordered term table" "noncanonical_encoding"
    Ext_bytes.Noncanonical_order Ext_bytes.Term_table (String.length reordered_term_prefix)
    (Ext_cert.read_module (Ext_bytes.of_string reordered_term));

  assert_decode_error "trailing bytes after hashes" "certificate_decode_error"
    Ext_bytes.Trailing_bytes Ext_bytes.Full_certificate (String.length minimal)
    (Ext_cert.read_module (Ext_bytes.of_string (minimal ^ "x")))

let encode_export_entry_full name_id kind_tag universe_params ty body type_hash body_hash
    reducibility opacity decl_interface_hash axiom_dependencies =
  encode_uvar_int name_id ^ encode_export_kind kind_tag ^ encode_usize_vec universe_params
  ^ encode_uvar_int ty ^ encode_option_usize body ^ type_hash ^ encode_option_hash body_hash
  ^ encode_option reducibility ^ encode_option opacity ^ decl_interface_hash
  ^ encode_axiom_refs axiom_dependencies

let first_declaration decoded =
  match decoded.Ext_cert.declaration_table with
  | declaration :: _ -> declaration
  | [] -> failwith "expected declaration fixture"

let assert_canonical_hash label expected_hex result =
  let hash = assert_ok label result in
  assert_equal label expected_hex (hex_of_raw_hash hash)

let assert_canonical_bytes label expected result =
  assert_equal label expected (assert_ok label result)

let assert_hash_hexes label expected result =
  let hashes = assert_ok label result in
  assert_int_equal (label ^ " length") (List.length expected) (List.length hashes);
  List.iteri
    (fun index expected_hex ->
      assert_equal
        (label ^ " " ^ string_of_int index)
        expected_hex
        (hex_of_raw_hash (List.nth hashes index)))
    expected;
  hashes

let located_names names =
  List.mapi (fun offset name -> { Ext_cert.name; offset }) names

let decode_level_table label names bytes =
  match Ext_level.read_table names (Ext_bytes.of_string bytes) with
  | Ok (levels, next) ->
      assert_int_equal (label ^ " offset") (String.length bytes) (Ext_bytes.offset next);
      levels
  | Error error ->
      failwith
        (label ^ ": unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason ^ " at "
       ^ Ext_bytes.section_name error.Ext_bytes.section ^ ":"
       ^ string_of_int error.Ext_bytes.offset)

let decode_term_table label names levels bytes =
  match Ext_term.read_table names levels (Ext_bytes.of_string bytes) with
  | Ok (terms, next) ->
      assert_int_equal (label ^ " offset") (String.length bytes) (Ext_bytes.offset next);
      terms
  | Error error ->
      failwith
        (label ^ ": unexpected decode error "
       ^ Ext_bytes.reason_code error.Ext_bytes.reason ^ " at "
       ^ Ext_bytes.section_name error.Ext_bytes.section ^ ":"
       ^ string_of_int error.Ext_bytes.offset)

let assert_export_term_hashes label decoded =
  let level_hashes =
    assert_ok (label ^ " level hashes") (Ext_canonical.level_hashes decoded.Ext_cert.level_table)
  in
  let term_hashes =
    assert_ok (label ^ " term hashes")
      (Ext_canonical.term_hashes decoded.Ext_cert.name_table decoded.Ext_cert.level_table
         level_hashes decoded.Ext_cert.term_table)
  in
  List.iteri
    (fun index export ->
      let prefix = label ^ " export " ^ string_of_int index in
      let type_hash =
        assert_ok (prefix ^ " type hash")
          (Ext_canonical.hash_for_term Ext_bytes.Export_block export.Ext_cert.export_offset
             decoded.Ext_cert.name_table decoded.Ext_cert.term_table term_hashes
             export.Ext_cert.export_ty)
      in
      assert_equal (prefix ^ " type hash")
        (hex_of_raw_hash export.Ext_cert.export_type_hash)
        (hex_of_raw_hash type_hash);
      match (export.Ext_cert.export_body, export.Ext_cert.export_body_hash) with
      | None, None -> ()
      | Some body, Some expected_body_hash ->
          let body_hash =
            assert_ok (prefix ^ " body hash")
              (Ext_canonical.hash_for_term Ext_bytes.Export_block export.Ext_cert.export_offset
                 decoded.Ext_cert.name_table decoded.Ext_cert.term_table term_hashes body)
          in
          assert_equal (prefix ^ " body hash") (hex_of_raw_hash expected_body_hash)
            (hex_of_raw_hash body_hash)
      | _ -> failwith (prefix ^ ": body and body_hash option mismatch"))
    decoded.Ext_cert.export_block

let assert_declaration_hashes label decoded =
  List.iteri
    (fun index declaration ->
      let prefix = label ^ " decl " ^ string_of_int index in
      let interface_payload =
        assert_ok (prefix ^ " interface payload")
          (Ext_canonical.declaration_interface_payload decoded.Ext_cert.name_table
             decoded.Ext_cert.level_table decoded.Ext_cert.term_table
             declaration.Ext_cert.payload declaration.Ext_cert.dependencies
             declaration.Ext_cert.axiom_dependencies)
      in
      let interface_hash =
        Ext_canonical.hash_with_domain Ext_canonical.domain_decl_interface interface_payload
      in
      assert_equal (prefix ^ " interface hash")
        (hex_of_raw_hash declaration.Ext_cert.hashes.Ext_cert.decl_interface_hash)
        (hex_of_raw_hash interface_hash);
      let certificate_payload =
        assert_ok (prefix ^ " certificate payload")
          (Ext_canonical.declaration_certificate_payload decoded.Ext_cert.name_table
             decoded.Ext_cert.level_table decoded.Ext_cert.term_table declaration.Ext_cert.payload
             interface_hash declaration.Ext_cert.dependencies declaration.Ext_cert.axiom_dependencies)
      in
      let certificate_hash =
        Ext_canonical.hash_with_domain Ext_canonical.domain_decl_certificate certificate_payload
      in
      assert_equal (prefix ^ " certificate hash")
        (hex_of_raw_hash declaration.Ext_cert.hashes.Ext_cert.decl_certificate_hash)
        (hex_of_raw_hash certificate_hash))
    decoded.Ext_cert.declaration_table

let recompute_stored_declaration_hashes label decoded =
  let declaration_table =
    List.mapi
      (fun index declaration ->
        let prefix = label ^ " decl " ^ string_of_int index in
        let interface_hash, certificate_hash =
          assert_ok (prefix ^ " recomputed hashes")
            (Ext_canonical.declaration_hashes decoded.Ext_cert.name_table
               decoded.Ext_cert.level_table decoded.Ext_cert.term_table declaration)
        in
        let hashes =
          {
            declaration.Ext_cert.hashes with
            Ext_cert.decl_interface_hash = interface_hash;
            decl_certificate_hash = certificate_hash;
          }
        in
        { declaration with Ext_cert.hashes = hashes })
      decoded.Ext_cert.declaration_table
  in
  { decoded with Ext_cert.declaration_table }

let replace_first_declaration decoded update =
  match decoded.Ext_cert.declaration_table with
  | declaration :: rest ->
      { decoded with Ext_cert.declaration_table = update declaration :: rest }
  | [] -> failwith "expected declaration fixture"

let assert_declaration_hash_verifies label decoded =
  match
    assert_ok (label ^ " declaration hash check")
      (Ext_canonical.verify_declaration_hashes decoded)
  with
  | Ext_canonical.Declaration_hashes_ok -> ()
  | Ext_canonical.Declaration_hash_mismatch mismatch ->
      failwith
        (label ^ ": unexpected declaration hash mismatch at "
       ^ string_of_int mismatch.Ext_canonical.mismatch_offset)

let assert_declaration_hash_rejects label expected_kind expected_reason decoded =
  match
    assert_ok (label ^ " declaration hash check")
      (Ext_canonical.verify_declaration_hashes decoded)
  with
  | Ext_canonical.Declaration_hashes_ok -> failwith (label ^ ": expected hash mismatch")
  | Ext_canonical.Declaration_hash_mismatch mismatch ->
      let kind =
        Ext_canonical.declaration_hash_mismatch_kind_code
          mismatch.Ext_canonical.mismatch_kind
      in
      let reason =
        Ext_canonical.declaration_hash_role_reason_code
          mismatch.Ext_canonical.mismatch_role
      in
      let offset = mismatch.Ext_canonical.mismatch_offset in
      assert_equal (label ^ " kind") expected_kind kind;
      assert_equal (label ^ " reason") expected_reason reason;
      assert_bool (label ^ " expected differs from actual")
        (mismatch.Ext_canonical.expected_hash <> mismatch.Ext_canonical.actual_hash);
      let raw =
        Ext_result.hash_mismatch_failure ~kind ~reason_code:reason
          ~section:"declarations" ~offset
      in
      assert_contains (label ^ " raw kind") ("\"kind\": \"" ^ expected_kind ^ "\"") raw;
      assert_contains (label ^ " raw reason")
        ("\"reason_code\": \"" ^ expected_reason ^ "\"")
        raw;
      assert_contains (label ^ " raw section") "\"section\": \"declarations\"" raw;
      assert_contains (label ^ " raw offset") ("\"offset\": " ^ string_of_int offset) raw

let assert_module_hash_verifies label bytes decoded =
  match
    assert_ok (label ^ " module hash check")
      (Ext_canonical.verify_module_hashes bytes decoded)
  with
  | Ext_canonical.Module_hashes_ok -> ()
  | Ext_canonical.Module_hash_mismatch mismatch ->
      failwith
        (label ^ ": unexpected module hash mismatch "
       ^ Ext_canonical.module_hash_role_kind_code
           mismatch.Ext_canonical.module_mismatch_role
       ^ " at "
       ^ string_of_int mismatch.Ext_canonical.module_mismatch_offset)

let assert_module_hash_rejects label expected_kind expected_offset bytes decoded =
  match
    assert_ok (label ^ " module hash check")
      (Ext_canonical.verify_module_hashes bytes decoded)
  with
  | Ext_canonical.Module_hashes_ok -> failwith (label ^ ": expected module hash mismatch")
  | Ext_canonical.Module_hash_mismatch mismatch ->
      let kind =
        Ext_canonical.module_hash_role_kind_code
          mismatch.Ext_canonical.module_mismatch_role
      in
      let offset = mismatch.Ext_canonical.module_mismatch_offset in
      assert_equal (label ^ " kind") expected_kind kind;
      assert_int_equal (label ^ " offset") expected_offset offset;
      assert_bool (label ^ " expected differs from actual")
        (mismatch.Ext_canonical.module_expected_hash
        <> mismatch.Ext_canonical.module_actual_hash);
      let raw =
        Ext_result.hash_mismatch_failure ~kind ~reason_code:kind ~section:"hashes"
          ~offset
      in
      assert_contains (label ^ " raw kind") ("\"kind\": \"" ^ expected_kind ^ "\"") raw;
      assert_contains (label ^ " raw reason")
        ("\"reason_code\": \"" ^ expected_kind ^ "\"")
        raw;
      assert_contains (label ^ " raw section") "\"section\": \"hashes\"" raw;
      assert_contains (label ^ " raw offset") ("\"offset\": " ^ string_of_int offset) raw

let theorem_payload_with_type payload decl_ty =
  match payload with
  | Ext_cert.TheoremDecl
      { decl_name; decl_universe_params; decl_universe_constraints; decl_proof; decl_opacity; _ }
    ->
      Ext_cert.TheoremDecl
        {
          decl_name;
          decl_universe_params;
          decl_universe_constraints;
          decl_ty;
          decl_proof;
          decl_opacity;
        }
  | _ -> failwith "expected theorem declaration"

let theorem_payload_with_proof payload decl_proof =
  match payload with
  | Ext_cert.TheoremDecl
      { decl_name; decl_universe_params; decl_universe_constraints; decl_ty; decl_opacity; _ }
    ->
      Ext_cert.TheoremDecl
        {
          decl_name;
          decl_universe_params;
          decl_universe_constraints;
          decl_ty;
          decl_proof;
          decl_opacity;
        }
  | _ -> failwith "expected theorem declaration"

let mutate_first_dependency_hash declaration hash =
  match declaration.Ext_cert.dependencies with
  | dependency :: rest ->
      let dependency =
        { dependency with Ext_cert.dependency_decl_interface_hash = hash }
      in
      { declaration with Ext_cert.dependencies = dependency :: rest }
  | [] -> failwith "expected dependency fixture"

let mutate_first_axiom_dependency_hash declaration hash =
  match declaration.Ext_cert.axiom_dependencies with
  | axiom :: rest ->
      let axiom = { axiom with Ext_cert.axiom_decl_interface_hash = hash } in
      { declaration with Ext_cert.axiom_dependencies = axiom :: rest }
  | [] -> failwith "expected axiom dependency fixture"

let run_hash_level_term_tests () =
  let names = [ make_name [ "u" ]; make_name [ "Imported" ] ] in
  let name_table = located_names names in
  let level_bytes =
    encode_uvar_int 4 ^ encode_level_param 0 ^ encode_level_succ 0
    ^ encode_level_max 1 0 ^ encode_level_imax 0 0
  in
  let level_table = decode_level_table "hash level table" names level_bytes in
  let level_hashes =
    assert_hash_hexes "level hash"
      [
        "14ca4d271ed543507887e0ea523cefe7767b12c4c88c64db7797af8e5d60edca";
        "3c4dc3d2830d5c7b16bf22a38bbdc0867936d8e0faa2cdfb909fbfb314e0b9ef";
        "5ca42f83e7ab0f56fa5d53b157a5816bba36dfe71ca83d228b790dd7f52f667e";
        "b7dff10a5ac7d0c3c25ec2f2007b12015444606e970292c103dd2239df70cc48";
      ]
      (Ext_canonical.level_hashes level_table)
  in

  let imported_ref = encode_global_imported 0 1 (hash_bytes 0x55) in
  let term_bytes =
    encode_uvar_int 8 ^ encode_term_sort 0 ^ encode_term_sort 1 ^ encode_term_bvar 0
    ^ encode_term_const imported_ref [ 0; 1 ]
    ^ encode_term_app 3 2 ^ encode_term_lam 0 4 ^ encode_term_pi 0 5
    ^ encode_term_let 0 2 6
  in
  let term_table = decode_term_table "hash term table" names level_table term_bytes in
  let term_hashes =
    assert_hash_hexes "term hash"
      [
        "4dbd7b9567ca2c9a3014d70c03e2213e85686af92f3aa86ee57a1003de1c48d5";
        "d4c881c652406552c33e9f7e374c0eed412f711733a4657b978d052262f19406";
        "7f20eac79de1e58183de939cbf75e45bc92e8c8a1ac0b7c8e4fca287d201fcb7";
        "f6aac19b5b3fbe1c698ebc7b02acd3f32d7d287fe06ad7108191d5d6cfe09c42";
        "aa45ed6b3051ec6dd79b578d048c64711404e1434d39082d8874ad1777db8ea9";
        "8079e8d16fa1f32538052afd5379b3107399c2964d6e43aad7082ad938b8c670";
        "37adbeb21882f9c57f6c6f952715b9e75e8a30e53ab88269d20ec40976b3300e";
        "9dde1d65cb02d6d632083bd28394894abb0c42b55285190f4e1d4b648433ac46";
      ]
      (Ext_canonical.term_hashes name_table level_table level_hashes term_table)
  in

  let mutated_level_table =
    decode_level_table "mutated level table" names
      (encode_uvar_int 2 ^ encode_level_zero ^ encode_level_succ 0)
  in
  let mutated_level_hashes =
    assert_ok "mutated level hashes"
      (Ext_canonical.level_hashes mutated_level_table)
  in
  assert_bool "mutating referenced level changes dependent level hash"
    (List.nth level_hashes 1 <> List.nth mutated_level_hashes 1);
  let mutated_term_table =
    decode_term_table "mutated term table" names level_table
      (encode_uvar_int 5 ^ encode_term_sort 0 ^ encode_term_sort 1
      ^ encode_term_bvar 1 ^ encode_term_const imported_ref [ 0; 1 ]
      ^ encode_term_app 3 2)
  in
  let mutated_term_hashes =
    assert_ok "mutated term hashes"
      (Ext_canonical.term_hashes name_table level_table level_hashes mutated_term_table)
  in
  assert_bool "mutating referenced term changes dependent term hash"
    (List.nth term_hashes 4 <> List.nth mutated_term_hashes 4);

  let dangling_level_table = [ { Ext_level.level = Ext_level.Succ Ext_level.Zero; offset = 7 } ] in
  assert_decode_error "level hash dangling child" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Level_table 7
    (Ext_canonical.level_hashes dangling_level_table);
  let dangling_term_table =
    [ { Ext_term.term = Ext_term.App (Ext_term.BVar 0, Ext_term.BVar 0); offset = 9 } ]
  in
  assert_decode_error "term hash dangling child" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Term_table 9
    (Ext_canonical.term_hashes [] [] [] dangling_term_table);
  let missing_level_term_table =
    [ { Ext_term.term = Ext_term.Sort Ext_level.Zero; offset = 11 } ]
  in
  assert_decode_error "term hash dangling level" "certificate_decode_error"
    Ext_bytes.Dangling_reference Ext_bytes.Term_table 11
    (Ext_canonical.term_hashes [] [] [] missing_level_term_table);

  let assert_golden_export_terms label path =
    let decoded =
      decode_module_bytes (label ^ " hash level-term golden") (read_binary_file path)
    in
    assert_export_term_hashes label decoded
  in
  assert_golden_export_terms "nat"
    (Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Nat/Basic/certificate.npcert");
  assert_golden_export_terms "eq"
    (Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Logic/Eq/certificate.npcert")

let run_hash_declarations_tests () =
  let assert_golden_declarations label path =
    let decoded =
      decode_module_bytes (label ^ " declaration hash golden") (read_binary_file path)
    in
    assert_declaration_hash_verifies label decoded
  in
  assert_golden_declarations "nat"
    (Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Nat/Basic/certificate.npcert");
  assert_golden_declarations "eq"
    (Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Logic/Eq/certificate.npcert");

  let simple_theorem_decl =
    encode_decl_cert (encode_theorem_decl_payload 0x02 0 [] 0 1) [] []
      (hash_bytes 0x41) (hash_bytes 0x42)
  in
  let simple_theorem_export =
    encode_export_entry_full 0 0x02 [] 0 None (hash_bytes 0x31) None None
      (Some encode_opacity_opaque) (hash_bytes 0x32) []
  in
  let simple_theorem_module =
    encode_module [ [ "A" ] ] [ encode_level_zero ]
      [ encode_term_sort 0; encode_term_bvar 0 ]
      [ simple_theorem_decl ] [ simple_theorem_export ]
  in
  let simple_theorem =
    recompute_stored_declaration_hashes "simple theorem declaration hashes"
      (decode_module_bytes "simple theorem declaration hashes" simple_theorem_module)
  in
  assert_declaration_hash_verifies "simple theorem valid declaration hashes"
    simple_theorem;
  let mutated_type =
    replace_first_declaration simple_theorem (fun declaration ->
        {
          declaration with
          Ext_cert.payload =
            theorem_payload_with_type declaration.Ext_cert.payload (Ext_term.BVar 0);
        })
  in
  assert_declaration_hash_rejects "mutated declaration type"
    "declaration_hash_mismatch" "decl_interface_hash_mismatch" mutated_type;
  let mutated_body =
    replace_first_declaration simple_theorem (fun declaration ->
        {
          declaration with
          Ext_cert.payload =
            theorem_payload_with_proof declaration.Ext_cert.payload
              Ext_term.(Sort Ext_level.Zero);
        })
  in
  assert_declaration_hash_rejects "mutated declaration body"
    "declaration_hash_mismatch" "decl_certificate_hash_mismatch" mutated_body;

  let imported_ref = encode_global_imported 0 1 (hash_bytes 0x55) in
  let dependency_theorem_decl =
    encode_decl_cert
      (encode_theorem_decl_payload 0x02 0 [] 0 1)
      [ (imported_ref, hash_bytes 0x55) ] [] (hash_bytes 0x51) (hash_bytes 0x52)
  in
  let dependency_theorem_export =
    encode_export_entry_full 0 0x02 [] 0 None (hash_bytes 0x31) None None
      (Some encode_opacity_opaque) (hash_bytes 0x32) []
  in
  let dependency_module =
    encode_module ~imports:[ ([ "Dep" ], hash_bytes 0x71, None) ]
      [ [ "A" ]; [ "Imported" ] ] [ encode_level_zero ]
      [ encode_term_sort 0; encode_term_const imported_ref [] ]
      [ dependency_theorem_decl ] [ dependency_theorem_export ]
  in
  let dependency_theorem =
    recompute_stored_declaration_hashes "dependency declaration hashes"
      (decode_module_bytes "dependency declaration hashes" dependency_module)
  in
  assert_declaration_hash_verifies "dependency valid declaration hashes"
    dependency_theorem;
  let mutated_dependency =
    replace_first_declaration dependency_theorem (fun declaration ->
        mutate_first_dependency_hash declaration (hash_bytes 0x56))
  in
  assert_declaration_hash_rejects "mutated declaration dependency"
    "dependency_hash_mismatch" "decl_certificate_hash_mismatch" mutated_dependency;

  let axiom_dependency_ref = encode_global_imported 0 1 (hash_bytes 0x44) in
  let axiom_dependency_decl =
    encode_decl_cert (encode_axiom_decl_payload 0 [] 0) []
      [ (axiom_dependency_ref, 1, hash_bytes 0x44) ] (hash_bytes 0x61)
      (hash_bytes 0x62)
  in
  let axiom_dependency_export =
    encode_export_entry_full 0 0x00 [] 0 None (hash_bytes 0x31) None None None
      (hash_bytes 0x32) []
  in
  let axiom_dependency_module =
    encode_module ~imports:[ ([ "Dep" ], hash_bytes 0x71, None) ]
      [ [ "A" ]; [ "Imported" ] ] [ encode_level_zero ] [ encode_term_sort 0 ]
      [ axiom_dependency_decl ] [ axiom_dependency_export ]
  in
  let axiom_dependency =
    recompute_stored_declaration_hashes "axiom dependency declaration hashes"
      (decode_module_bytes "axiom dependency declaration hashes" axiom_dependency_module)
  in
  assert_declaration_hash_verifies "axiom dependency valid declaration hashes"
    axiom_dependency;
  let mutated_axiom_dependency =
    replace_first_declaration axiom_dependency (fun declaration ->
        mutate_first_axiom_dependency_hash declaration (hash_bytes 0x45))
  in
  assert_declaration_hash_rejects "mutated declaration axiom dependency"
    "dependency_hash_mismatch" "decl_certificate_hash_mismatch"
    mutated_axiom_dependency

let run_hash_module_tests () =
  let golden_paths =
    [
      ( "nat",
        Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Nat/Basic/certificate.npcert"
      );
      ( "eq",
        Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Logic/Eq/certificate.npcert"
      );
    ]
  in
  let decoded_golden label path =
    let bytes = read_binary_file path in
    (bytes, decode_module_bytes (label ^ " module hash golden") bytes)
  in
  List.iter
    (fun (label, path) ->
      let bytes, decoded = decoded_golden label path in
      assert_module_hash_verifies (label ^ " valid module hashes") bytes decoded)
    golden_paths;

  let bytes, decoded = decoded_golden "nat mutation corpus" (List.assoc "nat" golden_paths) in
  let hashes = decoded.Ext_cert.hashes in
  let assert_mutated_hash label expected_kind offset =
    let mutated = mutate_byte bytes offset in
    let decoded_mutated =
      decode_module_bytes (label ^ " mutated module hash") mutated
    in
    assert_module_hash_rejects label expected_kind offset mutated decoded_mutated
  in
  assert_mutated_hash "mutated export hash" "export_hash_mismatch"
    hashes.Ext_cert.export_hash_offset;
  assert_mutated_hash "mutated axiom report hash" "axiom_report_mismatch"
    hashes.Ext_cert.axiom_report_hash_offset;
  assert_mutated_hash "mutated certificate hash" "certificate_hash_mismatch"
    hashes.Ext_cert.certificate_hash_offset;

  let prefix_mutated = mutate_byte bytes 0 in
  assert_module_hash_rejects "module certificate hash uses exact input prefix"
    "certificate_hash_mismatch" hashes.Ext_cert.certificate_hash_offset prefix_mutated
    decoded;

  let mutated_export_block =
    match decoded.Ext_cert.export_block with
    | export :: rest ->
        {
          export with
          Ext_cert.export_type_hash = mutate_byte export.Ext_cert.export_type_hash 0;
        }
        :: rest
    | [] -> failwith "expected golden export block"
  in
  let decoded_with_stored_export_block = { decoded with Ext_cert.export_block = mutated_export_block } in
  let forged_export_hash =
    assert_ok "stored export block hash"
      (Ext_canonical.export_hash decoded_with_stored_export_block)
  in
  let forged_hashes =
    { decoded.Ext_cert.hashes with Ext_cert.export_hash = forged_export_hash }
  in
  let decoded_with_forged_export_hash =
    { decoded_with_stored_export_block with Ext_cert.hashes = forged_hashes }
  in
  assert_module_hash_rejects
    "module hash verifier rebuilds expected export block from declarations"
    "export_hash_mismatch" hashes.Ext_cert.export_hash_offset bytes
    decoded_with_forged_export_hash

let run_hash_encoder_tests () =
  let empty_module = encode_module [] [] [] [] [] in
  let empty_decoded = decode_module_bytes "empty hash fixture" empty_module in
  assert_canonical_bytes "empty export payload" (encode_export_block [])
    (Ext_canonical.encode_export_block empty_decoded);
  assert_canonical_bytes "empty axiom report payload" (encode_axiom_report [] [])
    (Ext_canonical.encode_axiom_report empty_decoded.Ext_cert.name_table
       empty_decoded.Ext_cert.axiom_report);
  let empty_export_payload = assert_ok "empty export payload for domain"
      (Ext_canonical.encode_export_block empty_decoded)
  in
  assert_bool "domain label affects export hash"
    (Ext_canonical.hash_with_domain Ext_canonical.domain_module_export empty_export_payload
    <> Ext_canonical.hash_with_domain "NPA-MODULE-EXPORT-X" empty_export_payload);

  let axiom_module = encode_minimal_module [ minimal_axiom_decl ] [ minimal_export_entry ] in
  let axiom_decoded = decode_module_bytes "axiom hash fixture" axiom_module in
  assert_canonical_bytes "axiom export payload" (encode_export_block [ minimal_export_entry ])
    (Ext_canonical.encode_export_block axiom_decoded);
  let axiom_decl = first_declaration axiom_decoded in
  let sort_hash =
    assert_ok "sort term hash"
      (Ext_canonical.term_hash Ext_bytes.Term_table axiom_decl.Ext_cert.offset
         axiom_decoded.Ext_cert.name_table Ext_term.(Sort Ext_level.Zero))
  in
  let expected_axiom_iface =
    one_byte 0x00 ^ encode_name [ "A" ] ^ encode_uvar_int 0 ^ sort_hash
    ^ encode_dependency_entries []
  in
  assert_canonical_bytes "axiom declaration interface payload" expected_axiom_iface
    (Ext_canonical.declaration_interface_payload axiom_decoded.Ext_cert.name_table
       axiom_decoded.Ext_cert.level_table axiom_decoded.Ext_cert.term_table axiom_decl.Ext_cert.payload
       axiom_decl.Ext_cert.dependencies axiom_decl.Ext_cert.axiom_dependencies);
  let axiom_iface_hash =
    Ext_canonical.hash_with_domain Ext_canonical.domain_decl_interface expected_axiom_iface
  in
  assert_canonical_bytes "axiom declaration certificate payload"
    (axiom_iface_hash ^ encode_axiom_refs [])
    (Ext_canonical.declaration_certificate_payload axiom_decoded.Ext_cert.name_table
       axiom_decoded.Ext_cert.level_table axiom_decoded.Ext_cert.term_table
       axiom_decl.Ext_cert.payload axiom_iface_hash axiom_decl.Ext_cert.dependencies
       axiom_decl.Ext_cert.axiom_dependencies);

  let imported_ref = encode_global_imported 0 1 (hash_bytes 0x55) in
  let theorem_decl_bytes =
    encode_decl_cert
      (encode_theorem_decl_payload 0x02 0 [] 0 1)
      [ (imported_ref, hash_bytes 0x55) ] [] (hash_bytes 0x41) (hash_bytes 0x42)
  in
  let theorem_export =
    encode_export_entry_full 0 0x02 [] 0 None (hash_bytes 0x31) None None
      (Some encode_opacity_opaque) (hash_bytes 0x32) []
  in
  let theorem_module =
    encode_module ~imports:[ ([ "Dep" ], hash_bytes 0x71, None) ]
      [ [ "A" ]; [ "Imported" ] ] [ encode_level_zero ]
      [ encode_term_sort 0; encode_term_const imported_ref [] ]
      [ theorem_decl_bytes ] [ theorem_export ]
  in
  let theorem_decoded = decode_module_bytes "theorem hash fixture" theorem_module in
  assert_canonical_bytes "theorem export payload" (encode_export_block [ theorem_export ])
    (Ext_canonical.encode_export_block theorem_decoded);
  let theorem_decl = first_declaration theorem_decoded in
  let theorem_sort_hash =
    assert_ok "theorem sort term hash"
      (Ext_canonical.term_hash Ext_bytes.Term_table theorem_decl.Ext_cert.offset
         theorem_decoded.Ext_cert.name_table Ext_term.(Sort Ext_level.Zero))
  in
  let expected_theorem_iface =
    one_byte 0x02 ^ encode_name [ "A" ] ^ encode_uvar_int 0 ^ theorem_sort_hash
    ^ encode_opacity_opaque ^ encode_dependency_entries [] ^ encode_axiom_refs []
  in
  assert_canonical_bytes "theorem declaration interface payload" expected_theorem_iface
    (Ext_canonical.declaration_interface_payload theorem_decoded.Ext_cert.name_table
       theorem_decoded.Ext_cert.level_table theorem_decoded.Ext_cert.term_table
       theorem_decl.Ext_cert.payload theorem_decl.Ext_cert.dependencies
       theorem_decl.Ext_cert.axiom_dependencies);
  let theorem_proof =
    match theorem_decl.Ext_cert.payload with
    | Ext_cert.TheoremDecl { decl_proof; _ } -> decl_proof
    | _ -> failwith "expected theorem declaration"
  in
  let theorem_proof_hash =
    assert_ok "theorem proof term hash"
      (Ext_canonical.term_hash Ext_bytes.Term_table theorem_decl.Ext_cert.offset
         theorem_decoded.Ext_cert.name_table theorem_proof)
  in
  let theorem_iface_hash =
    Ext_canonical.hash_with_domain Ext_canonical.domain_decl_interface expected_theorem_iface
  in
  assert_canonical_bytes "theorem declaration certificate payload"
    (theorem_iface_hash ^ theorem_proof_hash
    ^ encode_dependency_entries [ (imported_ref, hash_bytes 0x55) ])
    (Ext_canonical.declaration_certificate_payload theorem_decoded.Ext_cert.name_table
       theorem_decoded.Ext_cert.level_table theorem_decoded.Ext_cert.term_table
       theorem_decl.Ext_cert.payload theorem_iface_hash theorem_decl.Ext_cert.dependencies
       theorem_decl.Ext_cert.axiom_dependencies);

  let import_decl =
    encode_decl_cert
      (encode_def_decl_payload 0x01 0 [] 0 1 `Reducible)
      [ (imported_ref, hash_bytes 0x55) ] [] (hash_bytes 0x56) (hash_bytes 0x57)
  in
  let import_export =
    encode_export_entry_full 0 0x01 [] 0 (Some 1) (hash_bytes 0x31)
      (Some (hash_bytes 0x61)) (Some (encode_reducibility `Reducible)) None
      (hash_bytes 0x32) []
  in
  let import_module =
    encode_module ~imports:[ ([ "Dep" ], hash_bytes 0x71, None) ] [ [ "A" ]; [ "Imported" ] ]
      [ encode_level_zero ]
      [ encode_term_sort 0; encode_term_const imported_ref [] ]
      [ import_decl ] [ import_export ]
  in
  let import_decoded = decode_module_bytes "import hash fixture" import_module in
  assert_canonical_bytes "import dependency payload"
    (encode_dependency_entries [ (imported_ref, hash_bytes 0x55) ])
    (Ext_canonical.encode_dependency_entries Ext_bytes.Declarations 0
       import_decoded.Ext_cert.name_table (first_declaration import_decoded).Ext_cert.dependencies);
  assert_canonical_bytes "import export payload" (encode_export_block [ import_export ])
    (Ext_canonical.encode_export_block import_decoded);

  let inductive_decl =
    encode_decl_cert (encode_inductive_decl_payload 0x03 0 [] [] [] 0 [ (1, 0) ] None) [] []
      (hash_bytes 0x81) (hash_bytes 0x82)
  in
  let inductive_export = encode_export_entry 0 0x03 [] 0 None [] in
  let inductive_module =
    encode_module [ [ "A" ]; [ "C" ] ] [ encode_level_zero ] [ encode_term_sort 0 ]
      [ inductive_decl ] [ inductive_export ]
  in
  let inductive_decoded = decode_module_bytes "inductive hash fixture" inductive_module in
  ignore
    (assert_ok "inductive declaration interface payload"
       (Ext_canonical.declaration_interface_payload inductive_decoded.Ext_cert.name_table
          inductive_decoded.Ext_cert.level_table inductive_decoded.Ext_cert.term_table
          (first_declaration inductive_decoded).Ext_cert.payload
          (first_declaration inductive_decoded).Ext_cert.dependencies
          (first_declaration inductive_decoded).Ext_cert.axiom_dependencies));
  assert_canonical_bytes "inductive export payload" (encode_export_block [ inductive_export ])
    (Ext_canonical.encode_export_block inductive_decoded);

  let assert_golden_module label path =
    let bytes = read_binary_file path in
    let fixture = golden_hash_fixture label in
    assert_int_equal (label ^ " golden byte length") fixture.golden_byte_len
      (String.length bytes);
    let decoded = decode_module_bytes (label ^ " golden") bytes in
    assert_equal (label ^ " stored export hash") fixture.golden_export_hash
      (hex_of_raw_hash decoded.Ext_cert.hashes.Ext_cert.export_hash);
    assert_equal (label ^ " stored axiom report hash") fixture.golden_axiom_report_hash
      (hex_of_raw_hash decoded.Ext_cert.hashes.Ext_cert.axiom_report_hash);
    assert_equal (label ^ " stored certificate hash") fixture.golden_certificate_hash
      (hex_of_raw_hash decoded.Ext_cert.hashes.Ext_cert.certificate_hash);
    assert_declaration_hashes label decoded;
    assert_canonical_hash (label ^ " encoded export hash") fixture.golden_export_hash
      (Ext_canonical.export_hash decoded);
    assert_canonical_hash (label ^ " encoded axiom report hash")
      fixture.golden_axiom_report_hash (Ext_canonical.axiom_report_hash decoded)
  in
  assert_golden_module "nat"
    (Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Nat/Basic/certificate.npcert");
  assert_golden_module "eq"
    (Filename.concat (root_dir ()) "../../proofs/vendor/npa-std/Std/Logic/Eq/certificate.npcert")

let should_run selected name = selected = [] || List.mem name selected

let () =
  let selected = Array.to_list Sys.argv |> List.tl in
  List.iter
    (fun name ->
      if
        not
          (List.mem name
             [
               "cli";
               "decoder-bytes";
               "decoder-declarations";
               "decoder-header";
               "decoder-reachability";
               "decoder-tables";
               "feature-policy";
               "hash-declarations";
               "hash-encoder";
               "hash-level-term";
               "hash-module";
               "sha256";
             ])
      then
        failwith ("unknown test filter " ^ name))
    selected;
  if should_run selected "sha256" then run_sha256_tests ();
  if should_run selected "decoder-bytes" then run_decoder_bytes_tests ();
  if should_run selected "decoder-header" then run_decoder_header_tests ();
  if should_run selected "decoder-tables" then run_decoder_tables_tests ();
  if should_run selected "decoder-declarations" then run_decoder_declarations_tests ();
  if should_run selected "decoder-reachability" then run_decoder_reachability_tests ();
  if should_run selected "feature-policy" then run_feature_policy_tests ();
  if should_run selected "hash-level-term" then run_hash_level_term_tests ();
  if should_run selected "hash-declarations" then run_hash_declarations_tests ();
  if should_run selected "hash-module" then run_hash_module_tests ();
  if should_run selected "hash-encoder" then run_hash_encoder_tests ();
  if should_run selected "cli" then run_cli_tests ()
