//! Tyractorsaur is a [configurable](./prelude/struct.TyractorsaurConfig.html) typed actor framework
//!
//! An [Actor](./prelude/trait.Actor.html) is an object with which you can only interact by sending predefined [messages](./prelude/trait.ActorMessage.html)
//!
//! Furthermore Actors are bound to a thread pool and can be moved between executions to any of the threads of said pool.
//!
//! [crossbeam-channel](https://github.com/crossbeam-rs/crossbeam) is used for all internal-messaging
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```rust
//! use tyractorsaur::prelude::*;
//! use std::process::exit;
//! use std::time::Duration;
//!
//! // define message
//! struct FooBar {}
//! impl ActorMessage for FooBar {}
//!
//! // define actor
//! struct HelloWorld {}
//! impl Actor for HelloWorld {}
//!
//! // setup required Factory
//! struct HelloWorldFactory {}
//! impl ActorFactory<HelloWorld> for HelloWorldFactory {
//!     fn new_actor(&self, _context: ActorContext<HelloWorld>) -> HelloWorld {
//!         HelloWorld {}
//!     }
//! }
//!
//! // each supported message has its own Handler implementation
//! // this is where the actual work is done
//! impl Handler<FooBar> for HelloWorld {
//!     fn handle(&mut self, _msg: FooBar, _context: &ActorContext<Self>) {
//!         println!("Message Received!");
//!     }
//! }
//!
//! fn main() {
//!     // create a new actor system with the default config
//!     let actor_config = TyractorsaurConfig::new().unwrap();
//!     let actor_system = ActorSystem::new(actor_config);
//!
//!     // create an actor and send it a message
//!     let factory = HelloWorldFactory {};
//!     let actor = actor_system
//!         .builder()
//!         .spawn("hello-world", factory)
//!         .unwrap();
//!     actor.send(FooBar {});
//!
//!     // cleanup
//!     actor.stop();
//!     actor_system.stop(Duration::from_secs(5));
//!     exit(actor_system.await_shutdown());
//! }
//! ```
//!
//! # Architecture
//!
//! ## User POV
//!
//! ```text
//!                             ┌──────────────────────┐
//!                             │                      │
//!                             │  TyractorsaurConfig  │
//!                             │                      │
//!                             └──────────┬───────────┘
//!                                        │
//!                                ┌───────▼───────┐
//!                                │               │
//!               ┌────────────────┤  ActorSystem  ├─────────────────┐
//!               │                │               │                 │
//!               │                └───────┬───────┘                 │
//!               │                        │                         │
//!      ┌────────▼─────────┐     ┌────────▼─────────┐     ┌─────────▼────────┐
//!      │                  │     │                  │     │                  │
//!      │  ActorBuilder A  │     │  ActorBuilder B  │     │  ActorBuilder N  │
//!      │                  │     │                  │     │                  │
//!      └┬─────────────────┘     └┬─────────────────┘     └┬─────────────────┘
//!       │                        │                        │
//!       │ ┌────────────┐         │  ┌────────────┐        │  ┌────────────┐
//!       │ │            │         │  │            │        │  │            │
//!       ├─►  Actor A1  │         ├──►  Actor B1  │        ├──►  Actor N1  │
//!       │ │            │         │  │            │        │  │            │
//!       │ └────────────┘         │  └────────────┘        │  └────────────┘
//!       │                        │                        │
//!       │ ┌────────────┐         │  ┌────────────┐        │  ┌────────────┐
//!       │ │            │         │  │            │        │  │            │
//!       ├─►  Actor A2  │         ├──►  Actor B2  │        ├──►  Actor N2  │
//!       │ │            │         │  │            │        │  │            │
//!       │ └────────────┘         │  └────────────┘        │  └────────────┘
//!       │                        │                        │
//!       │ ┌────────────┐         │  ┌────────────┐        │  ┌────────────┐
//!       │ │            │         │  │            │        │  │            │
//!       └─►  Actor An  │         └──►  Actor Bn  │        └──►  Actor Nn  │
//!         │            │            │            │           │            │
//!         └────────────┘            └────────────┘           └────────────┘
//!
//! ```
//!
mod actor;
mod config;
mod message;
mod routers;
mod system;

/// core components
pub mod prelude {
    pub use crate::actor::prelude::*;
    pub use crate::config::prelude::*;
    pub use crate::message::prelude::*;
    pub use crate::system::prelude::*;
}

/// collection of different router implementations
pub mod router {
    pub use crate::routers::prelude::*;
}
