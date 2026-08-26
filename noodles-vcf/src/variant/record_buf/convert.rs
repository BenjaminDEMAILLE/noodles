use std::io;

use super::{RecordBuf, Samples};
use crate::{Header, variant::Record};

impl RecordBuf {
    /// Converts a variant record to a buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{self as vcf, variant::RecordBuf};
    ///
    /// let header = vcf::Header::default();
    /// let record = vcf::Record::default();
    ///
    /// let record_buf = RecordBuf::try_from_variant_record(&header, &record)?;
    ///
    /// assert_eq!(record_buf, RecordBuf::default());
    /// # Ok::<_, std::io::Error>(())
    /// ```
    pub fn try_from_variant_record<R>(header: &Header, record: &R) -> io::Result<Self>
    where
        R: Record + ?Sized,
    {
        let mut dst = RecordBuf::default();
        dst.try_clone_from_variant_record(header, record)?;
        Ok(dst)
    }

    /// Clones the given record into this record.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{self as vcf, variant::RecordBuf};
    ///
    /// let mut record_buf = RecordBuf::builder()
    ///     .set_reference_sequence_name("sq0")
    ///     .build();
    ///
    /// let header = vcf::Header::default();
    /// let record = vcf::Record::default();
    ///
    /// record_buf.try_clone_from_variant_record(&header, &record)?;
    ///
    /// assert_eq!(record_buf, RecordBuf::default());
    /// # Ok::<_, std::io::Error>(())
    /// ```
    pub fn try_clone_from_variant_record<R>(
        &mut self,
        header: &Header,
        record: &R,
    ) -> io::Result<()>
    where
        R: Record + ?Sized,
    {
        *self.reference_sequence_name_mut() = record.reference_sequence_name(header)?.into();
        *self.variant_start_mut() = record.variant_start().transpose()?;
        *self.ids_mut() = record.ids().iter().map(String::from).collect();

        let raw_reference_bases: Vec<_> =
            record.reference_bases().iter().collect::<io::Result<_>>()?;
        *self.reference_bases_mut() = String::from_utf8(raw_reference_bases)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let raw_alternate_bases: Vec<_> = record
            .alternate_bases()
            .iter()
            .map(|result| result.map(String::from))
            .collect::<io::Result<_>>()?;
        *self.alternate_bases_mut() = raw_alternate_bases.into();

        *self.quality_score_mut() = record.quality_score().transpose()?;

        *self.filters_mut() = record
            .filters()
            .iter(header)
            .map(|result| result.map(String::from))
            .collect::<io::Result<_>>()?;

        *self.info_mut() = record
            .info()
            .iter(header)
            .map(|result| {
                result.and_then(|(key, value)| {
                    let v = value.map(|v| v.try_into()).transpose()?;
                    Ok((String::from(key), v))
                })
            })
            .collect::<io::Result<_>>()?;

        let samples = record.samples()?;

        let keys = samples
            .column_names(header)
            .map(|result| result.map(String::from))
            .collect::<io::Result<_>>()?;

        let values = samples
            .iter()
            .map(|sample| {
                sample
                    .iter(header)
                    .map(|result| {
                        result.and_then(|(_, value)| value.map(|v| v.try_into()).transpose())
                    })
                    .collect()
            })
            .collect::<io::Result<_>>()?;

        *self.samples_mut() = Samples::new(keys, values);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use noodles_core::Position;

    use super::*;

    #[test]
    fn test_try_clone_from_variant_record() -> io::Result<()> {
        let header = Header::default();

        let record = RecordBuf::builder()
            .set_reference_sequence_name("sq0")
            .set_variant_start(Position::MIN)
            .set_reference_bases("A")
            .build();

        let mut dst = RecordBuf::default();
        dst.try_clone_from_variant_record(&header, &record)?;

        assert_eq!(dst, record);

        Ok(())
    }
}
