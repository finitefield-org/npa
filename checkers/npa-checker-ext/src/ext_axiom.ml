type policy = {
  deny_sorry : bool;
  deny_custom_axioms : bool;
  allowed_axioms : Ext_name.t list;
}

let default_policy =
  { deny_sorry = true; deny_custom_axioms = true; allowed_axioms = [] }

type error = {
  section : Ext_bytes.certificate_section;
  offset : Ext_bytes.offset;
}

let error section offset = Error { section; offset }

let bind result f =
  match result with
  | Error err -> Error err
  | Ok value -> f value

let error_kind _ = "axiom_report_mismatch"

let error_reason_code _ = "axiom_report_mismatch"

let rec list_nth_opt index values =
  match (index, values) with
  | _, _ when index < 0 -> None
  | 0, value :: _ -> Some value
  | _, _ :: rest -> list_nth_opt (index - 1) rest
  | _, [] -> None

let builtin_is_axiom name = Ext_name.to_string name = "Eq.rec"

let global_ref_equal left right = left = right

let dependency_equal left right =
  global_ref_equal left.Ext_cert.dependency_global_ref
    right.Ext_cert.dependency_global_ref
  && left.Ext_cert.dependency_decl_interface_hash
     = right.Ext_cert.dependency_decl_interface_hash

let axiom_equal left right =
  global_ref_equal left.Ext_cert.axiom_global_ref right.Ext_cert.axiom_global_ref
  && Ext_name.equal left.Ext_cert.axiom_name right.Ext_cert.axiom_name
  && left.Ext_cert.axiom_decl_interface_hash
     = right.Ext_cert.axiom_decl_interface_hash

let rec list_equal equal left right =
  match (left, right) with
  | [], [] -> true
  | left_value :: left_rest, right_value :: right_rest ->
      equal left_value right_value && list_equal equal left_rest right_rest
  | _ -> false

let name_id section offset name_table name =
  let rec loop index remaining =
    match remaining with
    | [] -> error section offset
    | entry :: rest ->
        if Ext_name.equal entry.Ext_cert.name name then Ok index
        else loop (index + 1) rest
  in
  loop 0 name_table

let encode_order_uvar value =
  let buffer = Buffer.create 5 in
  let rec loop current =
    let byte = current land 0x7f in
    let next = current lsr 7 in
    if next = 0 then Buffer.add_char buffer (Char.chr byte)
    else (
      Buffer.add_char buffer (Char.chr (byte lor 0x80));
      loop next)
  in
  if value < 0 then invalid_arg "negative uvar order key" else loop value;
  Buffer.contents buffer

let global_ref_order_key section offset name_table global_ref =
  match global_ref with
  | Ext_term.Imported { import_index; name; decl_interface_hash } ->
      bind (name_id section offset name_table name) (fun name_index ->
          Ok
            ("\000" ^ encode_order_uvar import_index
            ^ encode_order_uvar name_index ^ decl_interface_hash))
  | Ext_term.Local { decl_index } ->
      Ok ("\001" ^ encode_order_uvar decl_index)
  | Ext_term.LocalGenerated { decl_index; name } ->
      bind (name_id section offset name_table name) (fun name_index ->
          Ok ("\002" ^ encode_order_uvar decl_index ^ encode_order_uvar name_index))
  | Ext_term.Builtin { name; decl_interface_hash } ->
      bind (name_id section offset name_table name) (fun name_index ->
          Ok ("\003" ^ encode_order_uvar name_index ^ decl_interface_hash))

let axiom_order_key section offset name_table axiom =
  bind
    (global_ref_order_key section offset name_table axiom.Ext_cert.axiom_global_ref)
    (fun global_key ->
      bind (name_id section offset name_table axiom.Ext_cert.axiom_name)
        (fun name_index ->
          Ok
            (global_key ^ encode_order_uvar name_index
            ^ axiom.Ext_cert.axiom_decl_interface_hash)))

let dependency_order_key section offset name_table dependency =
  bind
    (global_ref_order_key section offset name_table
       dependency.Ext_cert.dependency_global_ref)
    (fun global_key ->
      Ok (global_key ^ dependency.Ext_cert.dependency_decl_interface_hash))

let sort_unique_by_key section offset name_table key_fn equal values =
  let rec key_values remaining keyed =
    match remaining with
    | [] -> Ok keyed
    | value :: rest ->
        bind (key_fn section offset name_table value) (fun key ->
            key_values rest ((key, value) :: keyed))
  in
  bind (key_values values []) (fun keyed ->
      let sorted =
        List.sort (fun (left, _) (right, _) -> String.compare left right) keyed
      in
      let rec unique remaining previous values =
        match remaining with
        | [] -> Ok (List.rev values)
        | (_, value) :: rest -> (
            match previous with
            | Some previous_value when equal previous_value value ->
                unique rest previous values
            | _ -> unique rest (Some value) (value :: values))
      in
      unique sorted None [])

let sort_unique_axioms section offset name_table axioms =
  sort_unique_by_key section offset name_table axiom_order_key axiom_equal axioms

let sort_unique_dependencies section offset name_table dependencies =
  sort_unique_by_key section offset name_table dependency_order_key
    dependency_equal dependencies

let rec append_global_refs term refs =
  Ext_canonical.collect_global_refs_from_term term refs

let declaration_terms payload =
  match payload with
  | Ext_cert.AxiomDecl { decl_ty; _ } -> [ decl_ty ]
  | Ext_cert.DefDecl { decl_ty; decl_value; _ } -> [ decl_ty; decl_value ]
  | Ext_cert.TheoremDecl { decl_ty; decl_proof; _ } -> [ decl_ty; decl_proof ]
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

let generated_name_exists declaration name =
  match declaration.Ext_cert.payload with
  | Ext_cert.InductiveDecl { ind_constructors; ind_recursor; _ } ->
      List.exists
        (fun constructor ->
          Ext_name.equal constructor.Ext_cert.constructor_name name)
        ind_constructors
      ||
      (match ind_recursor with
      | None -> false
      | Some recursor -> Ext_name.equal recursor.Ext_cert.recursor_name name)
  | Ext_cert.MutualInductiveBlockDecl { mutual_inductives; _ } ->
      List.exists
        (fun inductive ->
          Ext_name.equal inductive.Ext_cert.mutual_name name
          || List.exists
               (fun constructor ->
                 Ext_name.equal constructor.Ext_cert.constructor_name name)
               inductive.Ext_cert.mutual_constructors
          ||
          match inductive.Ext_cert.mutual_recursor with
          | None -> false
          | Some recursor -> Ext_name.equal recursor.Ext_cert.recursor_name name)
        mutual_inductives
  | _ -> false

let find_import import_index imports =
  list_nth_opt import_index (Ext_import_store.import_environment_imports imports)

let find_public_export name decl_interface_hash exports =
  let rec loop remaining =
    match remaining with
    | [] -> None
    | export :: rest ->
        if
          Ext_name.equal export.Ext_import_store.public_export_name name
          && export.Ext_import_store.public_decl_interface_hash = decl_interface_hash
        then Some export
        else loop rest
  in
  loop exports

let imported_export_for_global_ref section offset imports global_ref =
  match global_ref with
  | Ext_term.Imported { import_index; name; decl_interface_hash } -> (
      match find_import import_index imports with
      | None -> error section offset
      | Some import -> (
          match
            find_public_export name decl_interface_hash
              import.Ext_import_store.resolved_public_environment
                .Ext_import_store.public_exports
          with
          | None -> error section offset
          | Some export -> Ok export))
  | _ -> error section offset

let interface_hash_for_global_ref section offset name_table imports current_decl_index
    (declarations : Ext_cert.declaration list) global_ref =
  match global_ref with
  | Ext_term.Builtin { name; decl_interface_hash } -> (
      match Ext_env.builtin_decl_interface_hash name with
      | Some expected when expected = decl_interface_hash -> Ok decl_interface_hash
      | _ -> error section offset)
  | Ext_term.Imported { decl_interface_hash; _ } ->
      bind
        (imported_export_for_global_ref section offset imports global_ref)
        (fun _ -> Ok decl_interface_hash)
  | Ext_term.Local { decl_index } -> (
      if decl_index >= current_decl_index then error section offset
      else
        match list_nth_opt decl_index declarations with
        | None -> error section offset
        | Some declaration ->
            Ok (declaration.Ext_cert.hashes).Ext_cert.decl_interface_hash)
  | Ext_term.LocalGenerated { decl_index; name } -> (
      if decl_index >= current_decl_index then error section offset
      else
        match list_nth_opt decl_index declarations with
        | Some declaration when generated_name_exists declaration name ->
            Ok (declaration.Ext_cert.hashes).Ext_cert.decl_interface_hash
        | _ -> error section offset)

let allow_self_reference payload =
  match payload with
  | Ext_cert.InductiveDecl _ | Ext_cert.MutualInductiveBlockDecl _ -> true
  | _ -> false

let expected_dependencies_for_decl section offset name_table imports decl_index
    declarations declaration =
  let refs =
    List.fold_left
      (fun refs term -> append_global_refs term refs)
      [] (declaration_terms declaration.Ext_cert.payload)
  in
  let refs =
    List.filter
      (function
        | Ext_term.Local { decl_index = referenced_decl_index }
        | Ext_term.LocalGenerated { decl_index = referenced_decl_index; _ }
          when allow_self_reference declaration.Ext_cert.payload
               && referenced_decl_index = decl_index ->
            false
        | _ -> true)
      refs
  in
  let rec loop remaining dependencies =
    match remaining with
    | [] -> sort_unique_dependencies section offset name_table dependencies
    | global_ref :: rest ->
        bind
          (interface_hash_for_global_ref section offset name_table imports decl_index
             declarations global_ref)
          (fun decl_interface_hash ->
            loop rest
              ({
                 Ext_cert.dependency_global_ref = global_ref;
                 dependency_decl_interface_hash = decl_interface_hash;
               }
              :: dependencies))
  in
  loop refs []

let local_axiom_ref_for_decl decl_index axioms =
  let rec loop remaining =
    match remaining with
    | [] -> None
    | axiom :: rest -> (
        match axiom.Ext_cert.axiom_global_ref with
        | Ext_term.Local { decl_index = axiom_decl_index }
          when axiom_decl_index = decl_index ->
            Some axiom
        | _ -> loop rest)
  in
  loop axioms

let import_index_exporting_axiom imports name decl_interface_hash =
  let rec loop index remaining =
    match remaining with
    | [] -> None
    | import :: rest ->
        if
          List.exists
            (fun export ->
              export.Ext_import_store.public_export_kind = Ext_cert.Export_axiom
              && Ext_name.equal export.Ext_import_store.public_export_name name
              && export.Ext_import_store.public_decl_interface_hash
                 = decl_interface_hash)
            import.Ext_import_store.resolved_public_environment
              .Ext_import_store.public_exports
        then Some index
        else loop (index + 1) rest
  in
  loop 0 (Ext_import_store.import_environment_imports imports)

let remap_imported_axiom_dependency section offset name_table imports axiom =
  bind (name_id section offset name_table axiom.Ext_cert.axiom_name) (fun _ ->
      match
        import_index_exporting_axiom imports axiom.Ext_cert.axiom_name
          axiom.Ext_cert.axiom_decl_interface_hash
      with
      | Some import_index ->
          Ok
            {
              axiom with
              Ext_cert.axiom_global_ref =
                Ext_term.Imported
                  {
                    import_index;
                    name = axiom.Ext_cert.axiom_name;
                    decl_interface_hash =
                      axiom.Ext_cert.axiom_decl_interface_hash;
                  };
            }
      | None ->
          if
            builtin_is_axiom axiom.Ext_cert.axiom_name
            && Ext_env.builtin_decl_interface_hash axiom.Ext_cert.axiom_name
               = Some axiom.Ext_cert.axiom_decl_interface_hash
          then
            Ok
              {
                axiom with
                Ext_cert.axiom_global_ref =
                  Ext_term.Builtin
                    {
                      name = axiom.Ext_cert.axiom_name;
                      decl_interface_hash =
                        axiom.Ext_cert.axiom_decl_interface_hash;
                    };
              }
          else error section offset)

let expected_axioms_for_decl section offset name_table imports decl_index declaration
    dependencies previous_axioms =
  let direct = ref [] in
  let transitive = ref [] in
  let add_direct axiom = direct := axiom :: !direct in
  let add_transitive axiom = transitive := axiom :: !transitive in
  let add_transitive_all axioms = transitive := axioms @ !transitive in
  let rec loop_dependencies remaining =
    match remaining with
    | [] -> Ok ()
    | dependency :: rest -> (
        match dependency.Ext_cert.dependency_global_ref with
        | Ext_term.Builtin { name; decl_interface_hash } ->
            if builtin_is_axiom name then (
              let axiom =
                {
                  Ext_cert.axiom_global_ref =
                    dependency.Ext_cert.dependency_global_ref;
                  axiom_name = name;
                  axiom_decl_interface_hash = decl_interface_hash;
                }
              in
              add_direct axiom;
              add_transitive axiom);
            loop_dependencies rest
        | Ext_term.Local { decl_index = dependency_index } -> (
            match list_nth_opt dependency_index previous_axioms with
            | None -> error section offset
            | Some dep_axioms ->
                (match local_axiom_ref_for_decl dependency_index dep_axioms with
                | None -> ()
                | Some axiom -> add_direct axiom);
                add_transitive_all dep_axioms;
                loop_dependencies rest)
        | Ext_term.LocalGenerated { decl_index = dependency_index; _ } -> (
            match list_nth_opt dependency_index previous_axioms with
            | None -> error section offset
            | Some dep_axioms ->
                add_transitive_all dep_axioms;
                loop_dependencies rest)
        | Ext_term.Imported { name; decl_interface_hash; _ } ->
            bind
              (imported_export_for_global_ref section offset imports
                 dependency.Ext_cert.dependency_global_ref)
              (fun export ->
                if export.Ext_import_store.public_export_kind = Ext_cert.Export_axiom then
                  add_direct
                    {
                      Ext_cert.axiom_global_ref =
                        dependency.Ext_cert.dependency_global_ref;
                      axiom_name = name;
                      axiom_decl_interface_hash = decl_interface_hash;
                    };
                let rec loop_axioms remaining =
                  match remaining with
                  | [] -> Ok ()
                  | axiom :: rest_axioms ->
                      bind
                        (remap_imported_axiom_dependency Ext_bytes.Axiom_report
                           offset name_table imports axiom)
                        (fun remapped ->
                          add_transitive remapped;
                          loop_axioms rest_axioms)
                in
                bind
                  (loop_axioms
                     export.Ext_import_store.public_axiom_dependencies)
                  (fun () -> loop_dependencies rest)))
  in
  bind (loop_dependencies dependencies) (fun () ->
      (match declaration.Ext_cert.payload with
      | Ext_cert.AxiomDecl { decl_name; _ } ->
          let self_ref =
            {
              Ext_cert.axiom_global_ref = Ext_term.Local { decl_index };
              axiom_name = decl_name;
              axiom_decl_interface_hash =
                (declaration.Ext_cert.hashes).Ext_cert.decl_interface_hash;
            }
          in
          add_direct self_ref;
          add_transitive self_ref
      | _ -> ());
      bind (sort_unique_axioms section offset name_table !direct) (fun direct ->
          bind (sort_unique_axioms section offset name_table !transitive)
            (fun transitive -> Ok (direct, transitive))))

let recompute_axiom_report imports (decoded : Ext_cert.decoded_module) =
  let section = Ext_bytes.Axiom_report in
  let name_table = decoded.Ext_cert.name_table in
  if
    List.length decoded.Ext_cert.axiom_report.Ext_cert.per_declaration
    <> List.length decoded.Ext_cert.declaration_table
  then
    error section decoded.Ext_cert.axiom_report.Ext_cert.module_axioms_offset
  else
    let rec loop decl_index (declarations : Ext_cert.declaration list)
        previous_axioms reports transitive_by_decl =
      match declarations with
      | [] ->
          bind
            (sort_unique_axioms section
               decoded.Ext_cert.axiom_report.Ext_cert.module_axioms_offset
               name_table (List.concat (List.rev transitive_by_decl)))
            (fun module_axioms ->
              Ok
                {
                  decoded.Ext_cert.axiom_report with
                  Ext_cert.per_declaration = List.rev reports;
                  module_axioms;
                })
      | declaration :: rest -> (
          let actual_report =
            list_nth_opt decl_index
              decoded.Ext_cert.axiom_report.Ext_cert.per_declaration
          in
          match actual_report with
          | None ->
              error section
                decoded.Ext_cert.axiom_report.Ext_cert.module_axioms_offset
          | Some actual_report -> (
              let offset = declaration.Ext_cert.offset in
              match
                expected_dependencies_for_decl Ext_bytes.Declarations offset name_table
                  imports decl_index decoded.Ext_cert.declaration_table declaration
              with
              | Error err -> Error err
              | Ok dependencies ->
                  if
                    not
                      (list_equal dependency_equal dependencies
                         declaration.Ext_cert.dependencies)
                  then error Ext_bytes.Declarations offset
                  else
                    bind
                      (expected_axioms_for_decl Ext_bytes.Declarations offset name_table
                         imports decl_index declaration dependencies previous_axioms)
                      (fun (direct_axioms, transitive_axioms) ->
                        let report =
                          {
                            actual_report with
                            Ext_cert.report_decl_index = decl_index;
                            report_direct_axioms = direct_axioms;
                            report_transitive_axioms = transitive_axioms;
                          }
                        in
                        loop (decl_index + 1) rest
                          (previous_axioms @ [ transitive_axioms ])
                          (report :: reports)
                          (transitive_axioms :: transitive_by_decl))))
    in
    loop 0 decoded.Ext_cert.declaration_table [] [] []

let verify_axiom_report imports (decoded : Ext_cert.decoded_module) =
  let stored_report = decoded.Ext_cert.axiom_report in
  bind (recompute_axiom_report imports decoded) (fun expected_report ->
      let rec compare_declaration_reports expected actual =
        match (expected, actual) with
        | [], [] -> Ok ()
        | expected_entry :: expected_rest, actual_entry :: actual_rest ->
            if
              expected_entry.Ext_cert.report_decl_index
              <> actual_entry.Ext_cert.report_decl_index
              || not
                   (list_equal axiom_equal
                      expected_entry.Ext_cert.report_direct_axioms
                      actual_entry.Ext_cert.report_direct_axioms)
              || not
                   (list_equal axiom_equal
                      expected_entry.Ext_cert.report_transitive_axioms
                      actual_entry.Ext_cert.report_transitive_axioms)
            then error Ext_bytes.Axiom_report actual_entry.Ext_cert.report_offset
            else compare_declaration_reports expected_rest actual_rest
        | _ ->
            error Ext_bytes.Axiom_report stored_report.Ext_cert.module_axioms_offset
      in
      bind
        (compare_declaration_reports expected_report.Ext_cert.per_declaration
           stored_report.Ext_cert.per_declaration)
        (fun () ->
          if
            not
              (list_equal axiom_equal expected_report.Ext_cert.module_axioms
                 stored_report.Ext_cert.module_axioms)
          then
            error Ext_bytes.Axiom_report
              stored_report.Ext_cert.module_axioms_offset
          else
            let rec compare_declaration_axioms decl_index declarations =
              match declarations with
              | [] -> Ok ()
              | declaration :: rest -> (
                  match
                    list_nth_opt decl_index
                      expected_report.Ext_cert.per_declaration
                  with
                  | None ->
                      error Ext_bytes.Axiom_report
                        stored_report.Ext_cert.module_axioms_offset
                  | Some report ->
                      if
                        list_equal axiom_equal declaration.Ext_cert.axiom_dependencies
                          report.Ext_cert.report_transitive_axioms
                      then compare_declaration_axioms (decl_index + 1) rest
                      else error Ext_bytes.Declarations declaration.Ext_cert.offset)
            in
            bind
              (compare_declaration_axioms 0 decoded.Ext_cert.declaration_table)
              (fun () ->
                match
                  Ext_canonical.encode_axiom_report decoded.Ext_cert.name_table
                    expected_report
                with
                | Error _ ->
                    error Ext_bytes.Axiom_report
                      stored_report.Ext_cert.module_axioms_offset
                | Ok payload ->
                    let expected_hash =
                      Ext_canonical.hash_with_domain
                        Ext_canonical.domain_axiom_report payload
                    in
                    if expected_hash = (decoded.Ext_cert.hashes).Ext_cert.axiom_report_hash
                    then Ok ()
                    else
                      error Ext_bytes.Hashes
                        (decoded.Ext_cert.hashes).Ext_cert.axiom_report_hash_offset)))
