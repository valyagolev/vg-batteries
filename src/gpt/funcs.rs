use std::collections::HashMap;

use anyhow::{Ok, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

static OPENAI_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    // dotenv().ok();
    // dotenv::from_filename(".envrc").ok();

    reqwest::Client::builder()
        .default_headers(
            (&[
                // ("Content-Type", "application/json".to_owned()),
                // ("Accept", "application/json".to_owned()),
                (
                    "Authorization",
                    format!(
                        "Bearer {}",
                        std::env::var("OPENAI_API_KEY").expect("needs OPENAI_API_KEY")
                    ),
                ),
            ]
            .into_iter()
            .map(|x| (x.0.to_owned(), x.1))
            .collect::<HashMap<_, _>>())
                .try_into()
                .expect("bad headers"),
        )
        .build()
        .expect("bad client")
});

#[derive(Debug, Clone, Serialize)]
#[serde(into = "Value")]
pub struct Type<'a> {
    pub name: &'a str,
    pub variants: Option<Vec<&'a str>>,
    pub description: Option<&'a str>,
    pub required: bool,
}

impl<'a> Into<Value> for Type<'a> {
    fn into(self) -> Value {
        let mut v = json!({
            "type": self.name,
        });
        if let Some(variants) = self.variants {
            v["enum"] = json!(variants);
        };
        v
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(into = "Value")]
pub struct Function<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub parameters: &'a [(&'a str, Type<'a>)],
}

impl<'a> Into<Value> for Function<'a> {
    fn into(self) -> Value {
        let mut v = json!({
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "required": vec!["todos", "any_or_all"],
                "properties": {
                    "todos": {
                        "type": "array",
                        "minContains": 1,
                        "items": {
                            "type": "object",
                            "required": self.parameters.iter().filter(|x| x.1.required).map(|x| x.0).collect::<Vec<_>>(),
                            "properties": self.parameters.into_iter().cloned().collect::<HashMap<_, _>>(),
                        }
                    },
                    "any_or_all": {
                        "type": "string",
                        "enum": ["any", "all"],
                        "description": "If all items are to be completed, or it's a choice of any of them"
                    }
                }
            }
        });
        v
    }
}

#[derive(Debug)]
pub enum AiResp {
    Text(String),
    Call(String, Value),
    Weird(Value),
}

pub async fn ai_query(
    api_base: &str,
    system: &str,
    user: &str, //, etc: Option<Value>
    functions: &[&Function<'_>],
) -> Result<AiResp> {
    // let api_base = API_BASE;

    let query = json!({
        // "model": "text-davinci-003".to_string(),

        // "prompt": prompt,

        // "max_tokens": 512,
        // "temperature": 0.2,
        // "top_p": 1.0,
        // "frequency_penalty": 0.0,
        // "presence_penalty": 0.0,
        // "n": 1,
        // "@PromptStudio": {
        //     "user": user
        // }
        // "model": "gpt-3.5-turbo",
        // "model": "gpt-3.5-turbo-0613",
        "model": "gpt-4-0613",
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "functions": functions,
        "function_call": "auto"
    });
    // if let Some(etc) = etc {
    //     merge(&mut query, &etc);
    // }
    println!("{}", serde_json::to_string_pretty(&query)?);

    // let resp = OPENAI_CLIENT.completions().create(query).await?;

    let resp = OPENAI_CLIENT
        .post(format!(
            "{}/v1/chat/completions",
            api_base.trim_end_matches('/')
        ))
        .json(&query)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("{}", serde_json::to_string_pretty(&resp)?);

    let call: Option<AiResp> = (|| {
        let call = resp
            .get("choices")?
            .get(0)?
            .get("message")?
            .get("function_call")?;

        Some(AiResp::Call(
            call.get("name")?.as_str()?.to_owned(),
            serde_json::from_str(call.get("arguments")?.as_str()?).ok()?,
        ))
    })();

    if let Some(call) = call {
        return Ok(call);
        // let call = call?;
    }

    let value: Option<String> = (|| {
        Some(
            resp.get("choices")?
                .get(0)?
                .get("message")?
                .get("content")?
                .as_str()?
                .to_owned(),
        )
    })();

    match value {
        Some(value) => Ok(AiResp::Text(value)),
        None => Ok(AiResp::Weird(resp)),
    }
}

fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), Value::Object(b)) => {
            for (k, v) in b {
                merge(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

// pub async fn ai_query_streaming(
//     api_base: &str,
//     prompt: &str,
//     etc: Option<Value>,
// ) -> Result<impl Stream<Item = anyhow::Result<String>>> {
//     let mut query = json!({
//         "model": "text-davinci-003".to_string(),

//         "prompt": prompt,

//         "max_tokens": 512,
//         "temperature": 0.2,
//         "top_p": 1.0,
//         "frequency_penalty": 0.0,
//         "presence_penalty": 0.0,
//         "stream": true,
//         "n": 1
//     });
//     if let Some(etc) = etc {
//         merge(&mut query, &etc);
//     }

//     // let resp = OPENAI_CLIENT.completions().create(query).await?;

//     let stream = OPENAI_CLIENT
//         .post(format!("{}/v1/completions", api_base.trim_end_matches('/')))
//         .json(&query)
//         .send()
//         .await?
//         // .json::<serde_json::Value>()
//         // .await?;
//         .bytes_stream()
//         .eventsource()
//         .map_err(anyhow::Error::new);

//     Ok(stream.and_then(|e| async move {
//         let value = serde_json::from_str::<serde_json::Value>(&e.data)?;

//         let value: Option<_> = try {
//             value
//                 .get("@PromptStudio")?
//                 .get("combined")?
//                 .get(0)?
//                 .as_str()?
//         };

//         let value = value.ok_or(anyhow!("no value"))?.to_owned();

//         // println!("{:?}", value);

//         Ok(value)
//     }))
// }

// #[cfg(test)]
// mod tests {
//     use std::io::Write;

//     use super::*;
//     // use anystore::stores::json;
//     // use anyhow::*;
//     use dotenv::dotenv;
//     use rand::Rng;
//     use tokio::time::Instant;

//     const API_BASE: &str =
//         "";

//     #[tokio::test]
//     async fn test_ai_grammar_check() -> anyhow::Result<()> {
//         let prompts = String::from_utf8(tokio::fs::read("prompts.txt").await?)?
//             .lines()
//             .filter_map(|s| {
//                 let s = s.trim();
//                 if s.is_empty() {
//                     None
//                 } else {
//                     Some(s.to_owned())
//                 }
//             })
//             .collect::<Vec<_>>();

//         let request_count = 10;

//         let all_requests = futures::future::join_all((0..request_count).map(|_| {
//             let mut rng = rand::thread_rng();
//             let prompt = prompts[rng.gen_range(0..prompts.len())].to_owned();
//             async move {
//                 let start = Instant::now();
//                 // ai_query(
//                 //     API_BASE,
//                 //     "This bot generates a random prompt for a startup:",
//                 //     json!({
//                 //         "id": 1
//                 //     }),
//                 // )
//                 // .await?;

//                 let result = ai_query_streaming(
//                     API_BASE,
//                     // "This bot generates a random prompt for a startup:",
//                     &prompt,
//                     Some(json!({
//                         "stop": "2.",
//                         "@PromptStudio": {
//                             "user": json!({
//                                 "id": rng.gen_range(1..4)
//                             })
//                         }
//                     })),
//                 )
//                 .await?
//                 .try_collect::<Vec<_>>()
//                 .await?
//                 .last()
//                 .map(|s| s.to_owned());

//                 if let Some(s) = result {
//                     println!("{}", s);
//                 };

//                 let duration = start.elapsed();

//                 // print!(".");
//                 std::io::stdout().flush()?;

//                 Ok(duration)
//             }
//         }))
//         .await
//         .into_iter()
//         .collect::<Vec<Result<_, _>>>();

//         let all_good_requests = all_requests
//             .iter()
//             .filter_map(|r| r.as_ref().ok())
//             .map(|r| r.to_owned())
//             .collect::<Vec<_>>();

//         println!("\n\n\n");

//         println!(
//             "min duration: {:?}",
//             all_good_requests.iter().min().unwrap()
//         );
//         println!(
//             "max duration: {:?}",
//             all_good_requests.iter().max().unwrap()
//         );
//         println!(
//             "avg duration: {:?}",
//             all_good_requests.iter().sum::<std::time::Duration>() / request_count
//         );

//         all_requests.into_iter().collect::<Result<Vec<_>, _>>()?;

//         Ok(())
//     }

//     // #[tokio::test]
//     // async fn test_ai_grammar_check_streaming() {
//     //     let mut stream = ai_grammar_check_streaming("this is a test", json!({}))
//     //         .await
//     //         .unwrap();

//     //     while let Some(resp) = stream.try_next().await.unwrap() {
//     //         println!("{:?}", resp);
//     //     }
//     // }
// }
