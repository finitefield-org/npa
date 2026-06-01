type t = {
  imports : Ext_import.store;
  declarations : Ext_cert.declaration list;
}

let empty = { imports = Ext_import.empty; declarations = [] }
