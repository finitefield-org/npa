type offset = int

type certificate_section =
  | Header_format
  | Header_core_spec
  | Header_module
  | Imports
  | Name_table
  | Level_table
  | Term_table
  | Declarations
  | Export_block
  | Axiom_report
  | Hashes
  | Import_store
  | Full_certificate

type decode_error_reason =
  | Unexpected_eof
  | Noncanonical_uvar
  | Uvar_overflow
  | Length_overflow

type decode_error = {
  section : certificate_section;
  offset : offset;
  reason : decode_error_reason;
}

type reader = {
  data : string;
  offset : offset;
}

type 'a read_result = ('a * reader, decode_error) result

let empty = { data = ""; offset = 0 }

let of_string data = { data; offset = 0 }

let of_bytes bytes = { data = Bytes.to_string bytes; offset = 0 }

let offset reader = reader.offset

let length reader = String.length reader.data

let remaining reader = length reader - reader.offset

let section_name section =
  match section with
  | Header_format -> "header_format"
  | Header_core_spec -> "header_core_spec"
  | Header_module -> "header_module"
  | Imports -> "imports"
  | Name_table -> "name_table"
  | Level_table -> "level_table"
  | Term_table -> "term_table"
  | Declarations -> "declarations"
  | Export_block -> "export_block"
  | Axiom_report -> "axiom_report"
  | Hashes -> "hashes"
  | Import_store -> "import_store"
  | Full_certificate -> "full_certificate"

let reason_code reason =
  match reason with
  | Unexpected_eof -> "unexpected_eof"
  | Noncanonical_uvar -> "noncanonical_uvar"
  | Uvar_overflow -> "uvar_overflow"
  | Length_overflow -> "length_overflow"

let error section offset reason = Error { section; offset; reason }

let advance reader offset = { reader with offset }

let read_byte section reader =
  if reader.offset >= length reader then error section reader.offset Unexpected_eof
  else
    let byte = Char.code reader.data.[reader.offset] in
    Ok (byte, advance reader (reader.offset + 1))

let take section count reader =
  if count < 0 then error section reader.offset Length_overflow
  else if count > max_int - reader.offset then error section reader.offset Length_overflow
  else
    let finish = reader.offset + count in
    if finish > length reader then error section (length reader) Unexpected_eof
    else Ok (String.sub reader.data reader.offset count, advance reader finish)

let encode_uvar value =
  let buffer = Buffer.create 10 in
  let rec loop value =
    let payload = Int64.to_int (Int64.logand value 0x7fL) in
    let next = Int64.shift_right_logical value 7 in
    let byte = if next = 0L then payload else payload lor 0x80 in
    Buffer.add_char buffer (Char.chr byte);
    if next <> 0L then loop next
  in
  loop value;
  Buffer.contents buffer

let read_uvar section reader =
  let start = reader.offset in
  let rec loop current shift byte_index value =
    match read_byte section current with
    | Error err -> Error err
    | Ok (byte, next) ->
        let payload = byte land 0x7f in
        let continues = byte land 0x80 <> 0 in
        if byte_index = 9 && (continues || payload > 1) then
          error section current.offset Uvar_overflow
        else
          let chunk = Int64.shift_left (Int64.of_int payload) shift in
          let value = Int64.logor value chunk in
          if continues then loop next (shift + 7) (byte_index + 1) value
          else
            let consumed = String.sub reader.data start (next.offset - start) in
            if consumed <> encode_uvar value then error section current.offset Noncanonical_uvar
            else Ok (value, next)
  in
  loop reader 0 0 0L

let read_usize section reader =
  let start = reader.offset in
  match read_uvar section reader with
  | Error err -> Error err
  | Ok (value, next) ->
      if value < 0L || value > Int64.of_int max_int then
        error section (max start (next.offset - 1)) Length_overflow
      else Ok (Int64.to_int value, next)
