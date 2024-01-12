//! An implementation of an L-system.

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use ori::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub rule: String,
    pub replace: String,
}

impl Rule {
    pub fn new(rule: &str, replace: &str) -> Self {
        Self {
            rule: rule.to_string(),
            replace: replace.to_string(),
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        let mut parts = input.split("->");

        let rule = parts.next()?.trim();
        let replace = parts.next()?.trim();

        Some(Self::new(rule, replace))
    }
}

impl ToString for Rule {
    fn to_string(&self) -> String {
        format!("{} -> {}", self.rule, self.replace)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rules {
    rules: Vec<Rule>,
}

impl Rules {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    pub fn parse(input: &str) -> Self {
        let mut rules = Self::new();

        for line in input.lines() {
            if let Some(rule) = Rule::parse(line) {
                rules.push(&rule.rule, &rule.replace);
            }
        }

        rules
    }

    pub fn push(&mut self, rule: &str, replace: &str) {
        self.rules.push(Rule::new(rule, replace));
    }

    pub fn apply(&self, input: &str) -> String {
        let mut output = String::new();
        // the number of characters matched by the last rule
        // this is used to skip over the matched characters
        // omitting them from the output
        let mut skip = 0;

        let mut i = 0;
        while i < input.len() {
            let Some(c) = input[i..].chars().next() else {
                break;
            };

            if skip > 0 {
                skip -= c.len_utf8();
                i += c.len_utf8();
                continue;
            }

            let mut matched = false;

            for rule in &self.rules {
                if input[i..].starts_with(&rule.rule) {
                    output.push_str(&rule.replace);
                    skip = rule.rule.len() - c.len_utf8();
                    matched = true;
                    break;
                }
            }

            if !matched {
                output.push(c);
            }

            i += c.len_utf8();
        }

        output
    }
}

impl Deref for Rules {
    type Target = Vec<Rule>;

    fn deref(&self) -> &Self::Target {
        &self.rules
    }
}

impl DerefMut for Rules {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rules
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Forward(f32),
    Turn(f32),
    Scale(f32),
    Push,
    Pop,
}

impl Instruction {
    fn parse<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<Self> {
        match parts.next()? {
            "forward" => {
                let length = parts.next()?.parse().ok()?;
                Some(Self::Forward(length))
            }
            "turn" => {
                let angle = parts.next()?.parse().ok()?;
                Some(Self::Turn(angle))
            }
            "scale" => {
                let scale = parts.next()?.parse().ok()?;
                Some(Self::Scale(scale))
            }
            "push" => Some(Self::Push),
            "pop" => Some(Self::Pop),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instructions {
    instructions: HashMap<char, Instruction>,
}

impl Instructions {
    fn parse_instruction(input: &str) -> Option<(char, Instruction)> {
        let mut parts = input.split_whitespace();

        let key = parts.next()?.chars().next()?;

        if parts.next()? != "=" {
            return None;
        }

        let instruction = Instruction::parse(parts)?;

        Some((key, instruction))
    }

    pub fn parse(input: &str) -> Self {
        let mut instructions = Self::new();

        for line in input.lines() {
            if let Some((key, instruction)) = Self::parse_instruction(line) {
                instructions.insert(key, instruction);
            }
        }

        instructions
    }

    pub fn new() -> Self {
        Self {
            instructions: HashMap::new(),
        }
    }

    pub fn insert(&mut self, c: char, instruction: Instruction) {
        self.instructions.insert(c, instruction);
    }

    pub fn apply(&self, input: &str) -> Vec<Instruction> {
        let mut output = Vec::new();

        for c in input.chars() {
            if let Some(instruction) = self.instructions.get(&c) {
                output.push(instruction.clone());
            }
        }

        output
    }
}

pub struct SystemOptions {
    pub branch_color: Color,
    pub branch_width: f32,
}

#[derive(Clone)]
struct Branch {
    indecies: [u32; 2],
    position: Point,
    rotation: Matrix,
    scale: f32,
}

fn apply_instruction(
    mesh: &mut Mesh,
    stack: &mut Vec<Branch>,
    options: &SystemOptions,
    instruction: Instruction,
) {
    let depth = stack.len();

    let Some(branch) = stack.last_mut() else {
        return;
    };

    match instruction {
        Instruction::Forward(mut length) => {
            // apply the scale
            length *= branch.scale;

            let depth_scale = f32::powi(0.9, depth as i32);
            let width = options.branch_width * depth_scale;

            let forward = branch.rotation * Vector::NEG_Y * length;
            let left = branch.rotation * Vector::NEG_X * width / 2.0;

            let index = mesh.vertices.len() as u32;
            mesh.vertices.push(Vertex {
                position: branch.position + left,
                tex_coords: Point::ZERO,
                color: options.branch_color,
            });
            mesh.vertices.push(Vertex {
                position: branch.position - left,
                tex_coords: Point::ZERO,
                color: options.branch_color,
            });

            mesh.indices.push(branch.indecies[0]);
            mesh.indices.push(branch.indecies[1]);
            mesh.indices.push(index);

            mesh.indices.push(branch.indecies[1]);
            mesh.indices.push(index);
            mesh.indices.push(index + 1);

            branch.indecies = [index, index + 1];
            branch.position += forward;
        }
        Instruction::Turn(angle) => {
            let rotation = Matrix::from_angle(angle.to_radians());
            branch.rotation = branch.rotation * rotation;
        }
        Instruction::Scale(scale) => {
            branch.scale *= scale;
        }
        Instruction::Push => {
            let branch = branch.clone();
            stack.push(branch);
        }
        Instruction::Pop => {
            stack.pop();
        }
    }
}

pub fn generate_mesh(options: &SystemOptions, instructions: &[Instruction]) -> Mesh {
    let mut mesh = Mesh::new();
    let mut stack = Vec::new();

    let x = options.branch_width / 2.0;
    mesh.vertices.push(Vertex {
        position: Point::new(-x, 0.0),
        tex_coords: Point::ZERO,
        color: options.branch_color,
    });
    mesh.vertices.push(Vertex {
        position: Point::new(x, 0.0),
        tex_coords: Point::ZERO,
        color: options.branch_color,
    });

    stack.push(Branch {
        indecies: [0, 1],
        position: Point::ZERO,
        rotation: Matrix::IDENTITY,
        scale: 1.0,
    });

    for instruction in instructions {
        apply_instruction(&mut mesh, &mut stack, options, instruction.clone());
    }

    mesh
}
