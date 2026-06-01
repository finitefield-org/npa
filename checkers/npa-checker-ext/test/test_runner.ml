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
  if should_run selected "cli" then run_cli_tests ()
