use futures::{Future, StreamExt};
use log::{debug, info};
use rig::{
    agent::{Agent, MultiTurnStreamItem, Text},
    completion::{Chat, CompletionModel, Message, Usage},
    message::{Reasoning, ToolCall},
    streaming::{StreamedAssistantContent, StreamingPrompt},
};
use std::{io, pin::Pin};

use thiserror::Error;

/// Unified error type for ResponseSink
#[derive(Debug, Error)]
pub enum SinkError {
    /// Wrapper Lower Level I/O error
    #[error("I/O Eror: {0}")]
    Io(#[from] io::Error),

    /// Output logic error
    #[error("Output Error: {0}")]
    Output(String),

    /// Other uncategorized errors
    #[error("Unknown Error: {0}")]
    Other(String),
}

// The following structs are used to implement the type-state builder pattern.
// This pattern ensures that a `Session` is always created with a valid
// implementation of either `Chat` or `Agent`.

/// A placeholder struct indicating that no implementation has been provided yet.
pub struct NoImplProvided;

/// A struct that holds a `Chat` implementation.
pub struct ChatImpl<T>(T)
where
    T: Chat;

/// A struct that holds an `Agent` implementation.
pub struct AgentImpl<M>
where
    M: CompletionModel + 'static,
{
    agent: Agent<M>,
    multi_turn_depth: usize,
    show_usage: bool,
    usage: Usage,
}

/// A type-state builder for creating a `Session`.
/// The type `T` represents the state of the builder.
pub struct SessionBuilder<T>(T);

/// A wrapper that executes the configured chat or agent.
pub struct Session<T>(T);

/// A trait for abstracting the output of the chat session.
/// This allows the session to be used with different frontends (e.g., CLI, GUI).
pub trait ResponseSink {
    /// Called at the start of the chat session.
    fn chat_start(&mut self) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
    /// Called at the start of a user's query.
    fn user_start(&mut self) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
    /// Called at the start of the assistant's answer.
    fn output_start(&mut self) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;

    /// Outputs normal text from the assistant.
    fn output_text(
        &mut self,
        content: &(dyn std::fmt::Display + Send + Sync),
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;

    /// Called at the start of a reasoning block.
    fn output_reason_start(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
    /// Called at the end of a reasoning block.
    fn output_reason_end(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;

    /// Called at the end of the assistant's answer.
    fn output_finished(
        &mut self,
        usage: &Option<Usage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
    /// Called at the end of the chat session.
    fn chat_finished(&mut self)
    -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;

    /// Outputs an error message.
    fn output_error(
        &mut self,
        e: &(dyn std::fmt::Display + Send + Sync),
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
}

/// A trait for abstracting the input of the chat session.
pub trait InputSource {
    /// Reads a line of input from the user.
    fn read_input(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, SinkError>> + Send + '_>>;
}

/// A trait that abstracts the behavior of a chat session.
/// This allows for different implementations of the session (e.g., with an agent or a simple chat model).
pub trait ChatSession {
    /// Sends a request to the model and streams the response to the `ResponseSink`.
    fn request<'a, S: ResponseSink + 'a>(
        &'a mut self,
        prompt: &'a str,
        chat_log: Vec<Message>,
        sink: &'a mut S,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + 'a>>;
}

/// Implementation of `ChatSession` for a simple `Chat` model.
impl<T> ChatSession for ChatImpl<T>
where
    T: Chat,
{
    fn request<'a, S: ResponseSink + 'a>(
        &'a mut self,
        prompt: &'a str,
        chat_log: Vec<Message>,
        sink: &'a mut S,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + 'a>> {
        Box::pin(async move {
            let res = self.0.chat(prompt, chat_log).await?;
            sink.output_text(&res).await?;
            Ok(res)
        })
    }
}

/// A helper function to extract the incremental update from a string.
pub fn extract_increment_and_update<'a>(previous: &mut String, new: &'a str) -> &'a str {
    if new == previous {
        ""
    } else if let Some(delta) = new.strip_prefix(previous.as_str()) {
        previous.clear();
        previous.push_str(new);
        delta
    } else {
        previous.push_str(new);
        new
    }
}

/// Implementation of `ChatSession` for an `Agent`.
/// This implementation can handle reasoning and tool calls.
impl<M> ChatSession for AgentImpl<M>
where
    M: CompletionModel + 'static,
{
    fn request<'a, S: ResponseSink + 'a>(
        &'a mut self,
        prompt: &'a str,
        chat_log: Vec<Message>,
        sink: &'a mut S,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + 'a>> {
        Box::pin(async move {
            let mut response_stream = self
                .agent
                .stream_prompt(prompt)
                .with_history(chat_log)
                .multi_turn(self.multi_turn_depth)
                .await;

            let mut acc = String::new();
            let mut text_store = String::new();
            let mut reasoning_store = String::new();
            let mut is_reasoning = false;

            loop {
                let Some(chunk) = response_stream.next().await else {
                    break Ok(acc);
                };

                match chunk {
                    Ok(MultiTurnStreamItem::StreamItem(StreamedAssistantContent::Text(Text {
                        text,
                    }))) => {
                        let true_text = extract_increment_and_update(&mut text_store, &text);
                        acc.push_str(true_text);

                        if is_reasoning {
                            sink.output_reason_end().await?;
                            is_reasoning = false;
                        }
                        sink.output_text(&true_text).await?;
                    }
                    Ok(MultiTurnStreamItem::StreamItem(StreamedAssistantContent::Reasoning(
                        Reasoning { reasoning, .. },
                    ))) => {
                        let true_reasoning = extract_increment_and_update(
                            &mut reasoning_store,
                            reasoning.last().map(|s| s.as_str()).unwrap_or(""),
                        );

                        if !is_reasoning {
                            sink.output_reason_start().await?;
                            is_reasoning = true;
                        }
                        sink.output_text(&true_reasoning).await?;
                    }
                    Ok(MultiTurnStreamItem::StreamItem(StreamedAssistantContent::ToolCall(
                        ToolCall { function, .. },
                    ))) => {
                        let call_msg = format!(
                            "Call function {} with arguments {}...",
                            function.name, function.arguments
                        );
                        acc.push_str(&call_msg);
                        sink.output_text(&call_msg).await?;
                    }
                    Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                        if self.show_usage {
                            self.usage = r.usage();
                        }
                    }
                    Err(e) => {
                        sink.output_error(&e).await?;
                    }
                    _ => {}
                }
            }
        })
    }
}

// The following `impl` blocks define the type-state builder pipeline.
// The pipeline starts with `SessionBuilder<NoImplProvided>` and transitions
// to either `SessionBuilder<AgentImpl>` or `SessionBuilder<ChatImpl>`.
// Finally, the `build` method is called to create a `Session`.

impl Default for SessionBuilder<NoImplProvided> {
    fn default() -> Self {
        Self(NoImplProvided)
    }
}

/// Methods for the initial state of the `SessionBuilder`.
impl SessionBuilder<NoImplProvided> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an `Agent` to the session.
    pub fn agent<M: CompletionModel + 'static>(
        self,
        agent: Agent<M>,
    ) -> SessionBuilder<AgentImpl<M>> {
        SessionBuilder(AgentImpl {
            agent,
            multi_turn_depth: 1,
            show_usage: false,
            usage: Usage::default(),
        })
    }

    /// Adds a `Chat` model to the session.
    pub fn chat<T: Chat>(self, chatbot: T) -> SessionBuilder<ChatImpl<T>> {
        SessionBuilder(ChatImpl(chatbot))
    }
}

/// Methods for the `SessionBuilder` with a `Chat` model.
impl<T> SessionBuilder<ChatImpl<T>>
where
    T: Chat,
{
    /// Builds the `Session`.
    pub fn build(self) -> Session<ChatImpl<T>> {
        let SessionBuilder(chat_impl) = self;
        Session(chat_impl)
    }
}

/// Methods for the `SessionBuilder` with an `Agent`.
impl<M> SessionBuilder<AgentImpl<M>>
where
    M: CompletionModel + 'static,
{
    /// Sets the multi-turn depth for the agent.
    pub fn multi_turn_depth(self, multi_turn_depth: usize) -> Self {
        SessionBuilder(AgentImpl {
            multi_turn_depth,
            ..self.0
        })
    }

    /// Sets whether to show token usage.
    pub fn show_usage(self) -> Self {
        SessionBuilder(AgentImpl {
            show_usage: true,
            ..self.0
        })
    }

    /// Builds the `Session`.
    pub fn build(self) -> Session<AgentImpl<M>> {
        Session(self.0)
    }
}

/// Methods for the `Session`.
impl<T> Session<T>
where
    T: ChatSession,
{
    /// Runs the chat session.
    pub async fn run<S>(mut self, sink: &mut S) -> anyhow::Result<()>
    where
        S: ResponseSink + InputSource,
    {
        sink.chat_start().await?;
        info!("Session loop started");

        let mut chat_log = vec![];
        loop {
            sink.user_start().await?;

            if let Some(input) = sink.read_input().await? {
                debug!("Processing user prompt (len={})", input.len());

                sink.output_start().await?;
                let response = self.0.request(&input, chat_log.clone(), sink).await?;

                chat_log.push(Message::user(input));
                chat_log.push(Message::assistant(response));
            } else {
                info!("Input source exhausted; exiting session loop");
                break;
            }
        }

        sink.chat_finished().await?;
        info!("Session loop finished");

        Ok(())
    }
}
