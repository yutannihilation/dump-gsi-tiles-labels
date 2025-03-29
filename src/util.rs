use nom::error::ErrorKind;

pub(crate) fn nom_error<T>(input: &[u8]) -> Result<(&[u8], T), nom::Err<nom::error::Error<&[u8]>>> {
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        ErrorKind::Fail,
    )))
}
