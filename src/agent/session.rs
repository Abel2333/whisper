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

/// Type-state builder that ensures an agent/chat implementation is provided.
pub struct SessionBuilder<T>(T);

/// Wrapper that executes the configured chat or agent.
pub struct Session<T>(T);

/// Trait to abstract display
pub trait ResponseSink {
    /// Output the string to indicate the start of the chat
    fn chat_start(&mut self) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
    /// Output the string to indicate the start of user's query
    fn user_start(&mut self) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
    /// Output the string to indicate the start of assistant's answer
    fn output_start(&mut self) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;

    /// Output the normal text
    fn output_text(
        &mut self,
        content: &(dyn std::fmt::Display + Send + Sync),
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;

    /// Output the start of reasoning content
    fn output_reason_start(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
    /// Output the end of reasoning content
    fn output_reason_end(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;

    /// Output the end of assistant's answer
    fn output_finished(
        &mut self,
        usage: &Option<Usage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
    /// Output the end of chat
    fn chat_finished(&mut self)
    -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;

    /// Output the error
    fn output_error(
        &mut self,
        e: &(dyn std::fmt::Display + Send + Sync),
    ) -> Pin<Box<dyn Future<Output = Result<(), SinkError>> + Send + '_>>;
}

/// Trait to abstract get input
pub trait InputSource {
    fn read_input(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, SinkError>> + Send + '_>>;
}

/// Trait to abstract message behavior
pub trait ChatSession {
    /// Send request and display the streaming answer within response sink
    fn request<'a, S: ResponseSink + 'a>(
        &'a mut self,
        prompt: &'a str,
        chat_log: Vec<Message>,
        sink: &'a mut S,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + 'a>>;
}

/// Could only chat with assistant.
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

/// Could chat, reasoning, call tools
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

/// Type-state builder pipeline:
/// `Builder<NoImplProvided>` -> `Builder<AgentImpl>` -> ... -> `Session<AgentImpl>`
///
/// or
///
/// `Builder<NoImplProvided>` -> `Builder<ChatImpl>` -> `Session<ChatImpl>`
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
        sink.chat_start().await?;
        info!("Session loop started");

        let mut chat_log = vec![];
        loop {
            sink.user_start().await?;

            if let Some(input) = sink.read_input().await? {
                debug!("Processing user prompt (len={})", input.len());
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
