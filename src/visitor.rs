#[derive(Debug)]
pub enum SymbolAtOffset<'a> {
    Identifier(&'a tx3_lang::ast::Identifier),
    TypeIdentifier(&'a tx3_lang::ast::TypeRecord),
}

pub fn find_symbol_in_program<'a>(
    program: &'a tx3_lang::ast::Program,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for tx in &program.txs {
        if let Some(sym) = visit_tx_def(tx, offset) {
            return Some(sym);
        }
    }
    for asset in &program.assets {
        if let Some(sym) = visit_asset_def(asset, offset) {
            return Some(sym);
        }
    }
    for ty in &program.types {
        if let Some(sym) = visit_type_def(ty, offset) {
            return Some(sym);
        }
    }
    for party in &program.parties {
        if let Some(sym) = visit_party_def(party, offset) {
            return Some(sym);
        }
    }
    for policy in &program.policies {
        if let Some(sym) = visit_policy_def(policy, offset) {
            return Some(sym);
        }
    }
    None
}

fn visit_tx_def<'a>(tx: &'a tx3_lang::ast::TxDef, offset: usize) -> Option<SymbolAtOffset<'a>> {
    if in_span(&tx.name.span, offset) {
        return Some(SymbolAtOffset::Identifier(&tx.name));
    }
    if let Some(sym) = visit_parameter_list(&tx.parameters, offset) {
        return Some(sym);
    }
    for input in &tx.inputs {
        if let Some(sym) = visit_input_block(input, offset) {
            return Some(sym);
        }
    }
    for output in &tx.outputs {
        if let Some(sym) = visit_output_block(output, offset) {
            return Some(sym);
        }
    }
    for mint in &tx.mints {
        if let Some(sym) = visit_mint_block(mint, offset) {
            return Some(sym);
        }
    }
    for ref_block in &tx.references {
        if let Some(sym) = visit_reference_block(ref_block, offset) {
            return Some(sym);
        }
    }
    for adhoc in &tx.adhoc {
        if let Some(sym) = visit_chain_specific_block(adhoc, offset) {
            return Some(sym);
        }
    }
    for col in &tx.collateral {
        if let Some(sym) = visit_collateral_block(col, offset) {
            return Some(sym);
        }
    }
    if let Some(signers) = &tx.signers {
        if let Some(sym) = visit_signers_block(signers, offset) {
            return Some(sym);
        }
    }
    if let Some(validity) = &tx.validity {
        if let Some(sym) = visit_validity_block(validity, offset) {
            return Some(sym);
        }
    }
    if let Some(burn) = &tx.burn {
        if let Some(sym) = visit_burn_block(burn, offset) {
            return Some(sym);
        }
    }
    if let Some(metadata) = &tx.metadata {
        if let Some(sym) = visit_metadata_block(metadata, offset) {
            return Some(sym);
        }
    }
    None
}

fn visit_parameter_list<'a>(
    params: &'a tx3_lang::ast::ParameterList,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for param in &params.parameters {
        if in_span(&param.name.span, offset) {
            return Some(SymbolAtOffset::Identifier(&param.name));
        }
        if let Some(sym) = visit_type(&param.r#type, offset) {
            return Some(sym);
        }
    }
    None
}

fn visit_type<'a>(ty: &'a tx3_lang::ast::TypeRecord, offset: usize) -> Option<SymbolAtOffset<'a>> {
    // TODO - complete for all types
    match &ty.r#type {
        tx3_lang::ast::Type::Custom(id) => visit_identifier(id, offset),
        tx3_lang::ast::Type::List(inner) => visit_type(inner, offset),
        _ => None,
    }
}

fn visit_identifier<'a>(
    id: &'a tx3_lang::ast::Identifier,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    if in_span(&id.span, offset) {
        Some(SymbolAtOffset::Identifier(id))
    } else {
        None
    }
}

fn visit_input_block<'a>(
    input: &'a tx3_lang::ast::InputBlock,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for field in &input.fields {
        if let Some(sym) = visit_input_block_field(field, offset) {
            return Some(sym);
        }
    }
    None
}

fn visit_input_block_field<'a>(
    field: &'a tx3_lang::ast::InputBlockField,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    match field {
        tx3_lang::ast::InputBlockField::From(addr) => visit_address_expr(addr, offset),
        tx3_lang::ast::InputBlockField::DatumIs(ty) => visit_type(ty, offset),
        tx3_lang::ast::InputBlockField::MinAmount(expr) => visit_asset_expr(expr, offset),
        tx3_lang::ast::InputBlockField::Redeemer(expr) => visit_data_expr(expr, offset),
        tx3_lang::ast::InputBlockField::Ref(expr) => visit_data_expr(expr, offset),
    }
}

fn visit_output_block<'a>(
    output: &'a tx3_lang::ast::OutputBlock,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for field in &output.fields {
        if let Some(sym) = visit_output_block_field(field, offset) {
            return Some(sym);
        }
    }
    None
}

fn visit_output_block_field<'a>(
    field: &'a tx3_lang::ast::OutputBlockField,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    match field {
        tx3_lang::ast::OutputBlockField::To(addr) => visit_address_expr(addr, offset),
        tx3_lang::ast::OutputBlockField::Amount(expr) => visit_asset_expr(expr, offset),
        tx3_lang::ast::OutputBlockField::Datum(expr) => visit_data_expr(expr, offset),
    }
}

fn visit_asset_expr<'a>(
    expr: &'a tx3_lang::ast::AssetExpr,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    match expr {
        tx3_lang::ast::AssetExpr::Identifier(id) => visit_identifier(id, offset),
        tx3_lang::ast::AssetExpr::StaticConstructor(constr) => {
            if let Some(sym) = visit_identifier(&constr.r#type, offset) {
                return Some(sym);
            }
            visit_data_expr(&constr.amount, offset)
        }
        tx3_lang::ast::AssetExpr::AnyConstructor(constr) => {
            if let Some(sym) = visit_data_expr(&constr.policy, offset) {
                return Some(sym);
            }
            if let Some(sym) = visit_data_expr(&constr.asset_name, offset) {
                return Some(sym);
            }
            visit_data_expr(&constr.amount, offset)
        }
        tx3_lang::ast::AssetExpr::BinaryOp(binop) => {
            if let Some(sym) = visit_asset_expr(&binop.left, offset) {
                return Some(sym);
            }
            visit_asset_expr(&binop.right, offset)
        }
        tx3_lang::ast::AssetExpr::PropertyAccess(pa) => visit_property_access(pa, offset),
    }
}

fn visit_data_expr<'a>(
    expr: &'a tx3_lang::ast::DataExpr,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    match expr {
        tx3_lang::ast::DataExpr::Identifier(id) => visit_identifier(id, offset),
        tx3_lang::ast::DataExpr::StructConstructor(sc) => visit_struct_constructor(sc, offset),
        tx3_lang::ast::DataExpr::ListConstructor(lc) => {
            for el in &lc.elements {
                if let Some(sym) = visit_data_expr(el, offset) {
                    return Some(sym);
                }
            }
            None
        }
        tx3_lang::ast::DataExpr::PropertyAccess(pa) => visit_property_access(pa, offset),
        tx3_lang::ast::DataExpr::BinaryOp(binop) => {
            if let Some(sym) = visit_data_expr(&binop.left, offset) {
                return Some(sym);
            }
            visit_data_expr(&binop.right, offset)
        }
        _ => None,
    }
}

fn visit_struct_constructor<'a>(
    sc: &'a tx3_lang::ast::StructConstructor,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    if let Some(sym) = visit_identifier(&sc.r#type, offset) {
        return Some(sym);
    }
    visit_variant_case_constructor(&sc.case, offset)
}

fn visit_variant_case_constructor<'a>(
    vc: &'a tx3_lang::ast::VariantCaseConstructor,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    if let Some(sym) = visit_identifier(&vc.name, offset) {
        return Some(sym);
    }
    for field in &vc.fields {
        if let Some(sym) = visit_record_constructor_field(field, offset) {
            return Some(sym);
        }
    }
    if let Some(spread) = &vc.spread {
        return visit_data_expr(spread, offset);
    }
    None
}

fn visit_record_constructor_field<'a>(
    field: &'a tx3_lang::ast::RecordConstructorField,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    if let Some(sym) = visit_identifier(&field.name, offset) {
        return Some(sym);
    }
    visit_data_expr(&field.value, offset)
}

fn visit_property_access<'a>(
    pa: &'a tx3_lang::ast::PropertyAccess,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    if let Some(sym) = visit_identifier(&pa.object, offset) {
        return Some(sym);
    }
    for id in &pa.path {
        if let Some(sym) = visit_identifier(id, offset) {
            return Some(sym);
        }
    }
    None
}

fn visit_reference_block<'a>(
    rb: &'a tx3_lang::ast::ReferenceBlock,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    visit_data_expr(&rb.r#ref, offset)
}

fn visit_chain_specific_block<'a>(
    _cb: &'a tx3_lang::ast::ChainSpecificBlock,
    _offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    None
}

fn visit_collateral_block<'a>(
    cb: &'a tx3_lang::ast::CollateralBlock,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for field in &cb.fields {
        match field {
            tx3_lang::ast::CollateralBlockField::From(addr) => {
                if let Some(sym) = visit_address_expr(addr, offset) {
                    return Some(sym);
                }
            }
            tx3_lang::ast::CollateralBlockField::MinAmount(expr) => {
                if let Some(sym) = visit_asset_expr(expr, offset) {
                    return Some(sym);
                }
            }
            tx3_lang::ast::CollateralBlockField::Ref(expr) => {
                if let Some(sym) = visit_data_expr(expr, offset) {
                    return Some(sym);
                }
            }
        }
    }
    None
}

fn visit_signers_block<'a>(
    sb: &'a tx3_lang::ast::SignersBlock,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for signer in &sb.signers {
        if let Some(sym) = visit_data_expr(signer, offset) {
            return Some(sym);
        }
    }
    None
}

fn visit_validity_block<'a>(
    vb: &'a tx3_lang::ast::ValidityBlock,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for field in &vb.fields {
        match field {
            tx3_lang::ast::ValidityBlockField::SinceSlot(expr)
            | tx3_lang::ast::ValidityBlockField::UntilSlot(expr) => {
                if let Some(sym) = visit_data_expr(expr, offset) {
                    return Some(sym);
                }
            }
        }
    }
    None
}

fn visit_burn_block<'a>(
    bb: &'a tx3_lang::ast::BurnBlock,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for field in &bb.fields {
        match field {
            tx3_lang::ast::MintBlockField::Amount(expr) => {
                if let Some(sym) = visit_asset_expr(expr, offset) {
                    return Some(sym);
                }
            }
            tx3_lang::ast::MintBlockField::Redeemer(expr) => {
                if let Some(sym) = visit_data_expr(expr, offset) {
                    return Some(sym);
                }
            }
        }
    }
    None
}

fn visit_metadata_block<'a>(
    _mb: &'a tx3_lang::ast::MetadataBlock,
    _offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    None
}

fn visit_mint_block<'a>(
    mb: &'a tx3_lang::ast::MintBlock,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for field in &mb.fields {
        match field {
            tx3_lang::ast::MintBlockField::Amount(expr) => {
                if let Some(sym) = visit_asset_expr(expr, offset) {
                    return Some(sym);
                }
            }
            tx3_lang::ast::MintBlockField::Redeemer(expr) => {
                if let Some(sym) = visit_data_expr(expr, offset) {
                    return Some(sym);
                }
            }
        }
    }
    None
}

fn visit_asset_def<'a>(
    asset: &'a tx3_lang::ast::AssetDef,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    if let Some(sym) = visit_data_expr(&asset.policy, offset) {
        return Some(sym);
    }
    if let Some(sym) = visit_data_expr(&asset.asset_name, offset) {
        return Some(sym);
    }
    None
}

fn visit_type_def<'a>(ty: &'a tx3_lang::ast::TypeDef, offset: usize) -> Option<SymbolAtOffset<'a>> {
    if in_span(&ty.name.span, offset) {
        return Some(SymbolAtOffset::Identifier(&ty.name));
    }
    for case in &ty.cases {
        for field in &case.fields {
            if in_span(&field.r#type.span, offset) {
                return Some(SymbolAtOffset::TypeIdentifier(&field.r#type));
            }
        }
        if let Some(sym) = visit_variant_case(case, offset) {
            return Some(sym);
        }
    }
    None
}

fn visit_variant_case<'a>(
    case: &'a tx3_lang::ast::VariantCase,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    for field in &case.fields {
        if let Some(sym) = visit_record_field(field, offset) {
            return Some(sym);
        }
    }
    None
}

fn visit_record_field<'a>(
    field: &'a tx3_lang::ast::RecordField,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    if in_span(&field.name.span, offset) {
        return Some(SymbolAtOffset::Identifier(&field.name));
    }
    visit_type(&field.r#type, offset)
}

fn visit_party_def<'a>(
    party: &'a tx3_lang::ast::PartyDef,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    if in_span(&party.span, offset) {
        return Some(SymbolAtOffset::Identifier(&party.name));
    }
    None
}

fn visit_policy_def<'a>(
    policy: &'a tx3_lang::ast::PolicyDef,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    match &policy.value {
        tx3_lang::ast::PolicyValue::Constructor(constr) => {
            for field in &constr.fields {
                if let Some(sym) = visit_policy_field(field, offset) {
                    return Some(sym);
                }
            }
        }
        tx3_lang::ast::PolicyValue::Assign(_) => {
            if in_span(&policy.span, offset) {
                return Some(SymbolAtOffset::Identifier(&policy.name));
            }
        }
    }
    None
}

fn visit_policy_field<'a>(
    field: &'a tx3_lang::ast::PolicyField,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    match field {
        tx3_lang::ast::PolicyField::Hash(expr) => visit_data_expr(expr, offset),
        tx3_lang::ast::PolicyField::Script(expr) => visit_data_expr(expr, offset),
        tx3_lang::ast::PolicyField::Ref(expr) => visit_data_expr(expr, offset),
    }
}

fn visit_address_expr<'a>(
    expr: &'a tx3_lang::ast::AddressExpr,
    offset: usize,
) -> Option<SymbolAtOffset<'a>> {
    match expr {
        tx3_lang::ast::AddressExpr::Identifier(id) => visit_identifier(id, offset),
        _ => None,
    }
}

fn in_span(span: &tx3_lang::ast::Span, offset: usize) -> bool {
    span.start <= offset && offset < span.end
}
