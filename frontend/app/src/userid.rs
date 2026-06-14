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
            let mut rng = rand::rng();
            let userid = format!("{:x}", rng.random::<u64>());
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

struct RngAdapter<R>(R);

impl<R: rand::RngCore + rand::TryRngCore> rand_core_06::RngCore for RngAdapter<R> {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.0.try_fill_bytes(dest).map_err(|_| {
            rand_core_06::Error::new("rng error")
        })
    }
}

pub fn generate_username(userid: &str) -> String {
    let rng: rand_chacha::ChaCha8Rng = rand_seeder::Seeder::from(userid).into_rng();
    let mut adapter = RngAdapter(rng);
    petname::Petnames::default().generate(&mut adapter, 2, "-")
}

pub fn get_username() -> String {
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .expect("failed to get local storage")
        .unwrap();
    match storage.get_item("/user/name") {
        Ok(Some(username)) => username,
        Ok(None) => {
            let username = generate_username(&get_userid());
            info!("Generated username {}", &username);
            if let Err(msg) = storage.set_item("/user/name", &username) {
                error!("Failed to save username: {:?}", msg);
            }
            username
        }
        Err(msg) => {
            error!("Failed read username: {:?}", msg);
            "unknown".to_string()
        }
    }
}
