type offset = int

type reader = {
  bytes : bytes;
  offset : offset;
}

let empty = { bytes = Bytes.empty; offset = 0 }
