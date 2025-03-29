use nom::{IResult, bytes::complete::take, bytes::complete::take_while};

const MSB_MASK: u8 = 0b10000000;

// cf. https://protobuf.dev/programming-guides/encoding/#varints

pub(crate) fn parse_varint(input: &[u8]) -> IResult<&[u8], u64> {
    let (input, cont_bytes) = take_while(|x| x & MSB_MASK != 0)(input)?;
    let (input, last_byte) = take(1usize)(input)?;

    let mut i = cont_bytes.len();
    let mut result = (last_byte[0] as u64) << (7 * i);
    for b in cont_bytes {
        i -= 1;
        let shift = 7 * i;
        result += ((b ^ MSB_MASK) as u64) << shift;
    }

    Ok((input, result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_varint() {
        let b1 = [1u8];
        let res = parse_varint(&b1).unwrap();
        assert!(res.0.is_empty());
        assert_eq!(res.1, 1);

        let b2 = [0b10010110, 0b00000001];
        let res = parse_varint(&b2).unwrap();
        assert!(res.0.is_empty());
        assert_eq!(res.1, 150);
    }
}
