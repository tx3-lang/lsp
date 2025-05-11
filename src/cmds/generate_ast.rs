use serde_json::{json, Value};

use crate::{Context, Error};

pub struct Args {
    document_url: String,
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
        })
    }
}

pub async fn run(
    context: &Context,
    args: impl TryInto<Args, Error = Error>,
) -> Result<Option<Value>, Error> {
    let args: Args = args.try_into()?;

    let protocol = context.get_document_protocol(&args.document_url)?;

    let ast = protocol.ast().to_owned();

    let out = json!({
        "ast": ast,
    });

    Ok(Some(out))
}
