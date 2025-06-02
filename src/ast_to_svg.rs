use std::fmt::Write;
use tx3_lang::ast::InputBlockField;
use tx3_lang::ast::OutputBlockField;
use tx3_lang::ast::Program;
use tx3_lang::ast::TxDef;

const UNIT: i32 = 16;
const CANVA_WIDTH: i32 = UNIT * 10;
const CANVA_HEIGHT: i32 = UNIT * 4;

// Supporting Structs and Functions
#[derive(Debug, Clone, PartialEq, Eq)]
enum PartyType {
    Unknown,
    Party,
    Policy,
}

#[derive(Debug, Clone)]
struct Party {
    name: String,
    party_type: PartyType,
}

#[derive(Debug, Clone)]
struct Parameter {
    name: String,
    party: Option<String>,
}

fn infer_party_type(program: &Program, name: &str) -> PartyType {
    if program.policies.iter().any(|policy| policy.name == name) {
        PartyType::Policy
    } else if program.parties.iter().any(|party| party.name == name) {
        PartyType::Party
    } else {
        PartyType::Unknown
    }
}

fn get_icon_url(party_type: &PartyType) -> &str {
    match party_type {
        PartyType::Unknown => "images/party.svg",

        PartyType::Party => "images/party.svg",

        PartyType::Policy => "images/policy.svg",
    }
}

fn get_input_parties(ast: &Program, tx: &TxDef) -> Vec<Party> {
    let mut names = std::collections::HashSet::new();
    for input in &tx.inputs {
        for field in &input.fields {
            if let InputBlockField::From(address_expr) = field {
                if let Some(identifier) = address_expr.as_identifier() {
                    names.insert(identifier.value.clone());
                }
            }
        }
    }
    names
        .into_iter()
        .map(|name| Party {
            name: name.clone(),
            party_type: infer_party_type(ast, &name),
        })
        .collect()
}
fn get_output_parties(ast: &Program, tx: &TxDef) -> Vec<Party> {
    let mut names = std::collections::HashSet::new();
    for output in &tx.outputs {
        for field in &output.fields {
            // REVIEW
            if let OutputBlockField::To(address_expr) = field {
                if let Some(identifier) = address_expr.as_identifier() {
                    // Assuming Identifier has a 'value' field of type String
                    names.insert(identifier.value.clone());
                }
            }
        }
    }
    names
        .into_iter()
        .map(|name| Party {
            name: name.clone(),
            party_type: infer_party_type(ast, &name),
        })
        .collect()
}

fn get_inputs(tx: &TxDef) -> Vec<Parameter> {
    tx.inputs
        .iter()
        .map(|input| {
            let name = input.name.clone();
            let party = input.fields.iter().find_map(|f| {
                if let InputBlockField::From(address_expr) = f {
                    address_expr
                        .as_identifier()
                        .map(|ident| ident.value.clone())
                } else {
                    None
                }
            });
            Parameter { name, party }
        })
        .collect()
}

fn get_outputs(tx: &TxDef) -> Vec<Parameter> {
    tx.outputs
        .iter()
        .enumerate()
        .map(|(i, output)| {
            let name = output
                .name
                .clone()
                .unwrap_or_else(|| format!("output {}", i + 1));
            let party = output.fields.iter().find_map(|f| {
                if let OutputBlockField::To(address_expr) = f {
                    address_expr
                        .as_ref()
                        .as_identifier()
                        .map(|ident| ident.value.clone())
                } else {
                    None
                }
            });
            Parameter { name, party }
        })
        .collect()
}

// SVG Rendering Functions
fn render_party(party: &Party, x: usize, y: usize) -> String {
    format!(
        "<svg x=\"{x}\" y=\"{y}\" width=\"{unit}\" height=\"{unit}\" viewBox=\"0 0 {unit} {unit}\">
            <image x=\"{image_x}%\" y=\"{image_y}%\" width=\"{image_width}%\" height=\"{image_height}%\" href=\"{href}\" />
            <text x=\"50%\" y=\"{text_y}%\" text-anchor=\"middle\" font-size=\"{font_size}%\" font-family=\"monospace\" fill=\"#fff\">{name}</text>
        </svg>",
        x = x,
        y = y,
        unit = UNIT,
        image_x = 25,
        image_y = 15,
        image_width = 50,
        image_height = 60,
        href = get_icon_url(&party.party_type),
        text_y = 85,
        font_size = 14,
        name = party.name,
    )
}

fn render_parameter(param: &Parameter, x: usize, y: usize) -> String {
    format!(
        "<g transform=\"translate(-{unit},{half_unit})\">
            <svg x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {unit} {quarter_unit}\">
                <text x=\"50%\" y=\"10%\" text-anchor=\"middle\" dominant-baseline=\"hanging\" font-size=\"10%\" font-family=\"monospace\" fill=\"#fff\">{name}</text>
                <line x1=\"20%\" y1=\"90%\" x2=\"80%\" y2=\"90%\" stroke=\"#fff\" stroke-width=\"0.25\"/>
                <line x1=\"70%\" y1=\"80%\" x2=\"80%\" y2=\"90%\" stroke=\"#fff\" stroke-width=\"0.25\"/>
                <line x1=\"70%\" y1=\"100%\" x2=\"80%\" y2=\"90%\" stroke=\"#fff\" stroke-width=\"0.25\"/>
            </svg>
        </g>",
        x = x,
        y = y,
        unit = UNIT,
        half_unit = UNIT / 2,
        quarter_unit = UNIT / 4,
        width = UNIT * 2,
        height = UNIT / 2,
        name = param.name
    )
}

fn render_tx(tx: &TxDef, x: usize, y: usize) -> String {
    format!(
        "<g transform=\"translate(-{unit})\">
            <svg x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {unit} {double_unit}\">
                <rect width=\"100%\" height=\"100%\" rx=\"{corner}\" ry=\"{corner}\" fill-opacity=\"0\" stroke=\"white\" stroke-width=\"0.25\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>
                <text x=\"50%\" y=\"50%\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"10%\" font-family=\"monospace\" fill=\"#fff\">{name}</text>
            </svg>
        </g>",
        x = x,
        y = y,
        unit = UNIT,
        double_unit = UNIT * 2,
        width = UNIT * 2,
        height = UNIT * 4,
        corner = UNIT / 10,
        name = tx.name
    )
}
pub fn tx_to_svg(ast: &Program, tx: &TxDef) -> String {
    let input_parties = get_input_parties(ast, tx);
    let output_parties = get_output_parties(ast, tx);
    let inputs = get_inputs(tx);
    let outputs = get_outputs(tx);

    let mut svg = String::new();

    write!(
        svg,
        r#"<svg width="100%" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">"#,
        width = CANVA_WIDTH,
        height = CANVA_HEIGHT
    )
    .unwrap();

    // Render transaction box in the center
    write!(
        svg,
        "{}",
        render_tx(
            tx,
            (CANVA_WIDTH / 2) as usize,
            (CANVA_HEIGHT / 2 - UNIT) as usize
        )
    )
    .unwrap();

    // Render input parties on the left
    for (i, party) in input_parties.iter().enumerate() {
        write!(
            svg,
            "{}",
            render_party(party, 0, (UNIT * i as i32) as usize)
        )
        .unwrap();
    }

    // Render output parties on the right
    for (i, party) in output_parties.iter().enumerate() {
        write!(
            svg,
            "{}",
            render_party(
                party,
                (CANVA_WIDTH - UNIT) as usize,
                (UNIT * i as i32) as usize
            )
        )
        .unwrap();
    }

    // Render input parameters
    for (i, input) in inputs.iter().enumerate() {
        write!(
            svg,
            "{}",
            render_parameter(
                input,
                (CANVA_WIDTH / 4) as usize,
                (UNIT * i as i32) as usize
            )
        )
        .unwrap();
    }

    // Render output parameters
    for (i, output) in outputs.iter().enumerate() {
        write!(
            svg,
            "{}",
            render_parameter(
                output,
                (CANVA_WIDTH * 3 / 4) as usize,
                (UNIT * i as i32) as usize
            )
        )
        .unwrap();
    }
    // Draw lines from input parties to input parameters
    for (input_index, input) in inputs.iter().enumerate() {
        if let Some(ref name) = input.party {
            if let Some(party_index) = input_parties.iter().position(|p| &p.name == name) {
                write!(
                svg,
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#fff\" stroke-width=\"0.4\" stroke-dasharray=\"1,1\" stroke-opacity=\"0.5\"/>",
                UNIT,
                UNIT * (party_index as i32) + UNIT / 2,
                CANVA_WIDTH / 4 - UNIT / 8,
                UNIT * (input_index as i32) + UNIT / 2
            ).unwrap();
            }
        }
    }

    // Draw lines from output parameters to output parties
    for (output_index, output) in outputs.iter().enumerate() {
        if let Some(ref name) = output.party {
            if let Some(party_index) = output_parties.iter().position(|p| &p.name == name) {
                write!(
                svg,
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#fff\" stroke-width=\"0.4\" stroke-dasharray=\"1,1\" stroke-opacity=\"0.5\"/>",
                CANVA_WIDTH * 3 / 4 + UNIT / 8,
                UNIT * (output_index as i32) + UNIT / 2,
                CANVA_WIDTH - UNIT,
                UNIT * (party_index as i32) + UNIT / 2
            ).unwrap();
            }
        }
    }

    svg.push_str("</svg>");

    svg
}
