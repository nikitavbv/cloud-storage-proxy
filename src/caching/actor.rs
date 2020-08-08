use actix::{Context, Handler, Actor, Message};

pub struct PutCacheEntry {
    pub bucket: String,
    pub key: String,
    pub body: Vec<u8>
}

pub struct GetCacheEntry {
    pub bucket: String,
    pub key: String
}

impl Message for PutCacheEntry {
    type Result = ();
}

impl Message for GetCacheEntry {
    type Result = Vec<u8>;
}

pub struct CachingActor;

impl Actor for CachingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("local cache actor is alive");
    }
}

impl Handler<PutCacheEntry> for CachingActor {
    type Result = ();

    fn handle(&mut self, msg: PutCacheEntry, _: &mut Context<Self>) -> Self::Result {
        println!("put cache entry message handle");
    }
}

impl Handler<GetCacheEntry> for CachingActor {
    type Result = Box<Vec<u8>>;

    fn handle(&mut self, msg: GetCacheEntry, _: &mut Context<Self>) -> Self::Result {
        println!("get cache entry message handle");
        Box::new(vec![])
    }
}
