use snowflake::SnowflakeIdGenerator;
use tokio::sync::{mpsc, oneshot};

pub enum Command {
    NextID { respond_to: oneshot::Sender<String> },
}

struct IDGeneratorActor {
    engine: SnowflakeIdGenerator,
    receiver: mpsc::Receiver<Command>,
}

impl IDGeneratorActor {
    fn new(receiver: mpsc::Receiver<Command>) -> Self {
        let engine = SnowflakeIdGenerator::new(1, 1);
        IDGeneratorActor { engine, receiver }
    }

    fn handle_message(&mut self, command: Command) {
        match command {
            Command::NextID { respond_to } => {
                let id = self.engine.generate().to_string();
                if let Err(err) = respond_to.send(id) {
                    println!("Failed to send id: {}", err);
                }
            }
        }
    }
}

async fn run_actor(mut actor: IDGeneratorActor) {
    while let Some(command) = actor.receiver.recv().await {
        actor.handle_message(command);
    }
}

#[derive(Clone)]
pub struct IDGeneratorHandler {
    sender: mpsc::Sender<Command>,
}

impl IDGeneratorHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        tokio::spawn(run_actor(IDGeneratorActor::new(receiver)));
        IDGeneratorHandler { sender }
    }

    pub async fn next_id(&self) -> Result<String, String> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(Command::NextID { respond_to })
            .await
            .map_err(|err| err.to_string())?;

        let id = response.await.map_err(|err| err.to_string())?;

        Ok(id)
    }
}
