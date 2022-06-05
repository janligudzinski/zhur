use bcrypt::bcrypt;
use sled::Db;
const BCRYPT_COST: u32 = 10;
pub struct UserRepo {
    db: Db,
}

impl UserRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
    fn user_exists(&self, name: &str) -> bool {
        self.db.contains_key(name).unwrap()
    }
    pub fn register_user(&self, name: &str, password: &str) -> bool {
        if self.user_exists(name) {
            return false;
        }
        let password = bcrypt::hash(password, BCRYPT_COST).unwrap();
        self.db.insert(name, password.as_bytes()).unwrap();
        true
    }
    pub fn login(&self, name: &str, password: &str) -> bool {
        if !self.user_exists(name) {
            return false;
        }
        let hash = self
            .db
            .get(name)
            .unwrap()
            .map(|bytes| String::from_utf8(bytes.to_vec()).unwrap())
            .unwrap();
        bcrypt::verify(password, &hash).unwrap()
    }
    pub fn change_password(&self, name: &str, old_password: &str, new_password: &str) -> bool {
        if !self.user_exists(name) || !self.login(name, old_password) {
            return false;
        }
        let hash = bcrypt::hash(new_password, BCRYPT_COST).unwrap();
        self.db.insert(name, hash.as_bytes()).unwrap();
        true
    }
}
