use std::io::{self, Write};

use crate::{binning_index::index::header::Format, io::writer::num::write_i32_le};

pub(super) fn write_format<W>(writer: &mut W, format: Format) -> io::Result<()>
where
    W: Write,
{
    let n = i32::from(format);
    write_i32_le(writer, n)
}
