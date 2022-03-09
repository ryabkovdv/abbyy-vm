use std::error;
use std::fmt;
use std::marker::PhantomData;

use smallvec::SmallVec;

const FILE_HEADER_SIZE: usize = 4 * 4;
const SEGMENT_HEADER_SIZE: usize = 3 * 4;

const FILE_MAGIC: [u8; 4] = [0x80, b'B', b'I', b'N'];

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
pub enum Error {
    InvalidFormat,
    UnsupportedVersion(u32),
    FileTooShort,
    FileTooLarge,
    InvalidOffsetRange { offset: u32, size: u32 },
    InvalidAddrRange { addr: u32, size: u32 },
}

pub struct File<'a> {
    data: &'a [u8],
    memory_size: u32,
    segment_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Segment<'a> {
    pub addr: u32,
    pub data: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub struct RawSegment {
    pub offset: u32,
    pub addr: u32,
    pub size: u32,
}

unsafe fn load_u32(ptr: *const u8, index: usize) -> u32 {
    u32::from_le_bytes(*(ptr.add(index * 4) as *const [u8; 4]))
}

impl<'a> File<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<File<'a>> {
        if data.len() < FILE_HEADER_SIZE {
            return Err(Error::InvalidFormat);
        }

        let magic;
        let version;
        let memory_size;
        let segment_count;
        unsafe {
            magic         = *(data.as_ptr() as *const [u8; 4]);
            version       = load_u32(data.as_ptr(), 1);
            memory_size   = load_u32(data.as_ptr(), 2);
            segment_count = load_u32(data.as_ptr(), 3);
        }

        if magic != FILE_MAGIC {
            return Err(Error::InvalidFormat);
        }

        if version != 1 {
            return Err(Error::UnsupportedVersion(version));
        }

        if let Some(size) = (segment_count as usize).checked_mul(SEGMENT_HEADER_SIZE) {
            if data.len() - FILE_HEADER_SIZE >= size {
                return Ok(File { data, memory_size, segment_count });
            }
        }
        Err(Error::FileTooShort)
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn memory_size(&self) -> u32 {
        self.memory_size
    }

    pub fn segment_count(&self) -> u32 {
        self.segment_count
    }

    pub fn segments(&self) -> SegmentIterator<'a> {
        SegmentIterator {
            cursor: unsafe { self.data.as_ptr().add(FILE_HEADER_SIZE) },
            remaining: self.segment_count as usize,
            data: self.data,
        }
    }

    pub fn raw_segments(&self) -> RawSegmentIterator<'a> {
        RawSegmentIterator {
            cursor: unsafe { self.data.as_ptr().add(FILE_HEADER_SIZE) },
            remaining: self.segment_count as usize,
            _data: PhantomData,
        }
    }

    pub fn to_memory(&self) -> Result<Vec<u8>> {
        let memory_size = self.memory_size as usize;
        let mut memory = vec![0_u8; memory_size];
        let memory_ptr = memory.as_mut_ptr();

        for segment in self.segments() {
            let segment = match segment {
                Ok(segment) => segment,
                Err(segment) => {
                    return Err(Error::InvalidOffsetRange {
                        offset: segment.offset,
                        size: segment.size,
                    });
                }
            };

            let addr = segment.addr as usize;
            let size = segment.data.len();
            let data_ptr = segment.data.as_ptr();

            if !addr.checked_add(size).map_or(false, |upper| upper <= memory_size) {
                return Err(Error::InvalidAddrRange {
                    addr: addr as u32,
                    size: size as u32,
                });
            }

            unsafe {
                memory_ptr.add(addr).copy_from_nonoverlapping(data_ptr, size);
            }
        }

        Ok(memory)
    }
}

#[derive(Clone, Copy)]
pub struct SegmentIterator<'a> {
    cursor: *const u8,
    remaining: usize,
    data: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct RawSegmentIterator<'a> {
    cursor: *const u8,
    remaining: usize,
    _data: PhantomData<&'a [u8]>,
}

impl<'a> Iterator for SegmentIterator<'a> {
    type Item = Result<Segment<'a>, RawSegment>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        unsafe {
            let offset = load_u32(self.cursor, 0);
            let addr   = load_u32(self.cursor, 1);
            let size   = load_u32(self.cursor, 2);

            self.cursor = self.cursor.add(SEGMENT_HEADER_SIZE);
            self.remaining -= 1;

            let range = (offset as usize)..(offset as usize).wrapping_add(size as usize);
            if let Some(data) = self.data.get(range) {
                Some(Ok(Segment { addr, data }))
            } else {
                Some(Err(RawSegment { offset, addr, size }))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a> ExactSizeIterator for SegmentIterator<'a> {}

impl<'a> Iterator for RawSegmentIterator<'a> {
    type Item = RawSegment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        unsafe {
            let offset = load_u32(self.cursor, 0);
            let addr   = load_u32(self.cursor, 1);
            let size   = load_u32(self.cursor, 2);

            self.cursor = self.cursor.add(SEGMENT_HEADER_SIZE);
            self.remaining -= 1;

            Some(RawSegment { offset, addr, size })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a> ExactSizeIterator for RawSegmentIterator<'a> {}

pub fn to_memory(data: &[u8]) -> Result<Vec<u8>> {
    File::from_bytes(data)?.to_memory()
}

pub fn serialize<'a, Iter>(memory_size: u32, segments: Iter, buffer: &mut Vec<u8>) -> Result<()>
where
    Iter: IntoIterator,
    Iter::Item: Into<Segment<'a>>,
{
    macro_rules! try_u32 {
        ($e:expr) => {
            u32::try_from($e).map_err(|_| Error::FileTooLarge)
        };
    }

    let mut data = Vec::new();
    let mut raw_segments: SmallVec<[RawSegment; 16]> = SmallVec::new();
    for segment in segments {
        let segment = segment.into();

        let offset = try_u32!(data.len())?;
        let size = try_u32!(segment.data.len())?;

        raw_segments.push(RawSegment { offset, addr: segment.addr, size });
        data.extend_from_slice(segment.data);
    }

    let segment_count = try_u32!(raw_segments.len())?;

    let offset_to_data = match (segment_count as usize).checked_mul(SEGMENT_HEADER_SIZE) {
        Some(value) => match value.checked_add(FILE_HEADER_SIZE) {
            Some(offset) => try_u32!(offset)?,
            None => return Err(Error::FileTooLarge),
        },
        None => return Err(Error::FileTooLarge),
    };

    if try_u32!(data.len())?.checked_add(offset_to_data).is_none() {
        return Err(Error::FileTooLarge);
    }

    {
        let mut header = [0_u8; FILE_HEADER_SIZE];
        header[0_..4_].copy_from_slice(&FILE_MAGIC);
        header[4_..8_].copy_from_slice(&u32::to_le_bytes(1));
        header[8_..12].copy_from_slice(&u32::to_le_bytes(memory_size));
        header[12..16].copy_from_slice(&u32::to_le_bytes(segment_count));
        buffer.extend_from_slice(&header);
    }

    for segment in raw_segments {
        let mut header = [0_u8; SEGMENT_HEADER_SIZE];
        header[0_..4_].copy_from_slice(&u32::to_le_bytes(segment.offset + offset_to_data));
        header[4_..8_].copy_from_slice(&u32::to_le_bytes(segment.addr));
        header[8_..12].copy_from_slice(&u32::to_le_bytes(segment.size));
        buffer.extend_from_slice(&header);
    }

    buffer.extend_from_slice(&data);

    Ok(())
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            InvalidFormat => f.write_str("invalid format"),
            FileTooShort => f.write_str("file is too short"),
            FileTooLarge => f.write_str("file is too large"),
            UnsupportedVersion(version) => {
                write!(f, "unsupported version {}", version)
            }
            InvalidOffsetRange { offset, size } => {
                write!(
                    f,
                    "segment does not fit in file (offset: 0x{:X}, size: 0x{:X})",
                    offset, size,
                )
            }
            InvalidAddrRange { addr, size } => {
                write!(
                    f,
                    "segment does not fit in memory (address: 0x{:X}, size: 0x{:X})",
                    addr, size,
                )
            }
        }
    }
}

impl error::Error for Error {}
