use crate::actor::actor::Actor;
use std::sync::{Arc, RwLock};
use crate::config::tyractorsaur_config::{TyractorsaurConfig, DEFAULT_POOL};
use crate::config::pool_config::ThreadPoolConfig;
use crossbeam_channel::{unbounded, bounded};
use crate::actor::executor::{ExecutorTrait, Executor};
use std::time::{Instant, Duration};
use std::thread::sleep;
use crate::message::serialized_message::SerializedMessage;
use crate::actor::actor_builder::ActorBuilder;
use crate::actor::actor_config::ActorConfig;
use crate::actor::actor_wrapper::ActorWrapper;
use std::panic::UnwindSafe;
use crate::actor::mailbox::Mailbox;
use crate::actor::context::Context;
use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_factory::ActorFactory;
use crate::system::system_state::SystemState;
use crate::system::thread_pool_manager::ThreadPoolManager;
use crate::system::wakeup_manager::WakeupManager;
use std::sync::atomic::AtomicBool;

pub struct WakeupMessage {
    pub iteration: usize,
    pub actor_address: ActorAddress,
}


#[derive(Clone)]
pub struct ActorSystem {
    state: SystemState,
    thread_pool_manager: ThreadPoolManager,
    wakeup_manager: WakeupManager,
    name: String,
    config: Arc<TyractorsaurConfig>,
}

impl ActorSystem {
    pub fn new(config: TyractorsaurConfig) -> Self {
        let thread_pool_config = config.thread_pool.clone();
        let system = ActorSystem {
            state: SystemState::new(),
            thread_pool_manager: ThreadPoolManager::new(),
            wakeup_manager: WakeupManager::new(),
            name: config.global.name.clone(),
            config: Arc::new(config.clone()),
        };

        for (key, value) in thread_pool_config.config.iter() {
            system.add_pool_with_config(key, value.clone());
        }
        system.start();
        system
    }

    pub fn add_pool(&self, name: &str) {
        let default_config = self.config.thread_pool.config.get(DEFAULT_POOL).unwrap();
        let config = self
            .config
            .thread_pool
            .config
            .get(name)
            .unwrap_or(default_config);
        self.add_pool_with_config(name, config.clone());
    }

    pub fn add_pool_with_config(&self, name: &str, thread_pool_config: ThreadPoolConfig) {
        self.thread_pool_manager.add_pool_with_config(name, thread_pool_config);
    }

    fn start(&self) {
        let s = self.clone();
        std::thread::spawn(move || s.manage_threads());
        let s = self.clone();
        std::thread::spawn(move || s.wake());
    }

    fn wake(&self) {
        self.wakeup_manager.manage(self.state.clone(), self.thread_pool_manager.clone());

    }

    fn manage_threads(&self) {
        self.thread_pool_manager.start(self.state.clone(), self.wakeup_manager.clone());
    }

    pub fn send_to_address(&self, address: &ActorAddress, msg: SerializedMessage) {
        self.state.send_to_address(address, msg);
    }

    pub fn builder(&self, name: impl Into<String>) -> ActorBuilder {
        ActorBuilder::new(self.clone(), name.into())
    }

    pub fn spawn<A, P>(&self, actor_props: P, actor_config: ActorConfig) -> ActorWrapper<A>
    where
        A: Actor + UnwindSafe + 'static,
        P: ActorFactory<A> + 'static,
    {
        let (sender, receiver) = if actor_config.mailbox_size == 0 {
            unbounded()
        } else {
            bounded(actor_config.mailbox_size)
        };

        let mailbox = Mailbox {
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_sleeping: Arc::new(AtomicBool::new(true)),
            msg_in: sender,
        };
        let actor_address = ActorAddress {
            actor: actor_config.actor_name.clone(),
            system: self.name.clone(),
            pool: actor_config.pool_name.clone(),
            remote: String::from("local"),
        };
        let actor_ref = ActorWrapper::new(mailbox.clone(), actor_address.clone(), self.wakeup_manager.clone());

        let context = Context {
            system: self.clone(),
            actor_ref: actor_ref.clone(),
        };
        let actor = actor_props.new_actor(context);
        let actor_handler = Executor::new(
            actor_props,
            actor_config,
            mailbox.clone(),
            receiver,
            self.clone(),
            self.name.clone(),
            actor_ref.clone(),
        );

        self.state.add_actor(actor_address.clone(), Arc::new(actor));
        self.wakeup_manager.add_sleeping_actor(actor_handler.get_address(), Arc::new(RwLock::new(actor_handler)));
        actor_ref
    }

    pub fn stop(&self, graceful_termination_timeout: Duration) {
        if self.state.is_stopping() {
            return;
        }
        self.state.stop();
        let s = self.clone();
        std::thread::spawn(move || s.shutdown(graceful_termination_timeout));
    }

    fn shutdown(&self, timeout: Duration) {
        let now = Instant::now();
        let mut is_forced_stop = false;
        while self.state.get_actor_count() != 0 {
            if now.elapsed() >= timeout {
                is_forced_stop = true;
                break;
            }
            sleep(Duration::from_secs(1));
        }
        self.state.finalize_stop(is_forced_stop);
    }

    pub fn await_shutdown(&self) -> i32 {
        while !self.state.is_stopped() {
            sleep(Duration::from_secs(1));
        }
        self.state.is_force_stopped() as i32
    }

    pub fn get_config(&self) -> &TyractorsaurConfig {
        &self.config
    }

}