use super::{
    config::Config,
    connect,
    error::{Error, Result},
    event::Event,
    session::Session,
    socket_forwarder::SocketForwarder,
    stage::Stage,
};
use crate::{
    event::{DispatchEvent, GatewayEvent},
    listener::Listeners,
};
use dawn_model::gateway::payload::{
    identify::{Identify, IdentifyInfo, IdentifyProperties},
    resume::Resume,
};
use futures_channel::mpsc::UnboundedReceiver;
use futures_util::stream::StreamExt;
use log::{trace, warn};
use serde::Serialize;
use std::{env::consts::OS, mem, ops::Deref, sync::Arc};
use tokio_tungstenite::tungstenite::Message;

/// Runs in the background and processes incoming events, and then broadcasts
/// to all listeners.
pub struct ShardProcessor {
    pub config: Arc<Config>,
    pub listeners: Arc<Listeners<Event>>,
    pub properties: IdentifyProperties,
    pub rx: UnboundedReceiver<Message>,
    pub session: Arc<Session>,
}

impl ShardProcessor {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let properties = IdentifyProperties::new("dawn.rs", "dawn.rs", OS, "", "");

        let url = "wss://gateway.discord.gg";

        let stream = connect::connect(url).await?;
        let (mut forwarder, rx, tx) = SocketForwarder::new(stream);
        tokio_executor::spawn(async move {
            let _ = forwarder.run().await;
        });

        Ok(Self {
            config,
            listeners: Arc::new(Listeners::default()),
            properties,
            rx,
            session: Arc::new(Session::new(tx)),
        })
    }

    pub async fn run(mut self) {
        let mut remove_listeners = Vec::new();

        loop {
            // Returns None when the socket forwarder has ended.
            let msg = self.rx.next().await.unwrap();

            let gateway_event: GatewayEvent = match msg {
                Message::Binary(bytes) => {
                    trace!("Payload: {}", String::from_utf8_lossy(&bytes));

                    serde_json::from_slice(&bytes).unwrap()
                },
                Message::Close(_) => {
                    self.reconnect().await;

                    continue;
                },
                Message::Ping(_) | Message::Pong(_) => continue,
                Message::Text(text) => {
                    trace!("Payload: {}", text);

                    serde_json::from_str(&text).unwrap()
                },
            };

            self.process(&gateway_event).await.unwrap();
            let event = Event::from(gateway_event);

            let mut listeners = self.listeners.listeners.lock().await;

            for (id, listener) in listeners.iter() {
                if !listener.events.contains(event.event_type()) {
                    continue;
                }

                // Since this is unbounded, this is always because the receiver
                // dropped.
                if listener.tx.unbounded_send(event.clone()).is_err() {
                    remove_listeners.push(*id);
                }
            }

            for id in &remove_listeners {
                listeners.remove(id);
            }

            listeners.clear();
        }
    }

    /// Identifies with the gateway to create a new session.
    async fn identify(&mut self) -> Result<()> {
        self.session.set_stage(Stage::Identifying);

        let identify = Identify::new(IdentifyInfo {
            compression: false,
            guild_subscriptions: true,
            large_threshold: 250,
            properties: self.properties.clone(),
            shard: Some(self.config.shard()),
            presence: None,
            token: self.config.token().to_owned(),
            v: 6,
        });

        self.send(identify).await
    }

    async fn process(&mut self, event: &GatewayEvent) -> Result<()> {
        use GatewayEvent::*;

        match event {
            Dispatch(seq, dispatch) => {
                self.session.set_seq(*seq);

                match dispatch.deref() {
                    DispatchEvent::Ready(ready) => {
                        self.session.set_stage(Stage::Connected);
                        self.session.set_id(&ready.session_id).await;
                    },
                    DispatchEvent::Resumed => {
                        self.session.set_stage(Stage::Connected);
                        self.session.heartbeats.receive();
                    },
                    _ => {},
                }
            },
            Heartbeat(seq) => {
                if *seq > self.session.seq() + 1 {
                    self.resume().await?;
                }

                if self.session.heartbeat().is_err() {
                    warn!("Error sending heartbeat; reconnecting");

                    self.reconnect().await;
                }
            },
            Hello(interval) => {
                self.session.set_stage(Stage::Identifying);

                if *interval > 0 {
                    self.session.set_heartbeat_interval(*interval);
                    self.session.start_heartbeater().await;
                }

                self.identify().await?;
            },
            HeartbeatAck => {
                self.session.heartbeats.receive();
            },
            InvalidateSession(true) => {
                self.resume().await?;
            },
            InvalidateSession(false) => {
                self.reconnect().await;
            },
            Reconnect => {
                self.reconnect().await;
            },
        }

        Ok(())
    }

    async fn reconnect(&mut self) {
        loop {
            self.config.queue.request().await;

            let shard = match Self::new(Arc::clone(&self.config.clone())).await {
                Ok(shard) => shard,
                Err(why) => {
                    warn!("Error reconnecting: {:?}", why);

                    continue;
                },
            };

            mem::replace(self, shard);
        }
    }

    async fn resume(&mut self) -> Result<()> {
        self.session.set_stage(Stage::Resuming);

        let id = if let Some(id) = self.session.id().await {
            id
        } else {
            self.reconnect().await;

            return Ok(());
        };

        let payload = Resume::new(self.session.seq(), id, self.config.token());

        self.send(payload).await?;

        Ok(())
    }

    pub async fn send(&mut self, payload: impl Serialize) -> Result<()> {
        match self.session.send(payload) {
            Ok(()) => Ok(()),
            Err(Error::PayloadSerialization {
                source,
            }) => {
                log::warn!("Failed to serialize message to send: {:?}", source);

                Err(Error::PayloadSerialization {
                    source,
                })
            },
            Err(Error::SendingMessage {
                source,
            }) => {
                log::warn!("Failed to send message: {:?}", source);
                log::info!("Reconnecting");

                self.reconnect().await;

                Ok(())
            },
            Err(other) => Err(other),
        }
    }
}
