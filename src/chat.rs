use futures::StreamExt;
use rig::{
    agent::{Agent, MultiTurnStreamItem, Text},
    cli_chatbot::AgentNotSet,
    completion::{CompletionModel, Message, Usage},
    message::{Reasoning, ToolCall},
    streaming::{StreamedAssistantContent, StreamingPrompt},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

pub struct SessionBuilder<A> {
    agent: A,
    multi_turn_depth: usize,
    show_usage: bool,
}

/// Set an empty builder
impl Default for SessionBuilder<AgentNotSet> {
    fn default() -> Self {
        SessionBuilder {
            agent: AgentNotSet,
            multi_turn_depth: 0,
            show_usage: false,
        }
    }
}

/// Set the Agent for a empty builder
impl SessionBuilder<AgentNotSet> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn agent<M>(self, agent: Agent<M>) -> SessionBuilder<Agent<M>>
    where
        M: CompletionModel + 'static,
    {
        SessionBuilder {
            agent,
            multi_turn_depth: self.multi_turn_depth,
            show_usage: self.show_usage,
        }
    }
}

/// Set the normal parameters
impl<A> SessionBuilder<A> {
    pub fn show_usage(self) -> Self {
        Self {
            show_usage: true,
            ..self
        }
    }

    pub fn multi_turn_depth(self, multi_turn_depth: usize) -> Self {
        Self {
            multi_turn_depth,
            ..self
        }
    }
}

/// Build the instance using a Builder that has the Agent configured
impl<M> SessionBuilder<Agent<M>>
where
    M: CompletionModel + 'static,
{
    pub fn build(self) -> Session<M> {
        Session {
            agent: self.agent,
            multi_turn_depth: self.multi_turn_depth,
            show_usage: self.show_usage,
        }
    }
}

pub struct Session<M>
where
    M: CompletionModel + 'static,
{
    agent: Agent<M>,
    multi_turn_depth: usize,
    show_usage: bool,
}

impl<M> Session<M>
where
    M: CompletionModel + 'static,
{
    pub async fn run(self) -> Result<(), anyhow::Error> {
        let mut chat_log = vec![];

        // Use async io system
        let mut output = BufWriter::new(tokio::io::stdout());
        let mut input = BufReader::new(tokio::io::stdin());
        output.write_all(b"Enter :q to quit\n").await?;

        loop {
            // Output the prompt character
            Self::user_start(&mut output).await?;

            let mut input_buf = String::new();
            input.read_line(&mut input_buf).await?;

            // Remove the newline character from the input
            let input = input_buf.trim();
            // Check for a command to exit
            if input == ":q" {
                break;
            }

            let mut usage = None;
            let mut response = String::new();

            let mut stream_response = self
                .agent
                .stream_prompt(input)
                .with_history(chat_log.clone())
                .multi_turn(self.multi_turn_depth)
                .await;

            Self::output_start(&mut output).await?;

            let mut is_reasoning = false;

            while let Some(chunk) = stream_response.next().await {
                match chunk {
                    Ok(MultiTurnStreamItem::StreamItem(StreamedAssistantContent::Text(Text {
                        text,
                    }))) => {
                        if text.contains("<think>") {
                            Self::output_reason_start(&mut output).await?;
                            is_reasoning = true;

                            continue;
                        }
                        if text.contains("</think>") {
                            Self::output_reason_end(&mut output).await?;
                            is_reasoning = false;

                            continue;
                        }

                        if !is_reasoning {
                            response.push_str(&text);
                        }
                        Self::output_text(text, &mut output).await?;
                    }
                    Ok(MultiTurnStreamItem::StreamItem(StreamedAssistantContent::Reasoning(
                        Reasoning { reasoning, .. },
                    ))) => {
                        let reasoning = reasoning.join("\n");

                        Self::output_reason_start(&mut output).await?;
                        Self::output_text(reasoning, &mut output).await?;
                        Self::output_reason_end(&mut output).await?;
                    }
                    Ok(MultiTurnStreamItem::StreamItem(StreamedAssistantContent::ToolCall(
                        ToolCall { function, .. },
                    ))) => {
                        let call_msg = format!(
                            "Call function {} with arguments {}...",
                            function.name, function.arguments
                        );

                        response.push_str(&call_msg);
                        Self::output_text(call_msg, &mut output).await?;
                    }
                    Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                        if self.show_usage {
                            usage = Some(r.usage());
                        }
                    }
                    Err(e) => {
                        Self::output_error(e, &mut output).await?;
                    }
                    _ => {}
                }
            }

            chat_log.push(Message::user(input));
            chat_log.push(Message::assistant(response.clone()));

            Self::output_finished(usage, &mut output).await?;

            tracing::info!("Response: \n{}\n", response);
        }

        Ok(())
    }

    async fn user_start(output: &mut BufWriter<tokio::io::Stdout>) -> std::io::Result<()> {
        output
            .write_all(b"\n\x1b[1;32m\xF0\x9F\x98\x80 User: \x1b[0m\n> ")
            .await?;
        output.flush().await?;

        Ok(())
    }

    async fn output_start(output: &mut BufWriter<tokio::io::Stdout>) -> std::io::Result<()> {
        output
            .write_all(b"\n\x1b[1;34m\xF0\x9F\xA4\x96 Agent: \x1b[0m\n")
            .await?;
        output.flush().await?;

        Ok(())
    }

    async fn output_text(
        content: impl std::fmt::Display,
        output: &mut BufWriter<tokio::io::Stdout>,
    ) -> std::io::Result<()> {
        output.write_all(content.to_string().as_bytes()).await?;
        output.flush().await?;

        Ok(())
    }

    async fn output_reason_start(output: &mut BufWriter<tokio::io::Stdout>) -> std::io::Result<()> {
        output
            .write_all("\n\x1b[1;90m🧠 Reasoning\n───────────────\n".as_bytes())
            .await?;

        output.flush().await?;
        Ok(())
    }

    async fn output_reason_end(output: &mut BufWriter<tokio::io::Stdout>) -> std::io::Result<()> {
        output
            .write_all("\n────────────────\x1b[0m".as_bytes())
            .await?;

        output.flush().await?;
        Ok(())
    }

    async fn output_finished(
        usage: Option<Usage>,
        output: &mut BufWriter<tokio::io::Stdout>,
    ) -> std::io::Result<()> {
        output.write_all(b"\n").await?;

        if let Some(usage) = usage {
            let usage_text = format!(
                "\n\x1b[1;33m📊 Token Usage\x1b[0m\n\
                 \x1b[1;30m────────────────\x1b[0m\n\
                 🔹 Input Tokens : {}\n\
                 🔹 Output Tokens: {}\n",
                usage.input_tokens, usage.output_tokens
            );
            output.write_all(usage_text.as_bytes()).await?;
        }

        output.flush().await?;
        Ok(())
    }

    async fn output_error(
        e: impl std::fmt::Display,
        output: &mut BufWriter<tokio::io::Stdout>,
    ) -> std::io::Result<()> {
        output
            .write_all(b"\x1b[1;31m\xE2\x9D\x8C ERROR: \x1b[0m")
            .await?;

        output.write_all(e.to_string().as_bytes()).await?;
        output.write_all(b"\n").await?;
        output.flush().await?;

        Ok(())
    }
}
