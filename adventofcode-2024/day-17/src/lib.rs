use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{anychar, char, multispace1};
use nom::combinator::all_consuming;
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair, tuple};
use nom::IResult;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::ops::BitXor;
use tracing::debug;

pub mod part1;
pub mod part2;

#[derive(TryFromPrimitive, IntoPrimitive, Debug, Clone, Copy)]
#[repr(u64)] // or u64, i32, etc. depending on your needs
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
            preceded(tuple((tag("Register "), anychar, tag(": "))), complete::u64),
        ),
        multispace1,
        preceded(tag("Program: "), separated_list1(char(','), complete::u64)),
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
    register_a: u64,
    register_b: u64,
    register_c: u64,
    program: Vec<u64>,
    instruction_pointer: usize,
    output: Vec<u64>,
}

impl Computer {
    pub(crate) fn reset(&mut self) {
        self.register_a = 0u64;
        self.register_b = 0u64;
        self.register_c = 0u64;
        self.instruction_pointer = 0;
        self.output = vec![];
    }
}

#[derive(Debug)]
enum OperandType {
    Combo,
    Literal,
    Ignored,
}

#[derive(Debug)]
enum ComboType {
    Literal,
    RegisterA,
    RegisterB,
    RegisterC,
    Unknown,
}

impl Computer {
    pub(crate) fn run(&mut self) {
        while self.instruction_pointer < self.program.len() {
            //debug!("running program at idx {}", self.instruction_pointer);
            self.run_one();
        }
        debug!("done. Final State: \n{self:?}");
    }
    pub(crate) fn run_one(&mut self) {
        let instruction = self.current_instruction();
        let op_code: u64 = instruction.into();
        let operand = self.current_operand();

        let instruction_pointer = self.instruction_pointer;

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

        let maybe_combo_type = match operand_type {
            OperandType::Combo => Some(match operand {
                0..=3 => ComboType::Literal,
                4 => ComboType::RegisterA,
                5 => ComboType::RegisterB,
                6 => ComboType::RegisterC,
                7.. => ComboType::Unknown,
            }),
            _ => None,
        };

        let describe_operand = || {
            let combo_type_string = maybe_combo_type
                .map(|ct| format!(" combo_type: {ct:?};"))
                .unwrap_or("".to_string());

            format!("op_code: {op_code}; operand: {operand}; type: {operand_type:?};{combo_type_string} resolved: {resolved_operand:?}")
        };

        match instruction {
            Instruction::Adv => {
                let a = self.register_a;
                let numerator = a;
                let resolved = resolved_operand.unwrap();
                let denominator = 2u64.pow(operand as u32);

                let result = numerator / denominator;
                self.register_a = result;
                debug!("idx: {instruction_pointer}; Instruction: Adv; {} |  a = a / 2^resolved_operand ==> {a} / 2^{operand} = {numerator} / {denominator} = {result}", describe_operand());

                self.instruction_pointer += 2;
            }
            Instruction::Bxl => {
                let b = self.register_b;
                let resolved = resolved_operand.unwrap();
                let result = b.bitxor(operand);

                self.register_b = result;
                debug!("idx: {instruction_pointer}; Instruction: Bxl; {} |  b = b XOR operand ==> {b} xor {operand} = {result}", describe_operand());

                self.instruction_pointer += 2;
            }
            Instruction::Bst => {
                let resolved = resolved_operand.unwrap();
                let result = resolved % 8;
                debug!("idx: {instruction_pointer}; Instruction: Bst; {} |  b = operand % 8 ==> {operand} % 8 = {result}", describe_operand());
                self.register_b = result;
                self.instruction_pointer += 2;
            }
            Instruction::Jnz => {
                let a = self.register_a;
                if a == 0 {
                    debug!("idx: {instruction_pointer}; Instruction: Jnz; op_code: {op_code}  |  a == 0 - no jump");
                    self.instruction_pointer += 2;
                } else {
                    let resolved = resolved_operand.unwrap();
                    let operand = resolved as usize;
                    self.instruction_pointer = operand;
                    debug!("idx: {instruction_pointer}; Instruction: Jnz; {} |  a != 0 ==> {a} != 0 ==> jumping to operand {operand}", describe_operand());
                }
            }
            Instruction::Bxc => {
                let res = self.register_b.bitxor(self.register_c);
                debug!(
                    "idx: {instruction_pointer}; Instruction: Bxc; op_code: {op_code}  |  b = b XOR c ==> {} xor {} = {}",
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
                    "idx: {instruction_pointer}; Instruction: Out; {} |  resolved % 8 = {resolved} % 8 = {result}", describe_operand()
                );
                self.instruction_pointer += 2;
            }
            Instruction::Bdv => {
                let a = self.register_a;
                let numerator = a;
                let resolved = resolved_operand.unwrap();
                let denominator = 2u64.pow(resolved as u32);

                let result = numerator / denominator;
                self.register_b = result;
                debug!("idx: {instruction_pointer}; Instruction: Bdv; {} |  b = a / 2^resolved ==> {a} / 2^{resolved} = {numerator} / {denominator} = {result}", describe_operand());

                self.instruction_pointer += 2;
            }
            Instruction::Cdv => {
                let a = self.register_a;
                let numerator = a;
                let resolved = resolved_operand.unwrap();
                let denominator = 2u64.pow(resolved as u32);

                let result = numerator / denominator;
                self.register_c = result;
                debug!("idx: {instruction_pointer}; Instruction: Cdv; {} |  c = a / 2^resolved_operand ==> {a} / 2^{resolved} = {numerator} / {denominator} = {result}", describe_operand());

                self.instruction_pointer += 2;
            }
        }
    }

    pub(crate) fn current_instruction(&self) -> Instruction {
        Instruction::try_from(self.program[self.instruction_pointer]).unwrap()
    }
    pub(crate) fn current_operand(&self) -> u64 {
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
