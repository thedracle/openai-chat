use clap::{arg, Command};

#[derive(Debug)]
pub struct GPTArgs {
    pub api_url: Option<String>,
    pub gpt_model: Option<String>,
    pub api_key: Option<String>,
}

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1/";
const GPT_DEFAULT_MODEL: &str = "gpt-3.5-turbo";
const DEFAULT_API_KEY: &str = "ABCD1234";

pub fn parse_args() -> GPTArgs {
    // Create the command line arguments
    let matches = Command::new("Simple GPT client for the internal Datadog openai proxy.")
        .version("1.0")
        .author("Jason Thomas <mail@jasonthom.as>")
        .about("Chat with GPT in your terminal.")
        .arg(
            arg!(-d --api_url <API_URL>  "Set the base url for the openai api. Default: ")
            )
        .arg(
            arg!(-g --gpt_model <GPT_MODEL> "Set the GPT model used.")
        )
        .arg(
            arg!(-q --api_key <API_KEY> "Set the API key to be used.")
        )
        .get_matches();

    let default_base_url = DEFAULT_BASE_URL.to_string();
    // Get the values for each argument
    let api_url = matches
        .try_get_one::<String>("api_url").unwrap()
        .unwrap_or(&default_base_url);

    let gpt_default_model = GPT_DEFAULT_MODEL.to_string();
    let gpt_model = matches
        .try_get_one::<String>("gpt_model").unwrap()
        .unwrap_or(&gpt_default_model);

    let default_api_key = DEFAULT_API_KEY.to_string();
    let api_key = matches
        .try_get_one::<String>("api_key")
        .unwrap()
        .unwrap_or(&default_api_key);

    GPTArgs {
        api_url: Some(api_url.to_owned()),
        gpt_model: Some(gpt_model.to_owned()),
        api_key: Some(api_key.to_owned()),
    }
}
