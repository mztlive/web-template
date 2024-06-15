use casbin::{CoreApi, Enforcer, MgmtApi, RbacApi};

use mongodb::Database;
use tokio::sync::{
    mpsc::{self, Receiver},
    oneshot,
};

const MODEL: &str = r#"
[request_definition]
r = sub, action

[policy_definition]
p = sub, action

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.action == p.action || r.sub == "bozzasggmy"
"#;

use crate::database::{self};

use super::fetcher::{self, RBACRole, RBACRoleFetcher, RBACUser, RBACUserFetcher};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] database::errors::Error),

    #[error("Casbin error: {0}")]
    CasbinError(#[from] casbin::error::Error),

    #[error("Other error: {0}")]
    OtherError(String),

    #[error("Fetcher error: {0}")]
    FetcherError(#[from] fetcher::Error),
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::OtherError(err)
    }
}

/// command for rbac actor
pub enum Command {
    /// check permission
    CheckPermission {
        user: String,
        action: String,
        respond_to: oneshot::Sender<bool>,
    },
    Reset,
}

struct RbacActor<R: RBACRoleFetcher, U: RBACUserFetcher> {
    receiver: Receiver<Command>,
    database: Database,
    enforcer: Enforcer,
    role_fetcher: R,
    user_fetcher: U,
}

impl<R: RBACRoleFetcher, U: RBACUserFetcher> RbacActor<R, U> {
    pub fn new(
        receiver: Receiver<Command>,
        database: Database,
        enforcer: Enforcer,
        role_fetcher: R,
        user_fetcher: U,
    ) -> Self {
        RbacActor {
            receiver,
            database,
            enforcer,
            role_fetcher,
            user_fetcher,
        }
    }

    async fn load_polices(&mut self) -> Result<(), Error> {
        if let Err(err) = self.enforcer.clear_policy().await {
            println!("Failed to clear polices: {}", err);
        }

        let all_roles: Vec<Box<dyn RBACRole>> = self.role_fetcher.find_all(&self.database).await?;
        let all_users: Vec<Box<dyn RBACUser>> = self.user_fetcher.find_all(&self.database).await?;
        let roles_len = all_roles.len();
        let users_len = all_users.len();

        for role in all_roles {
            for policy in role.to_casbin_policy() {
                println!("policy: {:?}", policy);
                self.enforcer.add_policy(policy).await?;
            }
        }

        for user in all_users {
            self.enforcer
                .add_role_for_user(&user.account(), &user.role_name(), None)
                .await?;
        }

        println!("load {} roles and {} users", roles_len, users_len);

        Ok(())
    }

    async fn handle_message(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::CheckPermission {
                user,
                action,
                respond_to,
            } => {
                let is_ok = self.enforcer.enforce((user, action))?;

                respond_to.send(is_ok).map_err(|err| err.to_string())?;
            }

            Command::Reset => self.load_polices().await?,
        }

        Ok(())
    }
}

async fn run_actor<R: RBACRoleFetcher, U: RBACUserFetcher>(mut actor: RbacActor<R, U>) {
    while let Some(command) = actor.receiver.recv().await {
        if let Err(err) = actor.handle_message(command).await {
            println!("Failed to handle message: {}", err);
        }
    }
}

#[derive(Clone)]
pub struct RbacActorHandler {
    sender: mpsc::Sender<Command>,
}

impl RbacActorHandler {
    /// returns a handler for the [RbacActor]
    ///
    /// # Panics
    ///
    /// Panics if
    /// - casbin enforcer create failed.
    /// - load polices from database failed.
    pub async fn new<R, U>(database: Database, role_fetcher: R, user_fetcher: U) -> Self
    where
        R: RBACRoleFetcher + 'static,
        U: RBACUserFetcher + 'static,
    {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let casbin_enforcer = create_enforcer().await.unwrap();
        let mut actor = RbacActor::new(
            receiver,
            database,
            casbin_enforcer,
            role_fetcher,
            user_fetcher,
        );
        actor.load_polices().await.unwrap();

        tokio::spawn(run_actor(actor));

        RbacActorHandler { sender }
    }

    pub async fn check_permission(&self, user: String, action: String) -> Result<bool, String> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(Command::CheckPermission {
                user,
                action,
                respond_to,
            })
            .await
            .map_err(|err| format! {"cannot send message to rbac actor: {0}", err})?;

        let result = response
            .await
            .map_err(|err| format! {"cannot receive response from rbac actor: {0}", err})?;

        Ok(result)
    }

    pub async fn reset(&self) -> Result<(), String> {
        self.sender
            .send(Command::Reset)
            .await
            .map_err(|err| format! {"cannot reset rbac polices: {0}", err})?;
        Ok(())
    }
}

async fn create_enforcer() -> Result<Enforcer, Error> {
    let model = casbin::DefaultModel::from_str(MODEL).await?;
    let adapter = casbin::MemoryAdapter::default();
    let e = Enforcer::new(model, adapter).await?;
    Ok(e)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_enforcer_model() {
        let mut enforcer = create_enforcer().await.unwrap();
        let policy = vec!["admin".to_string(), "read".to_string()];

        let user = "zhangsan";

        enforcer.add_policy(policy).await.unwrap();
        println!("{:?}", enforcer.get_all_policy());

        enforcer
            .add_role_for_user(user, "admin", None)
            .await
            .unwrap();

        println!("{:?}", enforcer.get_all_roles());

        let is_ok = enforcer.enforce((user, "read")).unwrap();
        assert_eq!(is_ok, true);

        let is_false = enforcer.enforce((user, "write")).unwrap();
        assert_eq!(is_false, false);

        // user not in the role
        let is_false = enforcer.enforce(("test", "read")).unwrap();
        assert_eq!(is_false, false);
    }
}
