use futures::StreamExt;
use rig::{
    agent::{Agent, MultiTurnStreamItem, Text},
    completion::{Chat, CompletionModel, Message, Usage},
    message::{Reasoning, ToolCall},
    streaming::{StreamedAssistantContent, StreamingPrompt},
};

use std::io;
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

    /// 其他未分类错误
    #[error("Unkown Error: {0}")]
    Other(String),
}

/// Nothing
pub struct NoImplProvided;

/// Could chat
pub struct ChatImpl<T>(T)
where
    T: Chat;

/// An agent
pub struct AgentImpl<M>
where
    M: CompletionModel + 'static,
{
    agent: Agent<M>,
    multi_turn_depth: usize,
    show_usage: bool,
    usage: Usage,
}

pub struct SessionBuilder<T>(T);

pub struct Session<T>(T);

/// Trait to abstract display
pub trait ResponseSink {
    /// Output the string to indicate the start of the chat
    async fn chat_start(&mut self) -> Result<(), SinkError>;
    /// Output the string to indicate the start of user's query
    async fn user_start(&mut self) -> Result<(), SinkError>;
    /// Output the string to indicate the start of assistant's answer
    async fn output_start(&mut self) -> Result<(), SinkError>;

    /// Output the normal text
    async fn output_text(
        &mut self,
        content: &(dyn std::fmt::Display + Send + Sync),
    ) -> Result<(), SinkError>;

    /// Output the start of reasoning content
    async fn output_reason_start(&mut self) -> Result<(), SinkError>;
    /// Output the end of reasoning content
    async fn output_reason_end(&mut self) -> Result<(), SinkError>;

    /// Output the end of assistant's answer
    async fn output_finished(&mut self, usage: &Option<Usage>) -> Result<(), SinkError>;
    /// Output the end of chat
    async fn chat_finished(&mut self) -> Result<(), SinkError>;

    /// Output the error
    async fn output_error(
        &mut self,
        e: &(dyn std::fmt::Display + Send + Sync),
    ) -> Result<(), SinkError>;
}

/// Trait to abstract get input
pub trait InputSource {
    async fn read_input(&mut self) -> Result<Option<String>, SinkError>;
}

/// Trait to abstract message behavior
trait ChatSession {
    /// Send request and display the streaming answer within response sink
    async fn request<S: ResponseSink>(
        &mut self,
        prompt: &str,
        chat_log: Vec<Message>,
        sink: &mut S,
    ) -> anyhow::Result<String>;

    /// Show usage or not
    fn show_usage(&self) -> bool {
        false
    }

    /// Get the usage
    fn usage(&self) -> Option<Usage> {
        None
    }
}

/// Could only chat with assistant.
impl<T> ChatSession for ChatImpl<T>
where
    T: Chat,
{
    async fn request<S: ResponseSink>(
        &mut self,
        prompt: &str,
        chat_log: Vec<Message>,
        sink: &mut S,
    ) -> anyhow::Result<String> {
        let res = self.0.chat(prompt, chat_log).await?;
        sink.output_text(&res);

        Ok(res)
    }
}

/// Could chat, reasoning, call tools
impl<M> ChatSession for AgentImpl<M>
where
    M: CompletionModel + 'static,
{
    async fn request<S: ResponseSink>(
        &mut self,
        prompt: &str,
        chat_log: Vec<Message>,
        sink: &mut S,
    ) -> anyhow::Result<String> {
        let mut response_stream = self
            .agent
            .stream_prompt(prompt)
            .with_history(chat_log)
            .multi_turn(self.multi_turn_depth)
            .await;

        let mut acc = String::new();

        let mut is_reasoning = false;
        loop {
            let Some(chunk) = response_stream.next().await else {
                break Ok(acc);
            };

            // Process every kind of chunk
            match chunk {
                // Normal text
                Ok(MultiTurnStreamItem::StreamItem(StreamedAssistantContent::Text(Text {
                    text,
                }))) => {
                    if text.contains("<think>") {
                        sink.output_reason_start().await?;
                        is_reasoning = true;

                        continue;
                    }
                    if text.contains("</think>") {
                        sink.output_reason_end().await?;
                        is_reasoning = false;

                        continue;
                    }

                    if !is_reasoning {
                        acc.push_str(&text);
                    }
                    sink.output_text(&text).await?;
                }
                // Reasoning
                Ok(MultiTurnStreamItem::StreamItem(StreamedAssistantContent::Reasoning(
                    Reasoning { reasoning, .. },
                ))) => {
                    let reasoning = reasoning.join("\n");

                    sink.output_reason_start().await?;
                    sink.output_text(&reasoning).await?;
                    sink.output_reason_end().await?;
                }
                // ToolCall
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
                // Final
                Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                    if self.show_usage {
                        self.usage = r.usage();
                    }
                }
                Err(e) => {
                    sink.output_error(&e);
                }
                _ => {}
            }
        }
    }

    fn show_usage(&self) -> bool {
        self.show_usage
    }

    fn usage(&self) -> Option<Usage> {
        Some(self.usage)
    }
}

/// type-state builder
/// Builder<NoImplProvided> -> Builder<AgentImpl> -> . -> Session<AgentImpl>
/// or
/// Builder<NoImplProvided> -> Builder<ChatImpl> -> Session<ChatImpl>
impl Default for SessionBuilder<NoImplProvided> {
    fn default() -> Self {
        Self(NoImplProvided)
    }
}

/// Builder from empty
impl SessionBuilder<NoImplProvided> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an agent to Session
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

    /// Add a chat to Session
    pub fn chat<T: Chat>(self, chatbot: T) -> SessionBuilder<ChatImpl<T>> {
        SessionBuilder(ChatImpl(chatbot))
    }
}

impl<T> SessionBuilder<ChatImpl<T>>
where
    T: Chat,
{
    pub fn build(self) -> Session<ChatImpl<T>> {
        let SessionBuilder(chat_impl) = self;
        Session(chat_impl)
    }
}

impl<M> SessionBuilder<AgentImpl<M>>
where
    M: CompletionModel + 'static,
{
    pub fn multi_turn_depth(self, multi_turn_depth: usize) -> Self {
        SessionBuilder(AgentImpl {
            multi_turn_depth,
            ..self.0
        })
    }

    pub fn show_usage(self) -> Self {
        SessionBuilder(AgentImpl {
            show_usage: true,
            ..self.0
        })
    }

    pub fn build(self) -> Session<AgentImpl<M>> {
        Session(self.0)
    }
}

impl<T> Session<T>
where
    T: ChatSession,
{
    pub async fn run<S>(mut self, sink: &mut S) -> anyhow::Result<()>
    where
        S: ResponseSink + InputSource,
    {
        let mut chat_log = vec![];
        loop {
            sink.user_start().await?;

            if let Some(input) = sink.read_input().await? {
                let response = self.0.request(&input, chat_log.clone(), sink).await?;
                chat_log.push(Message::user(input));
                chat_log.push(Message::assistant(response));
            } else {
                break;
            }
        }

        sink.chat_finished().await?;

        Ok(())
    }
}
