#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    todo!("day 01 - part 1");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        todo!("haven't built test yet");
        let input = r#"
input_goes_here_and_will_be_trimmed
        "#
        .trim();
        assert_eq!("", process(input)?);
        Ok(())
    }
}
