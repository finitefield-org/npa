type check_result =
  | Type_checked
  | Type_check_not_implemented

type error_reason =
  | Unknown_reference
  | Bad_universe_arity
  | Duplicate_universe_param
  | Unresolved_metavariable
  | Invalid_bvar
  | Expected_sort
  | Expected_function
  | Type_mismatch
  | Unsupported_declaration
  | Resource_limit

type error = {
  reason : error_reason;
  section : Ext_bytes.certificate_section;
  offset : Ext_bytes.offset;
}

type local_binding = {
  local_ty : Ext_term.t;
  local_value : Ext_term.t option;
}

type context = local_binding list

let max_fuel = 100_000

let empty_context = []

let push_assumption context ty = { local_ty = ty; local_value = None } :: context

let push_definition context ty value =
  { local_ty = ty; local_value = Some value } :: context

let error section offset reason = Error { reason; section; offset }

let bind result f =
  match result with
  | Error err -> Error err
  | Ok value -> f value

let error_reason_code reason =
  match reason with
  | Unknown_reference -> "unknown_reference"
  | Bad_universe_arity -> "bad_universe_arity"
  | Duplicate_universe_param -> "duplicate_universe_param"
  | Unresolved_metavariable -> "unresolved_metavariable"
  | Invalid_bvar -> "invalid_bvar"
  | Expected_sort -> "expected_sort"
  | Expected_function -> "expected_function"
  | Type_mismatch -> "type_mismatch"
  | Unsupported_declaration -> "unsupported_declaration"
  | Resource_limit -> "resource_limit"

let error_kind error =
  match error.reason with
  | Bad_universe_arity | Duplicate_universe_param | Unresolved_metavariable ->
      "universe_inconsistency"
  | Unknown_reference | Invalid_bvar | Expected_sort | Expected_function | Type_mismatch
  | Unsupported_declaration | Resource_limit ->
      "type_mismatch"

let error_of_env_error (env_error : Ext_env.error) =
  let reason =
    match env_error.Ext_env.reason with
    | Ext_env.Unknown_reference -> Unknown_reference
    | Ext_env.Duplicate_universe_param -> Duplicate_universe_param
  in
  Error
    {
      reason;
      section = env_error.Ext_env.section;
      offset = env_error.Ext_env.offset;
    }

let resolve_signature section offset env global_ref =
  match Ext_env.resolve_global_ref ~section ~offset env global_ref with
  | Ok signature -> Ok signature
  | Error env_error -> error_of_env_error env_error

let rec list_nth_opt index values =
  match (index, values) with
  | _, _ when index < 0 -> None
  | 0, value :: _ -> Some value
  | _, _ :: rest -> list_nth_opt (index - 1) rest
  | _, [] -> None

let spend_fuel section offset fuel =
  if !fuel = 0 then error section offset Resource_limit
  else (
    fuel := !fuel - 1;
    Ok ())

let rec position_name name params =
  match params with
  | [] -> None
  | current :: rest ->
      if Ext_name.equal current name then Some 0
      else (
        match position_name name rest with
        | None -> None
        | Some index -> Some (index + 1))

let rec ensure_level_wf section offset delta level =
  match level with
  | Ext_level.Zero -> Ok ()
  | Ext_level.Succ inner -> ensure_level_wf section offset delta inner
  | Ext_level.Max (lhs, rhs) | Ext_level.Imax (lhs, rhs) ->
      bind (ensure_level_wf section offset delta lhs) (fun () ->
          ensure_level_wf section offset delta rhs)
  | Ext_level.Param name ->
      if Ext_level.component_contains_universe_meta name then
        error section offset Unresolved_metavariable
      else if position_name name delta <> None then Ok ()
      else error section offset Unknown_reference

let rec subst_level params levels level =
  match level with
  | Ext_level.Zero -> Ext_level.Zero
  | Ext_level.Succ inner -> Ext_level.Succ (subst_level params levels inner)
  | Ext_level.Max (lhs, rhs) ->
      Ext_level.Max (subst_level params levels lhs, subst_level params levels rhs)
  | Ext_level.Imax (lhs, rhs) ->
      Ext_level.Imax (subst_level params levels lhs, subst_level params levels rhs)
  | Ext_level.Param name -> (
      match position_name name params with
      | None -> Ext_level.Param name
      | Some index -> (
          match list_nth_opt index levels with
          | None -> Ext_level.Param name
          | Some level -> level))

let rec subst_levels_term params levels term =
  match term with
  | Ext_term.Sort level -> Ext_term.Sort (subst_level params levels level)
  | Ext_term.BVar _ -> term
  | Ext_term.Const (global_ref, term_levels) ->
      Ext_term.Const (global_ref, List.map (subst_level params levels) term_levels)
  | Ext_term.App (fn, arg) ->
      Ext_term.App (subst_levels_term params levels fn, subst_levels_term params levels arg)
  | Ext_term.Lam (ty, body) ->
      Ext_term.Lam (subst_levels_term params levels ty, subst_levels_term params levels body)
  | Ext_term.Pi (ty, body) ->
      Ext_term.Pi (subst_levels_term params levels ty, subst_levels_term params levels body)
  | Ext_term.Let (ty, value, body) ->
      Ext_term.Let
        ( subst_levels_term params levels ty,
          subst_levels_term params levels value,
          subst_levels_term params levels body )

let rec shift_at section offset term amount cutoff =
  match term with
  | Ext_term.Sort _ | Ext_term.Const _ -> Ok term
  | Ext_term.BVar index ->
      if index < 0 then error section offset Invalid_bvar
      else if index < cutoff then Ok term
      else
        let shifted = index + amount in
        if shifted < 0 then error section offset Invalid_bvar
        else Ok (Ext_term.BVar shifted)
  | Ext_term.App (fn, arg) ->
      bind (shift_at section offset fn amount cutoff) (fun shifted_fn ->
          bind (shift_at section offset arg amount cutoff) (fun shifted_arg ->
              Ok (Ext_term.App (shifted_fn, shifted_arg))))
  | Ext_term.Lam (ty, body) ->
      bind (shift_at section offset ty amount cutoff) (fun shifted_ty ->
          bind (shift_at section offset body amount (cutoff + 1)) (fun shifted_body ->
              Ok (Ext_term.Lam (shifted_ty, shifted_body))))
  | Ext_term.Pi (ty, body) ->
      bind (shift_at section offset ty amount cutoff) (fun shifted_ty ->
          bind (shift_at section offset body amount (cutoff + 1)) (fun shifted_body ->
              Ok (Ext_term.Pi (shifted_ty, shifted_body))))
  | Ext_term.Let (ty, value, body) ->
      bind (shift_at section offset ty amount cutoff) (fun shifted_ty ->
          bind (shift_at section offset value amount cutoff) (fun shifted_value ->
              bind (shift_at section offset body amount (cutoff + 1)) (fun shifted_body ->
                  Ok (Ext_term.Let (shifted_ty, shifted_value, shifted_body)))))

let shift section offset term amount cutoff =
  if cutoff < 0 then error section offset Invalid_bvar
  else shift_at section offset term amount cutoff

let rec substitute_at section offset term target replacement =
  match term with
  | Ext_term.Sort _ | Ext_term.Const _ -> Ok term
  | Ext_term.BVar index ->
      if index < 0 then error section offset Invalid_bvar
      else if index = target then shift section offset replacement target 0
      else if index > target then Ok (Ext_term.BVar (index - 1))
      else Ok term
  | Ext_term.App (fn, arg) ->
      bind (substitute_at section offset fn target replacement) (fun substituted_fn ->
          bind (substitute_at section offset arg target replacement) (fun substituted_arg ->
              Ok (Ext_term.App (substituted_fn, substituted_arg))))
  | Ext_term.Lam (ty, body) ->
      bind (substitute_at section offset ty target replacement) (fun substituted_ty ->
          bind (substitute_at section offset body (target + 1) replacement)
            (fun substituted_body -> Ok (Ext_term.Lam (substituted_ty, substituted_body))))
  | Ext_term.Pi (ty, body) ->
      bind (substitute_at section offset ty target replacement) (fun substituted_ty ->
          bind (substitute_at section offset body (target + 1) replacement)
            (fun substituted_body -> Ok (Ext_term.Pi (substituted_ty, substituted_body))))
  | Ext_term.Let (ty, value, body) ->
      bind (substitute_at section offset ty target replacement) (fun substituted_ty ->
          bind (substitute_at section offset value target replacement) (fun substituted_value ->
              bind (substitute_at section offset body (target + 1) replacement)
                (fun substituted_body ->
                  Ok (Ext_term.Let (substituted_ty, substituted_value, substituted_body)))))

let substitute section offset term target replacement =
  if target < 0 then error section offset Invalid_bvar
  else substitute_at section offset term target replacement

let instantiate section offset body value = substitute section offset body 0 value

let lookup_binding section offset context index =
  match list_nth_opt index context with
  | Some binding -> Ok binding
  | None -> error section offset Invalid_bvar

let lookup_type section offset context index =
  bind (lookup_binding section offset context index) (fun binding ->
      shift section offset binding.local_ty (index + 1) 0)

let lookup_value section offset context index =
  bind (lookup_binding section offset context index) (fun binding ->
      match binding.local_value with
      | None -> Ok None
      | Some value ->
          bind (shift section offset value (index + 1) 0) (fun shifted ->
              Ok (Some shifted)))

let levels_equal lhs rhs =
  List.length lhs = List.length rhs
  && List.for_all2
       (fun left right -> Ext_level.normalize left = Ext_level.normalize right)
       lhs rhs

let global_ref_equal left right =
  match (left, right) with
  | ( Ext_term.Imported
        {
          import_index = left_import;
          name = left_name;
          decl_interface_hash = left_hash;
        },
      Ext_term.Imported
        {
          import_index = right_import;
          name = right_name;
          decl_interface_hash = right_hash;
        } ) ->
      left_import = right_import && Ext_name.equal left_name right_name && left_hash = right_hash
  | Ext_term.Local { decl_index = left_index }, Ext_term.Local { decl_index = right_index } ->
      left_index = right_index
  | ( Ext_term.LocalGenerated { decl_index = left_index; name = left_name },
      Ext_term.LocalGenerated { decl_index = right_index; name = right_name } ) ->
      left_index = right_index && Ext_name.equal left_name right_name
  | ( Ext_term.Builtin { name = left_name; decl_interface_hash = left_hash },
      Ext_term.Builtin { name = right_name; decl_interface_hash = right_hash } ) ->
      Ext_name.equal left_name right_name && left_hash = right_hash
  | _ -> false

let rec whnf_with_fuel env context section offset delta term fuel =
  bind (spend_fuel section offset fuel) (fun () ->
      match term with
      | Ext_term.BVar index ->
          bind (lookup_value section offset context index) (function
            | None -> Ok term
            | Some value -> whnf_with_fuel env context section offset delta value fuel)
      | Ext_term.Const (global_ref, levels) ->
          bind (resolve_signature section offset env global_ref) (fun signature ->
              if
                List.length signature.Ext_env.signature_universe_params
                <> List.length levels
              then error section offset Bad_universe_arity
              else
                match signature.Ext_env.signature_unfolding with
                | Ext_env.Reducible value ->
                    let value =
                      subst_levels_term signature.Ext_env.signature_universe_params levels value
                    in
                    whnf_with_fuel env context section offset delta value fuel
                | Ext_env.No_unfolding | Ext_env.Opaque -> Ok term)
      | Ext_term.App (fn, arg) ->
          bind (whnf_with_fuel env context section offset delta fn fuel) (function
            | Ext_term.Lam (_, body) ->
                bind (instantiate section offset body arg) (fun instantiated ->
                    whnf_with_fuel env context section offset delta instantiated fuel)
            | whnf_fn -> Ok (Ext_term.App (whnf_fn, arg)))
      | Ext_term.Let (_, value, body) ->
          bind (instantiate section offset body value) (fun instantiated ->
              whnf_with_fuel env context section offset delta instantiated fuel)
      | Ext_term.Sort _ | Ext_term.Lam _ | Ext_term.Pi _ -> Ok term)

let whnf ?(section = Ext_bytes.Declarations) ?(offset = 0) ?(delta = []) env context term =
  let fuel = ref max_fuel in
  whnf_with_fuel env context section offset delta term fuel

let rec is_defeq_with_fuel env context section offset delta lhs rhs fuel =
  bind (spend_fuel section offset fuel) (fun () ->
      bind (whnf_with_fuel env context section offset delta lhs fuel) (fun lhs_whnf ->
          bind (whnf_with_fuel env context section offset delta rhs fuel) (fun rhs_whnf ->
              match (lhs_whnf, rhs_whnf) with
              | Ext_term.Sort lhs_level, Ext_term.Sort rhs_level ->
                  Ok (Ext_level.normalize lhs_level = Ext_level.normalize rhs_level)
              | Ext_term.BVar lhs_index, Ext_term.BVar rhs_index -> Ok (lhs_index = rhs_index)
              | ( Ext_term.Const (lhs_ref, lhs_levels),
                  Ext_term.Const (rhs_ref, rhs_levels) ) ->
                  Ok (levels_equal lhs_levels rhs_levels && global_ref_equal lhs_ref rhs_ref)
              | Ext_term.App (lhs_fn, lhs_arg), Ext_term.App (rhs_fn, rhs_arg) ->
                  bind
                    (is_defeq_with_fuel env context section offset delta lhs_fn rhs_fn fuel)
                    (fun fn_equal ->
                      if not fn_equal then Ok false
                      else
                        is_defeq_with_fuel env context section offset delta lhs_arg rhs_arg
                          fuel)
              | Ext_term.Pi (lhs_ty, lhs_body), Ext_term.Pi (rhs_ty, rhs_body) ->
                  bind
                    (is_defeq_with_fuel env context section offset delta lhs_ty rhs_ty fuel)
                    (fun ty_equal ->
                      if not ty_equal then Ok false
                      else
                        let body_context = push_assumption context lhs_ty in
                        is_defeq_with_fuel env body_context section offset delta lhs_body
                          rhs_body fuel)
              | Ext_term.Lam (lhs_ty, lhs_body), Ext_term.Lam (rhs_ty, rhs_body) ->
                  bind
                    (is_defeq_with_fuel env context section offset delta lhs_ty rhs_ty fuel)
                    (fun ty_equal ->
                      if not ty_equal then Ok false
                      else
                        let body_context = push_assumption context lhs_ty in
                        is_defeq_with_fuel env body_context section offset delta lhs_body
                          rhs_body fuel)
              | _ -> Ok false)))

let is_defeq ?(section = Ext_bytes.Declarations) ?(offset = 0) ?(delta = []) env context lhs
    rhs =
  let fuel = ref max_fuel in
  is_defeq_with_fuel env context section offset delta lhs rhs fuel

let rec infer ?(section = Ext_bytes.Declarations) ?(offset = 0) ?(delta = []) env context
    term =
  match term with
  | Ext_term.Sort level ->
      bind (ensure_level_wf section offset delta level) (fun () ->
          Ok (Ext_term.Sort (Ext_level.Succ level)))
  | Ext_term.BVar index -> lookup_type section offset context index
  | Ext_term.Const (global_ref, levels) ->
      let rec ensure_levels remaining =
        match remaining with
        | [] -> Ok ()
        | level :: rest ->
            bind (ensure_level_wf section offset delta level) (fun () ->
                ensure_levels rest)
      in
      bind (ensure_levels levels) (fun () ->
          bind (resolve_signature section offset env global_ref) (fun signature ->
              if
                List.length signature.Ext_env.signature_universe_params
                <> List.length levels
              then error section offset Bad_universe_arity
              else
                Ok
                  (subst_levels_term signature.Ext_env.signature_universe_params levels
                     signature.Ext_env.signature_ty)))
  | Ext_term.Pi (ty, body) ->
      bind (expect_sort ~section ~offset ~delta env context ty) (fun domain_sort ->
          let body_context = push_assumption context ty in
          bind
            (expect_sort ~section ~offset ~delta env body_context body)
            (fun body_sort -> Ok (Ext_term.Sort (Ext_level.Imax (domain_sort, body_sort)))))
  | Ext_term.Lam (ty, body) ->
      bind (expect_sort ~section ~offset ~delta env context ty) (fun _ ->
          let body_context = push_assumption context ty in
          bind (infer ~section ~offset ~delta env body_context body) (fun body_ty ->
              Ok (Ext_term.Pi (ty, body_ty))))
  | Ext_term.App (fn, arg) ->
      bind (infer ~section ~offset ~delta env context fn) (fun fn_ty ->
          bind (whnf ~section ~offset ~delta env context fn_ty) (function
            | Ext_term.Pi (domain_ty, body_ty) ->
                bind (check ~section ~offset ~delta env context arg domain_ty) (fun () ->
                    instantiate section offset body_ty arg)
            | _ -> error section offset Expected_function))
  | Ext_term.Let (ty, value, body) ->
      bind (expect_sort ~section ~offset ~delta env context ty) (fun _ ->
          bind (check ~section ~offset ~delta env context value ty) (fun () ->
              let body_context = push_definition context ty value in
              bind (infer ~section ~offset ~delta env body_context body) (fun body_ty ->
                  instantiate section offset body_ty value)))

and check ?(section = Ext_bytes.Declarations) ?(offset = 0) ?(delta = []) env context term
    expected =
  match term with
  | Ext_term.Lam (ty, body) ->
      bind (whnf ~section ~offset ~delta env context expected) (function
        | Ext_term.Pi (expected_ty, expected_body) ->
            bind (expect_sort ~section ~offset ~delta env context ty) (fun _ ->
                bind (is_defeq ~section ~offset ~delta env context ty expected_ty)
                  (fun domain_equal ->
                    if not domain_equal then error section offset Type_mismatch
                    else
                      let body_context = push_assumption context ty in
                      check ~section ~offset ~delta env body_context body expected_body))
        | _ -> error section offset Type_mismatch)
  | _ ->
      bind (infer ~section ~offset ~delta env context term) (fun actual ->
          bind (is_defeq ~section ~offset ~delta env context actual expected) (fun equal ->
              if equal then Ok () else error section offset Type_mismatch))

and expect_sort ?(section = Ext_bytes.Declarations) ?(offset = 0) ?(delta = []) env context
    term =
  bind (infer ~section ~offset ~delta env context term) (fun ty ->
      bind (whnf ~section ~offset ~delta env context ty) (function
        | Ext_term.Sort level -> Ok level
        | _ -> error section offset Expected_sort))

let rec ensure_delta_wf section offset params =
  match params with
  | [] -> Ok ()
  | name :: rest ->
      if Ext_level.component_contains_universe_meta name then
        error section offset Unresolved_metavariable
      else ensure_delta_wf section offset rest

let declaration_universe_constraints payload =
  match payload with
  | Ext_cert.AxiomDecl { decl_universe_constraints; _ }
  | Ext_cert.DefDecl { decl_universe_constraints; _ }
  | Ext_cert.TheoremDecl { decl_universe_constraints; _ }
  | Ext_cert.InductiveDecl { decl_universe_constraints; _ }
  | Ext_cert.MutualInductiveBlockDecl { decl_universe_constraints; _ } ->
      decl_universe_constraints

let rec ensure_constraints_wf section offset delta constraints =
  match constraints with
  | [] -> Ok ()
  | constraint_ :: rest ->
      bind
        (ensure_level_wf section offset delta constraint_.Ext_cert.constraint_lhs)
        (fun () ->
          bind
            (ensure_level_wf section offset delta
               constraint_.Ext_cert.constraint_rhs)
            (fun () -> ensure_constraints_wf section offset delta rest))

let check_dependency section offset env (dependency : Ext_cert.dependency_entry) =
  bind
    (resolve_signature section offset env dependency.Ext_cert.dependency_global_ref)
    (fun signature ->
      match signature.Ext_env.signature_decl_interface_hash with
      | Some hash when hash = dependency.Ext_cert.dependency_decl_interface_hash -> Ok ()
      | _ -> error section offset Type_mismatch)

let rec check_dependencies section offset env dependencies =
  match dependencies with
  | [] -> Ok ()
  | dependency :: rest ->
      bind (check_dependency section offset env dependency) (fun () ->
          check_dependencies section offset env rest)

let add_checked_declaration env declaration =
  match Ext_env.add_checked_declaration env declaration with
  | Ok env -> Ok env
  | Error env_error -> error_of_env_error env_error

let check_declaration env (declaration : Ext_cert.declaration) =
  let section = Ext_bytes.Declarations in
  let offset = declaration.Ext_cert.offset in
  bind
    (check_dependencies section offset env declaration.Ext_cert.dependencies)
    (fun () ->
      let delta = Ext_env.declaration_universe_params declaration.Ext_cert.payload in
      bind (ensure_delta_wf section offset delta) (fun () ->
          bind
            (ensure_constraints_wf section offset delta
               (declaration_universe_constraints declaration.Ext_cert.payload))
            (fun () ->
              match declaration.Ext_cert.payload with
              | Ext_cert.AxiomDecl { decl_ty; _ } ->
                  bind
                    (expect_sort ~section ~offset ~delta env empty_context decl_ty)
                    (fun _ -> add_checked_declaration env declaration)
              | Ext_cert.DefDecl { decl_ty; decl_value; _ } ->
                  bind
                    (expect_sort ~section ~offset ~delta env empty_context decl_ty)
                    (fun _ ->
                      bind
                        (check ~section ~offset ~delta env empty_context decl_value
                           decl_ty)
                        (fun () -> add_checked_declaration env declaration))
              | Ext_cert.TheoremDecl { decl_ty; decl_proof; _ } ->
                  bind
                    (expect_sort ~section ~offset ~delta env empty_context decl_ty)
                    (fun _ ->
                      bind
                        (check ~section ~offset ~delta env empty_context decl_proof
                           decl_ty)
                        (fun () -> add_checked_declaration env declaration))
              | Ext_cert.InductiveDecl _ | Ext_cert.MutualInductiveBlockDecl _ ->
                  error section offset Unsupported_declaration)))

let check_declarations ?(env = Ext_env.empty) declarations =
  let rec loop current_env remaining =
    match remaining with
    | [] -> Ok current_env
    | declaration :: rest ->
        bind (check_declaration current_env declaration) (fun next_env ->
            loop next_env rest)
  in
  loop env declarations

let check_certificate _env _certificate = Type_check_not_implemented
