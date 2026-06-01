let assert_equal label expected actual =
  if expected <> actual then
    failwith
      (label ^ ": expected " ^ String.escaped expected ^ " but got "
     ^ String.escaped actual)

let assert_int_equal label expected actual =
  if expected <> actual then
    failwith
      (label ^ ": expected " ^ string_of_int expected ^ " but got " ^ string_of_int actual)

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

let should_run selected name = selected = [] || List.mem name selected

let () =
  let selected = Array.to_list Sys.argv |> List.tl in
  List.iter
    (fun name ->
      if not (List.mem name [ "cli"; "sha256" ]) then failwith ("unknown test filter " ^ name))
    selected;
  if should_run selected "sha256" then run_sha256_tests ();
  if should_run selected "cli" then run_cli_tests ()
