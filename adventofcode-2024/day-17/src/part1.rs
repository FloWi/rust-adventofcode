use itertools::Itertools;
use miette::miette;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{anychar, char, multispace1};
use nom::combinator::all_consuming;
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair, tuple};
use nom::IResult;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::ops::BitXor;
use tracing::{debug, info};

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, mut computer) = parse(input.trim()).map_err(|e| miette!("parse failed {}", e))?;

    info!("Input: \n{input}\n\nInitial_state: \n{computer:?}");

    computer.run();
    let output = computer
        .output
        .into_iter()
        .map(|num| num.to_string())
        .collect_vec()
        .join(",");

    info!("Output: {}", output);
    Ok(output)
}

#[derive(TryFromPrimitive, IntoPrimitive, Debug, Clone, Copy)]
#[repr(u32)] // or u32, i32, etc. depending on your needs
enum Instruction {
    ///The adv instruction (opcode 0) performs division. The numerator is the value in the A register.
    /// The denominator is found by raising 2 to the power of the instruction's combo operand.
    /// (So, an operand of 2 would divide A by 4 (2^2); an operand of 5 would divide A by 2^B.)
    /// The result of the division operation is truncated to an integer and then written to the A register.
    Adv = 0,

    ///The bxl instruction (opcode 1) calculates the bitwise XOR of register B
    /// and the instruction's literal operand, then stores the result in register B.
    Bxl = 1,

    ///The bst instruction (opcode 2) calculates the value of its combo operand modulo 8
    /// (thereby keeping only its lowest 3 bits), then writes that value to the B register.
    Bst = 2,

    ///The jnz instruction (opcode 3) does nothing if the A register is 0.
    /// However, if the A register is not zero,
    /// it jumps by setting the instruction pointer to the value of its literal operand;
    /// if this instruction jumps, the instruction pointer is not increased by 2 after this instruction.
    Jnz = 3,

    ///The bxc instruction (opcode 4) calculates the bitwise XOR of register B and register C,
    /// then stores the result in register B. (For legacy reasons,
    /// this instruction reads an operand but ignores it.)
    Bxc = 4,

    ///The out instruction (opcode 5) calculates the value of its combo operand modulo 8,
    /// then outputs that value.
    /// (If a program outputs multiple values, they are separated by commas.)
    Out = 5,

    ///The bdv instruction (opcode 6) works exactly like the adv instruction except that the result is stored in the B register.
    /// (The numerator is still read from the A register.)
    Bdv = 6,

    ///The cdv instruction (opcode 7) works exactly like the adv instruction except that the result is stored in the C register.
    /// (The numerator is still read from the A register.)
    Cdv = 7,
}

impl Instruction {
    pub fn operand_type(&self) -> OperandType {
        match self {
            Instruction::Adv => OperandType::Combo,
            Instruction::Bxl => OperandType::Literal,
            Instruction::Bst => OperandType::Combo,
            Instruction::Jnz => OperandType::Literal,
            Instruction::Bxc => OperandType::Ignored,
            Instruction::Out => OperandType::Combo,
            Instruction::Bdv => OperandType::Combo,
            Instruction::Cdv => OperandType::Combo,
        }
    }
}

fn parse(input: &str) -> IResult<&str, Computer> {
    // TODO - check back after watching Chris' video on how to do the parsing of exactly 3 registers in a typesafe manner
    let (rest, (registers, program)) = all_consuming(separated_pair(
        separated_list1(
            multispace1,
            preceded(tuple((tag("Register "), anychar, tag(": "))), complete::u32),
        ),
        multispace1,
        preceded(tag("Program: "), separated_list1(char(','), complete::u32)),
    ))(input)?;

    if registers.len() != 3 {
        panic!("wrong number of registers")
    }

    Ok((
        rest,
        Computer {
            register_a: registers[0],
            register_b: registers[1],
            register_c: registers[2],
            program: program.into_iter().collect_vec(),
            instruction_pointer: 0,
            output: vec![],
        },
    ))
}

#[derive(Default, Clone, Debug)]
struct Computer {
    register_a: u32,
    register_b: u32,
    register_c: u32,
    program: Vec<u32>,
    instruction_pointer: usize,
    output: Vec<u32>,
}

#[derive(Debug)]
enum OperandType {
    Combo,
    Literal,
    Ignored,
}

impl Computer {
    pub(crate) fn run(&mut self) {
        while self.instruction_pointer < self.program.len() {
            //info!("running program at idx {}", self.instruction_pointer);
            self.run_one();
        }
        info!("done. Final State: \n{self:?}");
    }
    #[tracing::instrument(skip(self), fields(instruction_pointer = self.instruction_pointer))]
    pub(crate) fn run_one(&mut self) {
        let instruction = self.current_instruction();
        let op_code: u32 = instruction.into();
        let operand = self.current_operand();

        let operand_type = instruction.operand_type();
        let resolved_operand = match operand_type {
            OperandType::Combo => match operand {
                num @ 0..=3 => Some(num),
                4 => Some(self.register_a),
                5 => Some(self.register_b),
                6 => Some(self.register_c),
                7.. => panic!("this should never happen"),
            },
            OperandType::Literal => Some(operand),
            OperandType::Ignored => None,
        };

        match instruction {
            Instruction::Adv => {
                let a = self.register_a;
                let numerator = a;
                let operand = resolved_operand.unwrap();
                let denominator = 2u32.pow(operand);

                let result = numerator / denominator;
                self.register_a = result;
                debug!("Performing Adv: a = a / 2^resolved_operand ==>  {a} / 2^{operand} = {numerator} / {denominator} = {result}");

                self.instruction_pointer += 2;
            }
            Instruction::Bxl => {
                let b = self.register_b;
                let operand = resolved_operand.unwrap();
                let result = b.bitxor(operand);

                self.register_b = result;
                debug!("Performing Bxl: b = b XOR operand --> {b} xor {operand} = {result}");

                self.instruction_pointer += 2;
            }
            Instruction::Bst => {
                let operand = resolved_operand.unwrap();
                let result = operand % 8;
                debug!("Performing Bst: b = operand % 8 = {operand} % 8 = {result}");
                self.register_b = result;
                self.instruction_pointer += 2;
            }
            Instruction::Jnz => {
                let a = self.register_a;
                if a == 0 {
                    debug!("Performing Jnz: a == 0 - no jump");
                    self.instruction_pointer += 2;
                } else {
                    let operand = resolved_operand.unwrap() as usize;
                    self.instruction_pointer = operand;
                    debug!("Performing Jnz: a != 0 ==> {a} != 0 ==> jumping to operand {operand}",);
                }
            }
            Instruction::Bxc => {
                let res = self.register_b.bitxor(self.register_c);
                debug!(
                    "Performing Bxc: b = b XOR c --> {} xor {} = {}",
                    self.register_b, self.register_c, res
                );
                self.register_b = res;
                self.instruction_pointer += 2;
            }
            Instruction::Out => {
                let resolved = resolved_operand.unwrap();
                let result = resolved % 8;
                self.output.push(result);
                debug!(
                    "Performing Out: (operand: {operand}; type: {operand_type:?}, resolved: {resolved}) ==> resolved % 8 = {resolved} % 8 = {result}",
                );
                self.instruction_pointer += 2;
            }
            Instruction::Bdv => {
                let a = self.register_a;
                let numerator = a;
                let operand = resolved_operand.unwrap();
                let denominator = 2u32.pow(operand);

                let result = numerator / denominator;
                self.register_b = result;
                debug!("Performing Bdv: b = a / 2^resolved_operand ==>  {a} / 2^{operand} = {numerator} / {denominator} = {result}");

                self.instruction_pointer += 2;
            }
            Instruction::Cdv => {
                let a = self.register_a;
                let numerator = a;
                let operand = resolved_operand.unwrap();
                let denominator = 2u32.pow(operand);

                let result = numerator / denominator;
                self.register_c = result;
                debug!("Performing Cdv: c = a / 2^resolved_operand ==>  {a} / 2^{operand} = {numerator} / {denominator} = {result}");

                self.instruction_pointer += 2;
            }
        }
    }

    pub(crate) fn current_instruction(&self) -> Instruction {
        Instruction::try_from(self.program[self.instruction_pointer]).unwrap()
    }
    pub(crate) fn current_operand(&self) -> u32 {
        self.program[self.instruction_pointer + 1]
    }

    // The computer knows eight instructions, each identified by a 3-bit number (called the instruction's opcode).
    // Each instruction also reads the 3-bit number after it as an input; this is called its operand.
    //
    // A number called the instruction pointer identifies the position in the program from which the next opcode will be read;
    // it starts at 0, pointing at the first 3-bit number in the program.
    // Except for jump instructions, the instruction pointer increases by 2 after each instruction is processed (to move past the instruction's opcode and its operand).
    // If the computer tries to read an opcode past the end of the program, it instead halts.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = r#"
Register A: 729
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0
        "#
        .trim();
        assert_eq!("4,6,3,5,6,3,5,2,1,0", process(input)?);
        Ok(())
    }

    #[test]
    fn test_example_1() -> miette::Result<()> {
        // If register C contains 9, the program 2,6 would set register B to 1.
        let input = r#"
Register A: 0
Register B: 0
Register C: 9

Program: 2,6
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();
        assert_eq!(computer.register_b, 1);
        Ok(())
    }
    #[test]
    fn test_example_2() -> miette::Result<()> {
        // If register A contains 10, the program 5,0,5,1,5,4 would output 0,1,2.
        let input = r#"
Register A: 10
Register B: 0
Register C: 0

Program: 5,0,5,1,5,4
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();

        assert_eq!(computer.output, vec![0, 1, 2]);
        Ok(())
    }

    #[test]
    fn test_example_3() -> miette::Result<()> {
        // If register A contains 2024, the program 0,1,5,4,3,0 would output 4,2,5,6,7,7,7,7,3,1,0 and leave 0 in register A.
        let input = r#"
Register A: 2024
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();

        assert_eq!(computer.output, vec![4, 2, 5, 6, 7, 7, 7, 7, 3, 1, 0]);
        assert_eq!(computer.register_a, 0);
        Ok(())
    }

    #[test]
    fn test_example_4() -> miette::Result<()> {
        // If register B contains 29, the program 1,7 would set register B to 26.
        let input = r#"
Register A: 0
Register B: 29
Register C: 0

Program: 1,7
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();

        assert_eq!(computer.register_b, 26);
        Ok(())
    }

    #[test]
    fn test_example_5() -> miette::Result<()> {
        // If register B contains 2024 and register C contains 43690, the program 4,0 would set register B to 44354.
        let input = r#"
Register A: 0
Register B: 2024
Register C: 43690

Program: 4,0
        "#
        .trim();

        let mut computer = parse(input).unwrap().1;
        computer.run();

        assert_eq!(computer.register_b, 44354);
        Ok(())
    }
}
