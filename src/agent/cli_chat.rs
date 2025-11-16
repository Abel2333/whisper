use std::{future::Future, pin::Pin};

use crate::agent::session::{self, InputSource, ResponseSink};
use log::{debug, info};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, stdin, stdout};

/// Simple stdin/stdout frontend that feeds the agent session loop.
pub struct CliFrontend {
    input: BufReader<tokio::io::Stdin>,
    output: BufWriter<tokio::io::Stdout>,
}

impl CliFrontend {
    pub fn new() -> Self {
        Self {
            input: BufReader::new(stdin()),
            output: BufWriter::new(stdout()),
        }
    }
}

impl Default for CliFrontend {
    fn default() -> Self {
        Self::new()
    }
}

impl InputSource for CliFrontend {
    fn read_input(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, session::SinkError>> + Send + '_>> {
        Box::pin(async move {
            let mut buf = String::new();
            self.input.read_line(&mut buf).await?;

            let line = buf.trim().to_string();

            if line == ":q" {
                info!("User requested to exit chat");
                return Ok(None);
            }

            debug!("Captured user input with {} characters", line.len());
            Ok(Some(line))
        })
    }
}

impl ResponseSink for CliFrontend {
    fn chat_start(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), session::SinkError>> + Send + '_>> {
        Box::pin(async move {
            self.output
                .write_all(
                    "\n\x1b[1;36m🌊 Session started successfully\x1b[0m\n\
      \x1b[1;90m──────────────────────────────────────\x1b[0m\n\
      Ready to dive in! Type \x1b[1;33m:q\x1b[0m anytime to exit the flow.\n"
                        .as_bytes(),
                )
                .await?;
            self.output.flush().await?;
            info!("CLI session initialized");
            Ok(())
        })
    }

    fn user_start(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), session::SinkError>> + Send + '_>> {
        Box::pin(async move {
            self.output
                .write_all(b"\n\n\x1b[1;32m\xF0\x9F\x98\x80 User: \x1b[0m\n> ")
                .await?;
            self.output.flush().await?;
            Ok(())
        })
    }

    fn output_start(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), session::SinkError>> + Send + '_>> {
        Box::pin(async move {
            self.output
                .write_all(b"\n\x1b[1;32m\xF0\x9F\x98\x80 User: \x1b[0m\n> ")
                .await?;
            self.output.flush().await?;
            Ok(())
        })
    }

    fn output_text(
        &mut self,
        content: &(dyn std::fmt::Display + Send + Sync),
    ) -> Pin<Box<dyn Future<Output = Result<(), session::SinkError>> + Send + '_>> {
        let text = content.to_string();
        Box::pin(async move {
            self.output.write_all(text.as_bytes()).await?;
            self.output.flush().await?;
            Ok(())
        })
    }

    fn output_reason_start(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), session::SinkError>> + Send + '_>> {
        Box::pin(async move {
            self.output
                .write_all("\n\x1b[1;90m🧠 Reasoning\n───────────────\n".as_bytes())
                .await?;
            self.output.flush().await?;
            Ok(())
        })
    }

    fn output_reason_end(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), session::SinkError>> + Send + '_>> {
        Box::pin(async move {
            self.output
                .write_all("\n────────────────\x1b[0m\n".as_bytes())
                .await?;
            self.output.flush().await?;
            Ok(())
        })
    }

    fn output_finished(
        &mut self,
        usage: &Option<rig::completion::Usage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), session::SinkError>> + Send + '_>> {
        let usage = *usage;
        Box::pin(async move {
            self.output.write_all(b"\n").await?;

            if let Some(usage) = usage {
                let usage_text = format!(
                    "\n\x1b[1;33m📊 Token Usage\x1b[0m\n\
                     \x1b[1;30m────────────────\x1b[0m\n\
                     🔹 Input Tokens : {}\n\
                     🔹 Output Tokens: {}\n",
                    usage.input_tokens, usage.output_tokens
                );
                self.output.write_all(usage_text.as_bytes()).await?;
            }

            self.output.flush().await?;
            Ok(())
        })
    }

    fn chat_finished(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), session::SinkError>> + Send + '_>> {
        Box::pin(async move {
            self.output
                .write_all(b"Session closed successfully. Wishing you a pleasant day ahead.\n")
                .await?;
            self.output.flush().await?;
            info!("CLI session closed");
            Ok(())
        })
    }

    fn output_error(
        &mut self,
        e: &(dyn std::fmt::Display + Send + Sync),
    ) -> Pin<Box<dyn Future<Output = Result<(), session::SinkError>> + Send + '_>> {
        let msg = e.to_string();
        Box::pin(async move {
            self.output
                .write_all(b"\x1b[1;31m\xE2\x9D\x8C ERROR: \x1b[0m")
                .await?;
            self.output.write_all(msg.as_bytes()).await?;
            self.output.write_all(b"\n").await?;
            self.output.flush().await?;
            info!("CLI sink encountered error: {msg}");
            Ok(())
        })
    }
}
