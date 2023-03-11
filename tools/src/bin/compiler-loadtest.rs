use goose::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("compiler_loadtest=info"),
    )
    .init();

    GooseAttack::initialize()?
        .register_scenario(
            scenario!("compile")
                .set_wait_time(Duration::from_secs(2), Duration::from_secs(5))?
                .register_transaction(transaction!(compile)),
        )
        .execute()
        .await?;

    Ok(())
}

async fn compile(user: &mut GooseUser) -> TransactionResult {
    let _goose = user
        .post("/compile", include_str!("../../../shared/ai/reference.rs"))
        .await?;

    Ok(())
}
