use std::collections::HashMap;

use serde_json::{json, Value};

use crate::{Context, Error};

#[derive(Debug)]
pub struct Args {
    document_url: String,
    tx_name: String,
}

impl TryFrom<Vec<Value>> for Args {
    type Error = Error;

    fn try_from(value: Vec<Value>) -> Result<Self, Self::Error> {
        Ok(Args {
            document_url: value
                .get(0)
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
                .ok_or(Error::InvalidCommandArgs("document_url".to_string()))?,
            tx_name: value
                .get(1)
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
                .ok_or(Error::InvalidCommandArgs("tx_name".to_string()))?,
        })
    }
}

pub async fn run(
    context: &Context,
    args: impl TryInto<Args, Error = Error>,
) -> Result<Option<Value>, Error> {
    let args: Args = args.try_into()?;

    let protocol = context.get_document_protocol(&args.document_url)?;

    let prototx = protocol.new_tx(&args.tx_name)?;

    let params = prototx
        .find_params()
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::to_value(v).unwrap()))
        .collect::<HashMap<String, Value>>();

    let out = json!({
        "tir": hex::encode(prototx.ir_bytes()),
        "version": tx3_lang::ir::IR_VERSION,
        "parameters": params,
    });

    Ok(Some(out))
}
