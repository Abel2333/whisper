use crate::agent::session::{self, InputSource, ResponseSink};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

struct CliFrontend {
    input: BufReader<tokio::io::Stdin>,
    output: BufWriter<tokio::io::Stdout>,
}

impl InputSource for CliFrontend {
    async fn read_input(&mut self) -> Result<Option<String>, session::SinkError> {
        let mut buf = String::new();
        self.input.read_line(&mut buf).await?;

        let line = buf.trim().to_string();

        if line == ":q" {
            return Ok(None);
        }

        Ok(Some(line))
    }
}

impl ResponseSink for CliFrontend {
    async fn chat_start(&mut self) -> Result<(), session::SinkError> {
        self.output.write_all(b"Enter `:q` to quit\n").await?;

        Ok(())
    }

    async fn user_start(&mut self) -> Result<(), session::SinkError> {
        self.output
            .write_all(b"\n\x1b[1;32m\xF0\x9F\x98\x80 User: \x1b[0m\n> ")
            .await?;
        self.output.flush().await?;

        Ok(())
    }

    async fn output_start(&mut self) -> Result<(), session::SinkError> {
        self.output
            .write_all(b"\n\x1b[1;32m\xF0\x9F\x98\x80 User: \x1b[0m\n> ")
            .await?;
        self.output.flush().await?;

        Ok(())
    }

    async fn output_text(
        &mut self,
        content: &(dyn std::fmt::Display + Send + Sync),
    ) -> Result<(), session::SinkError> {
        self.output
            .write_all(content.to_string().as_bytes())
            .await?;
        self.output.flush().await?;

        Ok(())
    }

    async fn output_reason_start(&mut self) -> Result<(), session::SinkError> {
        self.output
            .write_all("\n\x1b[1;90m🧠 Reasoning\n───────────────\n".as_bytes())
            .await?;
        self.output.flush().await?;

        Ok(())
    }

    async fn output_reason_end(&mut self) -> Result<(), session::SinkError> {
        self.output
            .write_all("\n────────────────\x1b[0m".as_bytes())
            .await?;

        self.output.flush().await?;

        Ok(())
    }

    async fn output_finished(
        &mut self,
        usage: &Option<rig::completion::Usage>,
    ) -> Result<(), session::SinkError> {
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
    }

    async fn chat_finished(&mut self) -> Result<(), session::SinkError> {
        self.output
            .write_all(b"Session closed successfully. Wishing you a pleasant day ahead.")
            .await?;
        self.output.flush().await?;

        Ok(())
    }

    async fn output_error(
        &mut self,
        e: &(dyn std::fmt::Display + Send + Sync),
    ) -> Result<(), session::SinkError> {
        self.output
            .write_all(b"\x1b[1;31m\xE2\x9D\x8C ERROR: \x1b[0m")
            .await?;

        self.output.write_all(e.to_string().as_bytes()).await?;
        self.output.write_all(b"\n").await?;
        self.output.flush().await?;

        Ok(())
    }
}
