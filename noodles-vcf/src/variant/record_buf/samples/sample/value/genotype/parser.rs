use std::{error, fmt};

use memchr::memchr2;

use super::{Allele, Genotype, allele};
use crate::variant::record::samples::series::value::genotype::Phasing;

/// An error returned when a raw VCF record genotype value fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// An allele is invalid.
    InvalidAllele(allele::ParseError),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Empty => None,
            Self::InvalidAllele(e) => Some(e),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty input"),
            Self::InvalidAllele(_) => f.write_str("invalid allele"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FirstPhasing {
    Explicit(Phasing),
    Implicit(Phasing),
}

impl FirstPhasing {
    fn phasing(&self) -> Phasing {
        match self {
            FirstPhasing::Explicit(phasing) => *phasing,
            FirstPhasing::Implicit(phasing) => *phasing,
        }
    }
}

pub(super) fn parse(mut s: &str) -> Result<Genotype, ParseError> {
    if s.is_empty() {
        return Err(ParseError::Empty);
    }

    let raw_allele = next_allele(&mut s);
    let first_allele = parse_first_allele(raw_allele).map_err(ParseError::InvalidAllele)?;
    let (first_position, mut first_phasing) = match first_allele {
        (position, Some(phasing)) => (position, FirstPhasing::Explicit(phasing)),
        (position, None) => (position, FirstPhasing::Implicit(Phasing::Phased)),
    };

    let mut alleles = vec![Allele::new(first_position, first_phasing.phasing())];

    while !s.is_empty() {
        let raw_allele = next_allele(&mut s);
        let allele: Allele = raw_allele.parse().map_err(ParseError::InvalidAllele)?;

        if first_phasing == FirstPhasing::Implicit(Phasing::Phased)
            && allele.phasing() == Phasing::Unphased
        {
            first_phasing = FirstPhasing::Implicit(Phasing::Unphased);
        }

        alleles.push(allele);
    }

    *alleles[0].phasing_mut() = first_phasing.phasing();

    Ok(Genotype(alleles))
}

fn next_allele<'a>(s: &mut &'a str) -> &'a str {
    let src = &s.as_bytes()[1..];
    let i = memchr2(b'|', b'/', src).map(|i| i + 1).unwrap_or(s.len());
    let (buf, rest) = s.split_at(i);
    *s = rest;
    buf
}

fn parse_first_allele(s: &str) -> Result<(Option<usize>, Option<Phasing>), allele::ParseError> {
    use super::allele::{parse_phasing, parse_position};

    match parse_phasing(&s[..1]) {
        Ok(phasing) => {
            let position = parse_position(&s[1..])?;
            Ok((position, Some(phasing)))
        }
        Err(_) => {
            if let Ok(position) = parse_position(s) {
                Ok((position, None))
            } else {
                Err(allele::ParseError::InvalidPhasing)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_allele() {
        let mut src = "0";
        assert_eq!(next_allele(&mut src), "0");
        assert!(src.is_empty());

        let mut src = "|0/0";
        assert_eq!(next_allele(&mut src), "|0");
        assert_eq!(next_allele(&mut src), "/0");
        assert!(src.is_empty());
    }
}
