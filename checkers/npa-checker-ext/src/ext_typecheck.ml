type check_result =
  | Type_checked
  | Type_check_not_implemented

let check_certificate _env _certificate = Type_check_not_implemented
