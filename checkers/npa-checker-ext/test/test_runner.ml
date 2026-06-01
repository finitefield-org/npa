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

let should_run selected name = selected = [] || List.mem name selected

let () =
  let selected = Array.to_list Sys.argv |> List.tl in
  List.iter
    (fun name ->
      if
        not
          (List.mem name
             [ "cli"; "decoder-bytes"; "decoder-header"; "feature-policy"; "sha256" ])
      then
        failwith ("unknown test filter " ^ name))
    selected;
  if should_run selected "sha256" then run_sha256_tests ();
  if should_run selected "decoder-bytes" then run_decoder_bytes_tests ();
  if should_run selected "decoder-header" then run_decoder_header_tests ();
  if should_run selected "feature-policy" then run_feature_policy_tests ();
  if should_run selected "cli" then run_cli_tests ()
