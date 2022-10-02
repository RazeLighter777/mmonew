#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![allow(incomplete_features)]
#![feature(unsize)]
#![feature(specialization)]
#![allow(unused)]
#![deny(warnings)]

use clap::Parser;
use futures::future::join_all;
use server_world::ServerWorld;
use tokio::time::Instant;

mod args;
mod change_tracker;
mod query;
mod server_world;
#[tokio::main]
async fn main() -> Result<(), server_world::ServerWorldError> {
    let args = args::Args::parse();

    let w = ServerWorld::new("dockercuck.prizrak.me","test","C:\\Users\\justin.suess\\Code\\mmonew\\raws").await?;
    let now = Instant::now();
    for i in 0..1000 {
        //w.write_component(mmolib::entity_id::EntityId::new(), &mmolib::position::Position { x : 1, y : 2 }).await?;
        // let mut q = Query::new(w.clone());
        // q.add_union::<mmolib::position::Position>();
        // let res = q.execute().await?;
        // for x in res.iter() {
        //     println!("Iterated through entity");
        //     let pos = x.get::<mmolib::position::Position>().await?;
        // }
        w.write_component(mmolib::entity_id::EntityId::new(), &mmolib::position::Position { x : 1, y : 2 }).await?;
   }
   let elapsed_time = now.elapsed();
   println!("Running writes() took {} millis.", elapsed_time.as_millis());

   let now = Instant::now();
   w.write_all_changes().await?;
   let elapsed_time = now.elapsed();
   println!("Running slow_function() took {} millis.", elapsed_time.as_millis());

   //join_all((0..1000).map(|x| { })).await;
   Ok(())

}
