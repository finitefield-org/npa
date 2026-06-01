let domain_level = "NPA-LEVEL-0.1"

let domain_term = "NPA-TERM-0.1"

let domain_decl_interface = "NPA-DECL-IFACE-0.1"

let domain_decl_certificate = "NPA-DECL-CERT-0.1"

let domain_generated_recursor_signature = "NPA-GEN-REC-SIG-0.1"

let domain_generated_computation_rule = "NPA-GEN-COMP-RULE-0.1"

let domain_module_export = "NPA-MODULE-EXPORT-0.1"

let domain_axiom_report = "NPA-AXIOM-REPORT-0.1"

let bind result f =
  match result with
  | Error err -> Error err
  | Ok value -> f value

exception Encode_error of Ext_bytes.decode_error

let unwrap result =
  match result with
  | Ok value -> value
  | Error err -> raise (Encode_error err)

let capture f =
  try Ok (f ()) with
  | Encode_error err -> Error err

let byte value = String.make 1 (Char.chr value)

let encode_uvar value = Ext_bytes.encode_uvar (Int64.of_int value)

let encode_hash hash = hash

let encode_string value = encode_uvar (String.length value) ^ value

let encode_name name =
  let components = Ext_name.components name in
  encode_uvar (List.length components)
  ^ String.concat "" (List.map encode_string components)

let hash_with_domain domain payload =
  Bytes.to_string (Ext_hash.sha256_raw_string (domain ^ payload))

let error section offset reason = Ext_bytes.error section offset reason

let name_id section offset name_table name =
  let rec loop index entries =
    match entries with
    | [] -> error section offset Ext_bytes.Dangling_reference
    | entry :: rest ->
        if Ext_name.equal entry.Ext_cert.name name then Ok index else loop (index + 1) rest
  in
  loop 0 name_table

let term_id section offset term_table term =
  let rec loop index entries =
    match entries with
    | [] -> error section offset Ext_bytes.Dangling_reference
    | entry :: rest ->
        if entry.Ext_term.term = term then Ok index else loop (index + 1) rest
  in
  loop 0 term_table

let encode_name_id section offset name_table name =
  bind (name_id section offset name_table name) (fun id -> Ok (encode_uvar id))

let encode_name_value section offset name_table name =
  bind (name_id section offset name_table name) (fun _ -> Ok (encode_name name))

let encode_name_values section offset name_table names =
  let rec loop remaining encoded =
    match remaining with
    | [] -> Ok (encode_uvar (List.length names) ^ String.concat "" (List.rev encoded))
    | name :: rest ->
        bind (encode_name_value section offset name_table name) (fun bytes ->
            loop rest (bytes :: encoded))
  in
  loop names []

let encode_term_id section offset term_table term =
  bind (term_id section offset term_table term) (fun id -> Ok (encode_uvar id))

let encode_global_ref section offset name_table global_ref =
  match global_ref with
  | Ext_term.Imported { import_index; name; decl_interface_hash } ->
      bind (encode_name_id section offset name_table name) (fun name_bytes ->
          Ok (byte 0x00 ^ encode_uvar import_index ^ name_bytes ^ encode_hash decl_interface_hash))
  | Ext_term.Local { decl_index } -> Ok (byte 0x01 ^ encode_uvar decl_index)
  | Ext_term.LocalGenerated { decl_index; name } ->
      bind (encode_name_id section offset name_table name) (fun name_bytes ->
          Ok (byte 0x02 ^ encode_uvar decl_index ^ name_bytes))
  | Ext_term.Builtin { name; decl_interface_hash } ->
      bind (encode_name_id section offset name_table name) (fun name_bytes ->
          Ok (byte 0x03 ^ name_bytes ^ encode_hash decl_interface_hash))

let rec level_payload level =
  match level with
  | Ext_level.Zero -> byte 0x00
  | Ext_level.Succ inner -> byte 0x01 ^ level_hash inner
  | Ext_level.Max (lhs, rhs) -> byte 0x02 ^ level_hash lhs ^ level_hash rhs
  | Ext_level.Imax (lhs, rhs) -> byte 0x03 ^ level_hash lhs ^ level_hash rhs
  | Ext_level.Param name -> byte 0x04 ^ encode_name name

and level_hash level = hash_with_domain domain_level (level_payload level)

let rec term_payload section offset name_table term =
  match term with
  | Ext_term.Sort level -> Ok (byte 0x00 ^ level_hash level)
  | Ext_term.BVar index -> Ok (byte 0x01 ^ encode_uvar index)
  | Ext_term.Const (global_ref, levels) ->
      bind (encode_global_ref section offset name_table global_ref) (fun global_ref_bytes ->
          Ok
            (byte 0x02 ^ global_ref_bytes ^ encode_uvar (List.length levels)
           ^ String.concat "" (List.map level_hash levels)))
  | Ext_term.App (fn, arg) ->
      bind (term_hash section offset name_table fn) (fun fn_hash ->
          bind (term_hash section offset name_table arg) (fun arg_hash ->
              Ok (byte 0x03 ^ fn_hash ^ arg_hash)))
  | Ext_term.Lam (ty, body) ->
      bind (term_hash section offset name_table ty) (fun ty_hash ->
          bind (term_hash section offset name_table body) (fun body_hash ->
              Ok (byte 0x04 ^ ty_hash ^ body_hash)))
  | Ext_term.Pi (ty, body) ->
      bind (term_hash section offset name_table ty) (fun ty_hash ->
          bind (term_hash section offset name_table body) (fun body_hash ->
              Ok (byte 0x05 ^ ty_hash ^ body_hash)))
  | Ext_term.Let (ty, value, body) ->
      bind (term_hash section offset name_table ty) (fun ty_hash ->
          bind (term_hash section offset name_table value) (fun value_hash ->
              bind (term_hash section offset name_table body) (fun body_hash ->
                  Ok (byte 0x06 ^ ty_hash ^ value_hash ^ body_hash))))

and term_hash section offset name_table term =
  bind (term_payload section offset name_table term) (fun payload ->
      Ok (hash_with_domain domain_term payload))

let level_hashes level_table = List.map (fun entry -> level_hash entry.Ext_level.level) level_table

let term_hashes name_table term_table =
  let rec loop entries hashes =
    match entries with
    | [] -> Ok (List.rev hashes)
    | entry :: rest ->
        bind
          (term_hash Ext_bytes.Term_table entry.Ext_term.offset name_table entry.Ext_term.term)
          (fun hash -> loop rest (hash :: hashes))
  in
  loop term_table []

let hash_for_level _level_table _level_hashes level = Ok (level_hash level)

let hash_for_term section offset name_table _term_table _term_hashes term =
  term_hash section offset name_table term

let encode_universe_constraint_relation relation =
  match relation with
  | Ext_cert.Le -> byte 0x00
  | Ext_cert.Eq -> byte 0x01

let encode_universe_constraints section offset level_table level_hashes constraints =
  let rec loop remaining encoded =
    match remaining with
    | [] -> Ok (encode_uvar (List.length constraints) ^ String.concat "" (List.rev encoded))
    | constraint_ :: rest ->
        bind
          (hash_for_level level_table level_hashes constraint_.Ext_cert.constraint_lhs)
          (fun lhs_hash ->
            bind
              (hash_for_level level_table level_hashes constraint_.Ext_cert.constraint_rhs)
              (fun rhs_hash ->
                loop rest
                  ((lhs_hash ^ encode_universe_constraint_relation constraint_.Ext_cert.constraint_relation
                   ^ rhs_hash)
                  :: encoded)))
  in
  loop constraints []

let encode_reducibility reducibility =
  match reducibility with
  | Ext_cert.Reducible -> byte 0x00
  | Ext_cert.Opaque_reducibility -> byte 0x01

let encode_opacity opacity =
  match opacity with
  | Ext_cert.Opaque -> byte 0x00

let encode_option encode value =
  match value with
  | None -> Ok (byte 0x00)
  | Some value -> bind (encode value) (fun encoded -> Ok (byte 0x01 ^ encoded))

let encode_option_hash value =
  encode_option (fun hash -> Ok (encode_hash hash)) value

let encode_option_reducibility value =
  encode_option (fun reducibility -> Ok (encode_reducibility reducibility)) value

let encode_option_opacity value = encode_option (fun opacity -> Ok (encode_opacity opacity)) value

let encode_dependency_entries section offset name_table dependencies =
  let rec loop remaining encoded =
    match remaining with
    | [] -> Ok (encode_uvar (List.length dependencies) ^ String.concat "" (List.rev encoded))
    | dependency :: rest ->
        bind
          (encode_global_ref section offset name_table dependency.Ext_cert.dependency_global_ref)
          (fun global_ref ->
            loop rest ((global_ref ^ encode_hash dependency.Ext_cert.dependency_decl_interface_hash) :: encoded))
  in
  loop dependencies []

let encode_axiom_refs section offset name_table axioms =
  let rec loop remaining encoded =
    match remaining with
    | [] -> Ok (encode_uvar (List.length axioms) ^ String.concat "" (List.rev encoded))
    | axiom :: rest ->
        bind (encode_global_ref section offset name_table axiom.Ext_cert.axiom_global_ref)
          (fun global_ref ->
            bind (encode_name_id section offset name_table axiom.Ext_cert.axiom_name)
              (fun name ->
                loop rest
                  ((global_ref ^ name ^ encode_hash axiom.Ext_cert.axiom_decl_interface_hash)
                  :: encoded)))
  in
  loop axioms []

let rec collect_global_refs_from_term term refs =
  match term with
  | Ext_term.Sort _ | Ext_term.BVar _ -> refs
  | Ext_term.Const (global_ref, _) ->
      if List.exists (( = ) global_ref) refs then refs else global_ref :: refs
  | Ext_term.App (fn, arg) ->
      collect_global_refs_from_term arg (collect_global_refs_from_term fn refs)
  | Ext_term.Lam (ty, body) | Ext_term.Pi (ty, body) ->
      collect_global_refs_from_term body (collect_global_refs_from_term ty refs)
  | Ext_term.Let (ty, value, body) ->
      collect_global_refs_from_term body
        (collect_global_refs_from_term value (collect_global_refs_from_term ty refs))

let interface_terms payload =
  match payload with
  | Ext_cert.AxiomDecl { decl_ty; _ } -> [ decl_ty ]
  | Ext_cert.DefDecl { decl_ty; decl_value; decl_reducibility; _ } ->
      if decl_reducibility = Ext_cert.Reducible then [ decl_ty; decl_value ] else [ decl_ty ]
  | Ext_cert.TheoremDecl { decl_ty; _ } -> [ decl_ty ]
  | Ext_cert.InductiveDecl { ind_params; ind_indices; ind_constructors; ind_recursor; _ } ->
      let recursor_terms =
        match ind_recursor with
        | None -> []
        | Some recursor -> [ recursor.Ext_cert.recursor_ty ]
      in
      List.map (fun binder -> binder.Ext_cert.binder_ty) ind_params
      @ List.map (fun binder -> binder.Ext_cert.binder_ty) ind_indices
      @ List.map (fun constructor -> constructor.Ext_cert.constructor_ty) ind_constructors
      @ recursor_terms
  | Ext_cert.MutualInductiveBlockDecl { mutual_inductives; _ } ->
      let terms_for_inductive inductive =
        let recursor_terms =
          match inductive.Ext_cert.mutual_recursor with
          | None -> []
          | Some recursor -> [ recursor.Ext_cert.recursor_ty ]
        in
        List.map (fun binder -> binder.Ext_cert.binder_ty) inductive.Ext_cert.mutual_params
        @ List.map
            (fun binder -> binder.Ext_cert.binder_ty)
            inductive.Ext_cert.mutual_indices
        @ List.map
            (fun constructor -> constructor.Ext_cert.constructor_ty)
            inductive.Ext_cert.mutual_constructors
        @ recursor_terms
      in
      List.concat (List.map terms_for_inductive mutual_inductives)

let interface_dependencies_for_decl payload dependencies =
  let refs =
    List.fold_left
      (fun refs term -> collect_global_refs_from_term term refs)
      [] (interface_terms payload)
  in
  List.filter
    (fun dependency -> List.exists (( = ) dependency.Ext_cert.dependency_global_ref) refs)
    dependencies

let encode_binder_type_hashes section offset name_table term_table term_hashes binders =
  let rec loop remaining encoded =
    match remaining with
    | [] -> Ok (encode_uvar (List.length binders) ^ String.concat "" (List.rev encoded))
    | binder :: rest ->
        bind
          (hash_for_term section offset name_table term_table term_hashes binder.Ext_cert.binder_ty)
          (fun hash -> loop rest (hash :: encoded))
  in
  loop binders []

let encode_constructor_specs section offset name_table term_table term_hashes constructors =
  let rec loop remaining encoded =
    match remaining with
    | [] -> Ok (encode_uvar (List.length constructors) ^ String.concat "" (List.rev encoded))
    | constructor :: rest ->
        bind (encode_name_value section offset name_table constructor.Ext_cert.constructor_name)
          (fun name ->
            bind
              (hash_for_term section offset name_table term_table term_hashes
                 constructor.Ext_cert.constructor_ty)
              (fun ty_hash -> loop rest ((name ^ ty_hash) :: encoded)))
  in
  loop constructors []

let encode_recursor_rules rules =
  encode_uvar rules.Ext_cert.minor_start ^ encode_uvar rules.Ext_cert.major_index

let generated_recursor_signature_payload section offset name_table term_table term_hashes recursor =
  match recursor with
  | None -> Ok (byte 0x00)
  | Some recursor ->
      bind (encode_name_value section offset name_table recursor.Ext_cert.recursor_name)
        (fun name ->
          bind (encode_name_values section offset name_table recursor.Ext_cert.recursor_universe_params)
            (fun universe_params ->
              bind
                (hash_for_term section offset name_table term_table term_hashes
                   recursor.Ext_cert.recursor_ty)
                (fun ty_hash -> Ok (byte 0x01 ^ name ^ universe_params ^ ty_hash))))

let generated_recursor_signature_hash section offset name_table term_table term_hashes recursor =
  bind
    (generated_recursor_signature_payload section offset name_table term_table term_hashes recursor)
    (fun payload -> Ok (hash_with_domain domain_generated_recursor_signature payload))

let generated_computation_rule_payload recursor =
  match recursor with
  | None -> byte 0x00
  | Some recursor -> byte 0x01 ^ encode_recursor_rules recursor.Ext_cert.recursor_rules

let generated_computation_rule_hash recursor =
  hash_with_domain domain_generated_computation_rule (generated_computation_rule_payload recursor)

let encode_mutual_inductive_specs section offset name_table level_table level_hashes term_table
    term_hashes inductives =
  let rec loop remaining encoded =
    match remaining with
    | [] -> Ok (encode_uvar (List.length inductives) ^ String.concat "" (List.rev encoded))
    | inductive :: rest ->
        bind (encode_name_value section offset name_table inductive.Ext_cert.mutual_name)
          (fun name ->
            bind
              (encode_binder_type_hashes section offset name_table term_table term_hashes
                 inductive.Ext_cert.mutual_params)
              (fun params ->
                bind
                  (encode_binder_type_hashes section offset name_table term_table term_hashes
                     inductive.Ext_cert.mutual_indices)
                  (fun indices ->
                    bind
                      (hash_for_level level_table level_hashes inductive.Ext_cert.mutual_sort)
                      (fun sort_hash ->
                        bind
                          (encode_constructor_specs section offset name_table term_table term_hashes
                             inductive.Ext_cert.mutual_constructors)
                          (fun constructors ->
                            bind
                              (generated_recursor_signature_hash section offset name_table
                                 term_table term_hashes inductive.Ext_cert.mutual_recursor)
                              (fun recursor_sig_hash ->
                                let recursor_rule_hash =
                                  generated_computation_rule_hash
                                    inductive.Ext_cert.mutual_recursor
                                in
                                loop rest
                                  ((name ^ params ^ indices ^ sort_hash ^ constructors
                                   ^ recursor_sig_hash ^ recursor_rule_hash)
                                  :: encoded)))))))
  in
  loop inductives []

let declaration_interface_payload name_table level_table term_table payload dependencies
    axiom_dependencies =
  capture (fun () ->
      let level_hashes = level_hashes level_table in
      let term_hashes = unwrap (term_hashes name_table term_table) in
      let section = Ext_bytes.Declarations in
      let offset = 0 in
      let name = encode_name_value section offset name_table in
      let names = encode_name_values section offset name_table in
      let term = hash_for_term section offset name_table term_table term_hashes in
      let level = hash_for_level level_table level_hashes in
      let constraints = encode_universe_constraints section offset level_table level_hashes in
      let interface_dependencies = interface_dependencies_for_decl payload dependencies in
      let deps = encode_dependency_entries section offset name_table interface_dependencies in
      let axioms = encode_axiom_refs section offset name_table axiom_dependencies in
      match payload with
      | Ext_cert.AxiomDecl { decl_name; decl_universe_params; decl_universe_constraints = []; decl_ty }
        ->
          byte 0x00 ^ unwrap (name decl_name) ^ unwrap (names decl_universe_params)
          ^ unwrap (term decl_ty) ^ unwrap deps
      | Ext_cert.AxiomDecl { decl_name; decl_universe_params; decl_universe_constraints; decl_ty }
        ->
          byte 0x10 ^ unwrap (name decl_name) ^ unwrap (names decl_universe_params)
          ^ unwrap (constraints decl_universe_constraints) ^ unwrap (term decl_ty) ^ unwrap deps
      | Ext_cert.DefDecl
          {
            decl_name;
            decl_universe_params;
            decl_universe_constraints = [];
            decl_ty;
            decl_value;
            decl_reducibility;
          } ->
          byte 0x01 ^ unwrap (name decl_name) ^ unwrap (names decl_universe_params)
          ^ unwrap (term decl_ty) ^ encode_reducibility decl_reducibility ^ unwrap deps
          ^ unwrap axioms
          ^
          if decl_reducibility = Ext_cert.Reducible then unwrap (term decl_value) else ""
      | Ext_cert.DefDecl
          {
            decl_name;
            decl_universe_params;
            decl_universe_constraints;
            decl_ty;
            decl_value;
            decl_reducibility;
          } ->
          byte 0x11 ^ unwrap (name decl_name) ^ unwrap (names decl_universe_params)
          ^ unwrap (constraints decl_universe_constraints) ^ unwrap (term decl_ty)
          ^ encode_reducibility decl_reducibility ^ unwrap deps ^ unwrap axioms
          ^
          if decl_reducibility = Ext_cert.Reducible then unwrap (term decl_value) else ""
      | Ext_cert.TheoremDecl
          {
            decl_name;
            decl_universe_params;
            decl_universe_constraints = [];
            decl_ty;
            decl_opacity;
            _;
          } ->
          byte 0x02 ^ unwrap (name decl_name) ^ unwrap (names decl_universe_params)
          ^ unwrap (term decl_ty) ^ encode_opacity decl_opacity ^ unwrap deps
          ^ unwrap axioms
      | Ext_cert.TheoremDecl
          {
            decl_name;
            decl_universe_params;
            decl_universe_constraints;
            decl_ty;
            decl_opacity;
            _;
          } ->
          byte 0x12 ^ unwrap (name decl_name) ^ unwrap (names decl_universe_params)
          ^ unwrap (constraints decl_universe_constraints) ^ unwrap (term decl_ty)
          ^ encode_opacity decl_opacity ^ unwrap deps ^ unwrap axioms
      | Ext_cert.InductiveDecl
          {
            decl_name;
            decl_universe_params;
            decl_universe_constraints = [];
            ind_params;
            ind_indices;
            ind_sort;
            ind_constructors;
            ind_recursor;
          } ->
          byte 0x03 ^ unwrap (name decl_name) ^ unwrap (names decl_universe_params)
          ^ unwrap
              (encode_binder_type_hashes section offset name_table term_table term_hashes
                 ind_params)
          ^ unwrap
              (encode_binder_type_hashes section offset name_table term_table term_hashes
                 ind_indices)
          ^ unwrap (level ind_sort)
          ^ unwrap
              (encode_constructor_specs section offset name_table term_table term_hashes
                 ind_constructors)
          ^ unwrap
              (generated_recursor_signature_hash section offset name_table term_table term_hashes
                 ind_recursor)
          ^ generated_computation_rule_hash ind_recursor ^ unwrap deps ^ unwrap axioms
      | Ext_cert.InductiveDecl
          {
            decl_name;
            decl_universe_params;
            decl_universe_constraints;
            ind_params;
            ind_indices;
            ind_sort;
            ind_constructors;
            ind_recursor;
          } ->
          byte 0x13 ^ unwrap (name decl_name) ^ unwrap (names decl_universe_params)
          ^ unwrap (constraints decl_universe_constraints)
          ^ unwrap
              (encode_binder_type_hashes section offset name_table term_table term_hashes
                 ind_params)
          ^ unwrap
              (encode_binder_type_hashes section offset name_table term_table term_hashes
                 ind_indices)
          ^ unwrap (level ind_sort)
          ^ unwrap
              (encode_constructor_specs section offset name_table term_table term_hashes
                 ind_constructors)
          ^ unwrap
              (generated_recursor_signature_hash section offset name_table term_table term_hashes
                 ind_recursor)
          ^ generated_computation_rule_hash ind_recursor ^ unwrap deps ^ unwrap axioms
      | Ext_cert.MutualInductiveBlockDecl
          { decl_name; decl_universe_params; decl_universe_constraints; mutual_inductives } ->
          byte 0x04 ^ unwrap (name decl_name) ^ unwrap (names decl_universe_params)
          ^ unwrap (constraints decl_universe_constraints)
          ^ unwrap
              (encode_mutual_inductive_specs section offset name_table level_table level_hashes
                 term_table term_hashes mutual_inductives)
          ^ unwrap deps ^ unwrap axioms)

let declaration_certificate_payload name_table term_table payload interface_hash dependencies
    axiom_dependencies =
  bind (term_hashes name_table term_table) (fun term_hashes ->
      let section = Ext_bytes.Declarations in
      let offset = 0 in
      let term = hash_for_term section offset name_table term_table term_hashes in
      let deps = encode_dependency_entries section offset name_table dependencies in
      let axioms = encode_axiom_refs section offset name_table axiom_dependencies in
      match payload with
      | Ext_cert.AxiomDecl _ -> bind axioms (fun axioms -> Ok (interface_hash ^ axioms))
      | Ext_cert.DefDecl { decl_value; _ } ->
          bind (term decl_value) (fun value ->
              bind deps (fun deps ->
                  bind axioms (fun axioms -> Ok (interface_hash ^ value ^ deps ^ axioms))))
      | Ext_cert.TheoremDecl { decl_proof; _ } ->
          bind (term decl_proof) (fun proof ->
              bind deps (fun deps -> Ok (interface_hash ^ proof ^ deps)))
      | Ext_cert.InductiveDecl _ | Ext_cert.MutualInductiveBlockDecl _ ->
          bind deps (fun deps ->
              bind axioms (fun axioms -> Ok (interface_hash ^ deps ^ axioms))))

let encode_export_kind kind =
  match kind with
  | Ext_cert.Export_axiom -> byte 0x00
  | Ext_cert.Export_def -> byte 0x01
  | Ext_cert.Export_theorem -> byte 0x02
  | Ext_cert.Export_inductive -> byte 0x03
  | Ext_cert.Export_constructor -> byte 0x04
  | Ext_cert.Export_recursor -> byte 0x05

let encode_usize_vector values =
  encode_uvar (List.length values) ^ String.concat "" (List.map encode_uvar values)

let list_name_ids section offset name_table names =
  let rec loop remaining ids =
    match remaining with
    | [] -> Ok (List.rev ids)
    | name :: rest ->
        bind (name_id section offset name_table name) (fun id -> loop rest (id :: ids))
  in
  loop names []

let encode_option_usize value =
  match value with
  | None -> Ok (byte 0x00)
  | Some value -> Ok (byte 0x01 ^ encode_uvar value)

let encode_export_block decoded =
  let name_table = decoded.Ext_cert.name_table in
  let term_table = decoded.Ext_cert.term_table in
  let section = Ext_bytes.Export_block in
  let rec loop remaining encoded =
    match remaining with
    | [] -> Ok (encode_uvar (List.length decoded.Ext_cert.export_block) ^ String.concat "" (List.rev encoded))
    | export :: rest ->
        let offset = export.Ext_cert.export_offset in
        bind (name_id section offset name_table export.Ext_cert.export_name) (fun export_name_id ->
            bind
              (list_name_ids section offset name_table export.Ext_cert.export_universe_params)
              (fun universe_param_ids ->
                bind (term_id section offset term_table export.Ext_cert.export_ty) (fun ty_id ->
                    bind
                      (match export.Ext_cert.export_body with
                      | None -> encode_option_usize None
                      | Some body ->
                          bind (term_id section offset term_table body) (fun body_id ->
                              encode_option_usize (Some body_id)))
                      (fun body ->
                        bind (encode_option_hash export.Ext_cert.export_body_hash) (fun body_hash ->
                            bind (encode_option_reducibility export.Ext_cert.export_reducibility)
                              (fun reducibility ->
                                bind (encode_option_opacity export.Ext_cert.export_opacity)
                                  (fun opacity ->
                                    bind
                                      (encode_axiom_refs section offset name_table
                                         export.Ext_cert.export_axiom_dependencies)
                                      (fun axioms ->
                                        loop rest
                                          ((encode_uvar export_name_id
                                           ^ encode_export_kind export.Ext_cert.export_kind
                                           ^ encode_usize_vector universe_param_ids
                                           ^ encode_uvar ty_id ^ body
                                           ^ encode_hash export.Ext_cert.export_type_hash ^ body_hash
                                           ^ reducibility ^ opacity
                                           ^ encode_hash export.Ext_cert.export_decl_interface_hash
                                           ^ axioms)
                                          :: encoded)))))))))
  in
  loop decoded.Ext_cert.export_block []

let encode_axiom_report name_table report =
  let section = Ext_bytes.Axiom_report in
  let rec encode_decl_reports remaining encoded =
    match remaining with
    | [] -> Ok (encode_uvar (List.length report.Ext_cert.per_declaration) ^ String.concat "" (List.rev encoded))
    | entry :: rest ->
        let offset = entry.Ext_cert.report_offset in
        bind (encode_axiom_refs section offset name_table entry.Ext_cert.report_direct_axioms)
          (fun direct ->
            bind (encode_axiom_refs section offset name_table entry.Ext_cert.report_transitive_axioms)
              (fun transitive ->
                encode_decl_reports rest
                  ((encode_uvar entry.Ext_cert.report_decl_index ^ direct ^ transitive) :: encoded)))
  in
  bind (encode_decl_reports report.Ext_cert.per_declaration []) (fun per_declaration ->
      bind
        (encode_axiom_refs section report.Ext_cert.module_axioms_offset name_table
           report.Ext_cert.module_axioms)
        (fun module_axioms ->
          let core_features =
            match report.Ext_cert.core_features with
            | [] -> ""
            | features ->
                encode_string Ext_cert.core_feature_report_tag
                ^ encode_uvar (List.length features)
                ^ String.concat ""
                    (List.map
                       (fun feature -> encode_string feature.Ext_feature.feature)
                       features)
          in
          Ok (per_declaration ^ module_axioms ^ core_features)))

let export_hash decoded =
  bind (encode_export_block decoded) (fun payload -> Ok (hash_with_domain domain_module_export payload))

let axiom_report_hash decoded =
  bind (encode_axiom_report decoded.Ext_cert.name_table decoded.Ext_cert.axiom_report) (fun payload ->
      Ok (hash_with_domain domain_axiom_report payload))
