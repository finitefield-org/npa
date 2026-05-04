pub type ByteOffset = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub file_id: FileId,
    pub start: ByteOffset,
    pub end: ByteOffset,
}

impl Span {
    pub fn new(file_id: FileId, start: ByteOffset, end: ByteOffset) -> Self {
        Self {
            file_id,
            start,
            end,
        }
    }

    pub fn join(self, other: Self) -> Self {
        debug_assert_eq!(self.file_id, other.file_id);
        Self {
            file_id: self.file_id,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}
