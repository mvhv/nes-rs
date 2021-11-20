use crate::cpu::Mnemonic;

fn parse_mnemonic(input: &str) -> IResult<(&str, Mnemonic), nom::Err<(&str, nom::error::ErrorKind)>> {
    take(3)(input).try_into()
}