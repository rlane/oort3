use log::{error, info};
use rand::Rng;

pub fn get_userid() -> String {
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .expect("failed to get local storage")
        .unwrap();
    match storage.get_item("/user/id") {
        Ok(Some(userid)) => userid,
        Ok(None) => {
            let mut rng = rand::thread_rng();
            let userid = format!("{:x}", rng.gen::<u64>());
            info!("Generated userid {}", &userid);
            if let Err(msg) = storage.set_item("/user/id", &userid) {
                error!("Failed to save userid: {:?}", msg);
            }
            userid
        }
        Err(msg) => {
            error!("Failed read userid: {:?}", msg);
            "unknown".to_string()
        }
    }
}

pub fn get_username(userid: &str) -> String {
    let mut rng: rand_chacha::ChaCha8Rng = rand_seeder::Seeder::from(userid).make_rng();
    petname::Petnames::default().generate(&mut rng, 2, "-")
}
