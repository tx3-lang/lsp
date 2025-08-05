use std::fmt::Write;
use tx3_lang::ast::Identifier;
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
    if program
        .policies
        .iter()
        .any(|policy| policy.name.value == name)
    {
        PartyType::Policy
    } else if program.parties.iter().any(|party| party.name.value == name) {
        PartyType::Party
    } else {
        PartyType::Unknown
    }
}

fn get_icon_svg(party_type: &PartyType, x: &i32, y: &i32, width: &i32, height: &i32) -> String {
    let svg = match party_type {
        PartyType::Unknown | PartyType::Party => {
            r#"
            <path d="M16 2C12.134 2 9 5.13401 9 9V13C9 15.3787 10.1865 17.4804 12 18.7453V21H8.01722C5.78481 21 3.82288 22.4799 3.20959 24.6264L2.03848 28.7253C1.95228 29.027 2.0127 29.3517 2.20166 29.6022C2.39062 29.8527 2.68622 30 3.00001 30H29C29.3138 30 29.6094 29.8527 29.7984 29.6022C29.9873 29.3517 30.0477 29.027 29.9615 28.7253L28.7904 24.6264C28.1771 22.4799 26.2152 21 23.9828 21H20V18.7453C21.8135 17.4804 23 15.3787 23 13V9C23 5.13401 19.866 2 16 2Z" fill="white"/>
            "#
        }
        PartyType::Policy => {
            r#"
            <path fill-rule="evenodd" clip-rule="evenodd" d="M5 5C5 3.34315 6.34315 2 8 2H24C25.6569 2 27 3.34315 27 5V27C27 28.6569 25.6569 30 24 30H8C6.34315 30 5 28.6569 5 27V5ZM10 6C9.44772 6 9 6.44772 9 7C9 7.55228 9.44772 8 10 8H12C12.5523 8 13 7.55228 13 7C13 6.44772 12.5523 6 12 6H10ZM19 20C18.4477 20 18 20.4477 18 21C18 21.5523 18.4477 22 19 22H22C22.5523 22 23 21.5523 23 21C23 20.4477 22.5523 20 22 20H19ZM21 23C20.4477 23 20 23.4477 20 24C20 24.5523 20.4477 25 21 25H22C22.5523 25 23 24.5523 23 24C23 23.4477 22.5523 23 22 23H21ZM15 6C14.4477 6 14 6.44772 14 7C14 7.55228 14.4477 8 15 8H22C22.5523 8 23 7.55228 23 7C23 6.44772 22.5523 6 22 6H15ZM10 9C9.44772 9 9 9.44772 9 10C9 10.5523 9.44772 11 10 11H22C22.5523 11 23 10.5523 23 10C23 9.44772 22.5523 9 22 9H10ZM10 12C9.44772 12 9 12.4477 9 13C9 13.5523 9.44772 14 10 14H22C22.5523 14 23 13.5523 23 13C23 12.4477 22.5523 12 22 12H10ZM13 15C10.7909 15 9 16.7909 9 19C9 21.2091 10.7909 23 13 23C15.2091 23 17 21.2091 17 19C17 16.7909 15.2091 15 13 15ZM13 24C11.8744 24 10.8357 23.6281 10 23.0004V26C10 26.3466 10.1795 26.6684 10.4743 26.8507C10.7691 27.0329 11.1372 27.0494 11.4472 26.8944L13 26.118L14.5528 26.8944C14.8628 27.0494 15.2309 27.0329 15.5257 26.8507C15.8205 26.6684 16 26.3466 16 26V23.0004C15.1643 23.6281 14.1256 24 13 24Z" fill="white"/>
            "#
        }
    };

    format!(
        r#"<svg x="{x}%" y="{y}%" width="{width}%" height="{height}%" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" fill="none">
            {svg}
        </svg>"#,
        x = x,
        y = y,
        width = width,
        height = height,
        svg = svg
    )
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

    let mut parties: Vec<Party> = names
        .into_iter()
        .map(|name| Party {
            name: name.clone(),
            party_type: infer_party_type(ast, &name),
        })
        .collect();

    parties.sort_by_key(|p| p.name.clone());

    parties
}

fn get_output_parties(ast: &Program, tx: &TxDef) -> Vec<Party> {
    let mut names = std::collections::HashSet::new();

    for output in &tx.outputs {
        for field in &output.fields {
            if let OutputBlockField::To(address_expr) = field {
                if let Some(identifier) = address_expr.as_identifier() {
                    names.insert(identifier.value.clone());
                }
            }
        }
    }

    let mut parties: Vec<Party> = names
        .into_iter()
        .map(|name| Party {
            name: name.clone(),
            party_type: infer_party_type(ast, &name),
        })
        .collect();

    parties.sort_by_key(|p| p.name.clone());

    parties
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
                .clone()
                .name
                .unwrap_or(Identifier::new(format!("output {}", i + 1)))
                .value;

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
fn render_party(party: &Party, x: i32, y: i32) -> String {
    format!(
        r#"<svg x="{x}" y="{y}" width="{unit}" height="{unit}" viewBox="0 0 {unit} {unit}">
    {image_svg}
        <text x="50%" y="{text_y}%" text-anchor="middle" font-size="{font_size}%" font-family="monospace" fill="rgb(255, 255, 255)">{name}</text>
    </svg>"#,
        x = x,
        y = y,
        unit = UNIT,
        image_svg = get_icon_svg(&party.party_type, &25, &15, &50, &60),
        text_y = 85,
        font_size = 14,
        name = party.name,
    )
}

fn render_parameter(param: &Parameter, x: i32, y: i32) -> String {
    format!(
        r#"
        <g transform="translate(-{unit},{half_unit})">
        <svg x="{x}" y="{y}" width="{width}" height="{height}" viewBox="0 0 {unit} {quarter_unit}">
            <text x="50%" y="10%" text-anchor="middle" dominant-baseline="hanging" font-size="10%" font-family="monospace" fill="rgb(255, 255, 255)">{name}</text>
            <line x1="20%" y1="90%" x2="80%" y2="90%" stroke="rgb(255, 255, 255)" stroke-width="0.25"/>
            <line x1="70%" y1="80%" x2="80%" y2="90%" stroke="rgb(255, 255, 255)" stroke-width="0.25"/>
            <line x1="70%" y1="100%" x2="80%" y2="90%" stroke="rgb(255, 255, 255)" stroke-width="0.25"/>
        </svg>
    </g>"#,
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

fn render_tx(tx: &TxDef, x: i32, y: i32) -> String {
    format!(
        r#"<g transform="translate(-{unit})">
        <svg x="{x}" y="{y}" width="{width}" height="{height}" viewBox="0 0 {unit} {double_unit}">
            <rect width="100%" height="100%" rx="{corner}" ry="{corner}" fill-opacity="0" stroke="white" stroke-width="0.25" stroke-linecap="round" stroke-linejoin="round"/>
            <text x="50%" y="50%" text-anchor="middle" dominant-baseline="middle" font-size="10%" font-family="monospace" fill="rgb(255, 255, 255)">{name}</text>
        </svg>
    </g>"#,
        x = x,
        y = y,
        unit = UNIT,
        double_unit = UNIT * 2,
        width = UNIT * 2,
        height = UNIT * 4,
        corner = UNIT as f64 / 10.0,
        name = tx.name.value
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
        r#"<svg width="100%" viewBox="0 0 {width} {height}" style="margin-block-end:64px; margin-block-start:64px; margin-bottom:64px; margin-left:0px; margin-right:0px; margin-top:64px;">"#,
        width = CANVA_WIDTH,
        height = CANVA_HEIGHT
    ).unwrap();

    // Render transaction box in the center
    write!(svg, "{}", render_tx(tx, CANVA_WIDTH / 2, 0)).unwrap();

    // Render input parties on the left
    for (i, party) in input_parties.iter().enumerate() {
        write!(svg, "{}", render_party(party, 0, UNIT * i as i32)).unwrap();
    }

    // Render output parties on the right
    for (i, party) in output_parties.iter().enumerate() {
        write!(
            svg,
            "{}",
            render_party(party, CANVA_WIDTH - UNIT, UNIT * i as i32)
        )
        .unwrap();
    }

    // Render input parameters
    write!(
        svg,
        r#"<g transform="translate({half_unit})">"#,
        half_unit = UNIT / 2
    )
    .unwrap();
    for (i, input) in inputs.iter().enumerate() {
        write!(
            svg,
            "{}",
            render_parameter(input, CANVA_WIDTH / 4, UNIT * i as i32)
        )
        .unwrap();
    }
    write!(svg, "</g>").unwrap();

    // Render output parameters
    write!(
        svg,
        r#"<g transform="translate(-{half_unit})">"#,
        half_unit = UNIT / 2
    )
    .unwrap();
    for (i, output) in outputs.iter().enumerate() {
        write!(
            svg,
            "{}",
            render_parameter(output, CANVA_WIDTH * 3 / 4, UNIT * i as i32)
        )
        .unwrap();
    }
    write!(svg, "</g>").unwrap();

    // Draw lines from input parties to input parameters
    for (input_index, input) in inputs.iter().enumerate() {
        if let Some(ref name) = input.party {
            if let Some(party_index) = input_parties.iter().position(|p| &p.name == name) {
                write!(
                svg,
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"rgb(255, 255, 255)\" stroke-width=\"0.4\" stroke-dasharray=\"1,1\" stroke-opacity=\"0.5\"/>",
                UNIT,
                UNIT * (party_index as i32) + UNIT / 2,
                CANVA_WIDTH / 4 - UNIT / 8,
                UNIT * (input_index as i32 + 1) - UNIT / 16,
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
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"rgb(255, 255, 255)\" stroke-width=\"0.4\" stroke-dasharray=\"1,1\" stroke-opacity=\"0.5\"/>",
                CANVA_WIDTH / 2 + CANVA_WIDTH / 4 + UNIT / 8,
                UNIT * (output_index as i32 + 1) - UNIT / 16,
                (CANVA_WIDTH - UNIT),
                (UNIT * (party_index as i32) + UNIT / 2)
            ).unwrap();
            }
        }
    }

    svg.push_str("</svg>");

    svg
}
