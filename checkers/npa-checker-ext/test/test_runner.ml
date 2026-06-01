let assert_equal label expected actual =
  if expected <> actual then
    failwith
      (label ^ ": expected " ^ String.escaped expected ^ " but got "
     ^ String.escaped actual)

let assert_int_equal label expected actual =
  if expected <> actual then
    failwith
      (label ^ ": expected " ^ string_of_int expected ^ " but got " ^ string_of_int actual)

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

let () =
  let version = Ext_cli.run [ "--version" ] in
  assert_int_equal "version exit" 0 version.code;
  assert_equal "version stdout" "npa-checker-ext 0.1.0\n" version.stdout;
  assert_equal "version stderr" "" version.stderr;

  let no_args = Ext_cli.run [] in
  assert_int_equal "no args exit" 2 no_args.code;
  assert_equal "no args stdout" "" no_args.stdout;
  assert_equal "no args stderr" "npa-checker-ext: missing required --cert\n" no_args.stderr;

  let source_path =
    Ext_cli.run
      [
        "--cert";
        "example.npa";
        "--import-dir";
        "imports";
        "--policy";
        "policy.toml";
        "--output";
        "json";
      ]
  in
  assert_int_equal "source path exit" 2 source_path.code;
  assert_equal "source path stderr" "npa-checker-ext: --cert must not point to .npa source\n"
    source_path.stderr;

  let bad_output =
    Ext_cli.run
      [
        "--cert";
        "example.npcert";
        "--import-dir";
        "imports";
        "--policy";
        "policy.toml";
        "--output";
        "pretty";
      ]
  in
  assert_int_equal "bad output exit" 2 bad_output.code;
  assert_equal "bad output stderr" "npa-checker-ext: --output must be json\n"
    bad_output.stderr;

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
