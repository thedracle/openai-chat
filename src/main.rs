use std::sync::Arc;

use cursive::{
    theme::{BaseColor, Color, ColorStyle, ColorType, Theme},
    utils::markup::StyledString,
    view::{Nameable, Resizable, ScrollStrategy, Scrollable},
    views::{Dialog, EditView, TextView},
    Cursive,
};
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use tokio::{select, sync::Mutex, task};

mod args;

const CHATGPT_REQUEST_TIMEOUT: u64 = 60;

fn set_monokai_theme(siv: &mut Cursive) {
    let mut theme = Theme::default();

    theme
        .palette
        .set_color("Background", Color::Rgb(39, 40, 34));
    theme.palette.set_color("View", Color::Rgb(39, 40, 34));
    theme
        .palette
        .set_color("Primary", Color::Rgb(248, 248, 242));
    theme
        .palette
        .set_color("Secondary", Color::Rgb(166, 166, 166));

    siv.set_theme(theme);
}

#[tokio::main]
async fn main() {
    let args = args::parse_args();
    println!("Starting with parameters: {:?}", args);

    openai::openai_use_base_url(args.api_url.unwrap().as_str());
    openai::set_key(String::from(args.api_key.unwrap().as_str()));

    let messages = Arc::new(Mutex::new(vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: "You are a large language model built into a command line interface.".to_string(),
        name: None,
    }]));

    let mut siv = cursive::default();

    set_monokai_theme(&mut siv);

    // Create the text view that will display the input.
    let mut output = TextView::new("").with_name("output").scrollable();
    output.set_scroll_strategy(ScrollStrategy::StickToBottom);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    // Create the input view.
    let input = EditView::new();
    let messages_clone = messages.clone();
    let input = input
        .on_submit(move |s, text| {
            tx.send(String::from(text)).unwrap();

            // let mut output = s.find_name::<TextView>("output").unwrap();
            // output.append(text);
            let messages_clone = messages_clone.clone();
            let content = String::from(text);
            tokio::spawn(async move {
                messages_clone.lock().await.push(ChatCompletionMessage {
                    role: ChatCompletionMessageRole::User,
                    content,
                    name: None,
                });
            });
            outp(
                s,
                Color::Dark(BaseColor::Cyan),
                Color::Dark(BaseColor::Black),
                "You: ",
            );
            outpln(
                s,
                Color::Dark(BaseColor::White),
                Color::Dark(BaseColor::Black),
                text,
            );

            let mut input = s.find_name::<EditView>("input").unwrap();
            input.set_content("");
        })
        .with_name("input")
        .full_width();

    // Create a dialog with the text view and the input view.
    let dialog = Dialog::around(
        cursive::views::LinearLayout::vertical()
            .child(output.full_screen())
            .child(input),
    )
    .button("Quit", |s| s.quit());

    // Add the dialog to the cursive root.
    siv.add_layer(dialog);

    let messages_clone = messages.clone();
    let cb_sink = siv.cb_sink().clone();
    task::spawn(async move {
        loop {
            let input = rx.recv().await.unwrap();
            messages_clone.lock().await.push(ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: input,
                name: None,
            });
            let messages_cloned = messages_clone.lock().await.clone();
            let chat_completion =
                ChatCompletion::builder(args.gpt_model.clone().unwrap().as_str(), messages_cloned)
                    .create();

            let chat_completion = select! {
                completion = chat_completion => completion,
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(CHATGPT_REQUEST_TIMEOUT)) => {
                    cb_sink.send(Box::new(move |s| {
                        outpln(
                            s,
                            Color::Dark(BaseColor::Red),
                            Color::Dark(BaseColor::Black),
                            "Error: Request timed out. Are you connected to AppGate?",
                        );
                    })).unwrap();
                    continue;
                }
            };

            match chat_completion {
                Ok(Ok(chat_completion)) => {
                    let returned_message = chat_completion.choices.first().unwrap().message.clone();

                    let message = returned_message.content.trim().to_owned();
                    cb_sink
                        .send(Box::new(move |s| {
                            outp(
                                s,
                                Color::Dark(BaseColor::Blue),
                                Color::Dark(BaseColor::Black),
                                "GPT: ",
                            );
                            outpln(
                                s,
                                Color::Dark(BaseColor::White),
                                Color::Dark(BaseColor::Black),
                                message.as_str(),
                            );
                        }))
                        .unwrap();

                    messages.lock().await.push(returned_message);
                }
                Err(e) => {
                    cb_sink
                        .send(Box::new(move |s| {
                            outpln(
                                s,
                                Color::Dark(BaseColor::Red),
                                Color::Dark(BaseColor::Black),
                                (String::from("Error: ") + e.to_string().as_str()).as_str(),
                            );
                        }))
                        .unwrap();
                }
                Ok(Err(e)) => {
                    cb_sink
                        .send(Box::new(move |s| {
                            outpln(
                                s,
                                Color::Dark(BaseColor::Red),
                                Color::Dark(BaseColor::Black),
                                (String::from("Error: ") + e.to_string().as_str()).as_str(),
                            );
                        }))
                        .unwrap();
                }
            };
        }
    });

    siv.run();
}

fn outpln<F, B>(s: &mut Cursive, front: F, back: B, message: &str)
where
    F: Into<ColorType>,
    B: Into<ColorType>,
{
    outp(s, front, back, (String::from(message) + "\n").as_str());
}

fn outp<F, B>(s: &mut Cursive, front: F, back: B, message: &str)
where
    F: Into<ColorType>,
    B: Into<ColorType>,
{
    let mut output = s.find_name::<TextView>("output").unwrap();
    let styled_text = StyledString::styled(message, ColorStyle::new(front, back));
    output.append(styled_text);
}
