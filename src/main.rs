mod system;

use ori::prelude::*;
use system::{Instruction, Instructions, Rules, SystemOptions};

const DISPLAY_SIZE: Size = Size::all(450.0);
const INITIAL_AXIOM: &str = "A";
const INITIAL_RULES: &str = "A -> F[-A]F[-A]+FA\nF -> FF";
const INITIAL_INSTRUCTIONS: &str = include_str!("instructions.txt");

struct Data {
    mesh: Option<Mesh>,
    axiom: String,
    rules: String,
    instructions: String,
    options: SystemOptions,
    iterations: String,
}

impl Data {
    fn new() -> Self {
        let options = SystemOptions {
            branch_color: hex("#6ac974"),
            branch_width: 3.0,
        };

        let mut data = Self {
            mesh: None,
            axiom: String::from(INITIAL_AXIOM),
            rules: String::from(INITIAL_RULES),
            instructions: String::from(INITIAL_INSTRUCTIONS),
            options,
            iterations: String::from("7"),
        };

        data.instructions.pop();

        data.generate_mesh();
        data
    }

    fn mesh_bounds(mesh: &Mesh) -> Rect {
        let mut bounds = Rect::ZERO;

        for vertex in mesh.vertices.iter() {
            bounds.min = Point::min(bounds.min, vertex.position);
            bounds.max = Point::max(bounds.max, vertex.position);
        }

        bounds
    }

    fn generate_mesh(&mut self) {
        let rect = Rect::min_size(Point::ZERO, DISPLAY_SIZE);

        let rules = Rules::parse(&self.rules);

        let mut tree = self.axiom.clone();

        for _ in 0..self.iterations() {
            tree = rules.apply(&tree);
        }

        let instructions = self.instructions().apply(&tree);
        let mut mesh = system::generate_mesh(&self.options, &instructions);
        let bounds = Self::mesh_bounds(&mesh);

        // scale and center the mesh
        let scale = rect.size() / bounds.size();
        let scale = f32::min(scale.width, scale.height);
        let offset = rect.bottom() - bounds.bottom() * scale;

        for vertex in mesh.vertices.iter_mut() {
            vertex.position *= scale;
            vertex.position += offset;
        }

        self.mesh = Some(mesh);
    }

    fn set_axiom(&mut self, axiom: String) {
        if self.axiom != axiom {
            self.axiom = axiom;
            self.generate_mesh();
        }
    }

    fn rules(&self) -> Rules {
        Rules::parse(&self.rules)
    }

    fn set_rules(&mut self, rules: String) {
        let prev = self.rules();
        self.rules = rules;

        if prev != self.rules() {
            self.generate_mesh();
        }
    }

    fn instructions(&self) -> Instructions {
        let mut instructions = Instructions::parse(&self.instructions);

        instructions.insert('[', Instruction::Push);
        instructions.insert(']', Instruction::Pop);

        instructions
    }

    fn set_instructions(&mut self, instructions: String) {
        let prev = self.instructions();
        self.instructions = instructions;

        if prev != self.instructions() {
            self.generate_mesh();
        }
    }

    fn iterations(&self) -> usize {
        self.iterations.parse().unwrap_or(0)
    }

    fn set_iterations(&mut self, iterations: String) {
        let prev = self.iterations();
        self.iterations = iterations;

        if prev != self.iterations() {
            self.generate_mesh();
        }
    }
}

fn background(content: impl View<Data>) -> impl View<Data> {
    let colors = &[style(Palette::BACKGROUND), style(Palette::BACKGROUND_LIGHT)];
    container(content).background(gradient(-45.0, colors))
}

fn mesh_painter() -> impl View<Data> {
    let painter = painter(|_cx, data: &mut Data, canvas| {
        if let Some(mesh) = data.mesh.clone() {
            canvas.draw(mesh);
        }
    });

    let painter = container(painter)
        .border_bottom(1.0)
        .border_color(style(Palette::ACCENT_DARKER));

    size(DISPLAY_SIZE, painter)
}

fn regenerate() -> impl View<Data> {
    let button = button(text("Regenerate").font_size(24.0)).fancy(6.0);

    on_click(button, |_, data: &mut Data| {
        data.generate_mesh();
        info!("Regenerating mesh");
    })
}

fn restart() -> impl View<Data> {
    let button = button(text("Restart").font_size(24.0)).fancy(6.0);

    on_click(button, |_, data: &mut Data| {
        *data = Data::new();
        info!("Restarting");
    })
}

fn button_row() -> impl View<Data> {
    hstack![regenerate(), restart()].gap(10.0)
}

fn input_container(content: impl View<Data>) -> impl View<Data> {
    container(pad(8.0, content))
        .background(style(Palette::ACCENT))
        .border_radius(8.0)
}

fn axiom(data: &mut Data) -> impl View<Data> {
    let label = text("Axiom").font_size(24.0);
    let input = text_input()
        .on_change(|_, data: &mut Data, text| data.set_axiom(text))
        .text(&data.axiom)
        .font_family(FontFamily::Name(String::from("Noto Sans Mono")));

    let input = input_container(input);
    let content = hstack![label, flex_grow(1.0, input)].gap(10.0);

    alt("The starting string for the L-system", content)
}

fn iterations(data: &mut Data) -> impl View<Data> {
    let input = text_input()
        .on_change(|_, data: &mut Data, text| data.set_iterations(text))
        .text(&data.iterations)
        .font_family(FontFamily::Name(String::from("Noto Sans Mono")));

    alt("Number of iterations", width(100.0, input_container(input)))
}

fn axiom_row(data: &mut Data) -> impl View<Data> {
    let content = hstack![flex_grow(1.0, axiom(data)), iterations(data)].gap(10.0);
    width(FILL, content)
}

fn rules(data: &mut Data) -> impl View<Data> {
    let input = text_input()
        .on_change(|_, data: &mut Data, text| data.set_rules(text))
        .text(&data.rules)
        .multiline(true)
        .font_family(FontFamily::Name(String::from("Noto Sans Mono")));

    alt(
        "Rules for the L-system",
        width(FILL, input_container(input)),
    )
}

fn instructions(data: &mut Data) -> impl View<Data> {
    let input = text_input()
        .on_change(|_, data: &mut Data, text| data.set_instructions(text))
        .text(&data.instructions)
        .multiline(true)
        .font_family(FontFamily::Name(String::from("Noto Sans Mono")));

    alt(
        "Instructions for the L-system",
        width(FILL, input_container(input)),
    )
}

fn ui(data: &mut Data) -> impl View<Data> {
    let content = vstack![
        mesh_painter(),
        button_row(),
        axiom_row(data),
        rules(data),
        instructions(data)
    ]
    .align_items(Align::Center)
    .gap(12.0);

    let content = vscroll(content);

    size(FILL, background(pad(20.0, top(content))))
}

fn palette() -> Palette {
    Palette {
        background: hex("#dd6c81"),
        accent: hex("#adc96a"),
        ..Palette::light()
    }
}

fn main() {
    let window = WindowDescriptor::new()
        .title("Lily")
        .size(500, 800)
        .resizable(false);

    Launcher::new(Data::new())
        .window(window, ui)
        .theme(palette)
        .launch();
}
