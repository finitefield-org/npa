type t = {
  imports : Ext_import_store.import_environment;
  declarations : Ext_cert.declaration list;
}

let empty = { imports = Ext_import_store.import_environment_empty; declarations = [] }
