use std::collections::HashMap;

use crate::types;

#[derive(Debug, Default, Clone)]
pub struct RosState {
    pub nodes: HashMap<(String, String), types::NodeProperties>,
    pub topics: HashMap<String, types::TopicProperties>,
    pub services: HashMap<String, types::ServiceProperties>,
}

impl RosState {
    pub fn update(&mut self, event: types::DiscoveryEvent) {
        match event {
            types::DiscoveryEvent::Ping => {}
            types::DiscoveryEvent::NodeAdded { name, namespace, properties } => {
                self.nodes.insert((name, namespace), properties);
            }
            types::DiscoveryEvent::NodeRemoved { name, namespace } => {
                self.nodes.remove(&(name, namespace));
            }
            types::DiscoveryEvent::TopicAdded { name, properties } => {
                self.topics.insert(name, properties);
            }
            types::DiscoveryEvent::TopicRemoved { name } => {
                self.topics.remove(&name);
            }
            types::DiscoveryEvent::ServiceAdded { name, properties } => {
                self.services.insert(name, properties);
            }
            types::DiscoveryEvent::ServiceRemoved { name } => {
                self.services.remove(&name);
            }
        }
    }

    pub fn changes(&self, prev: &Self) -> Vec<types::DiscoveryEvent> {
        let mut events = vec![];

        for node_key in prev.nodes.keys() {
            if !self.nodes.contains_key(node_key) {
                events.push(types::DiscoveryEvent::NodeRemoved {
                    name: node_key.0.clone(),
                    namespace: node_key.1.clone(),
                });
            }
        }

        for (node_key, node) in self.nodes.iter() {
            let prev_node = prev.nodes.get(node_key);
            let new_node = self.nodes.get(node_key);
            if prev_node != new_node {
                events.push(types::DiscoveryEvent::NodeAdded {
                    name: node_key.0.clone(),
                    namespace: node_key.1.clone(),
                    properties: node.clone(),
                });
            }
        }

        for topic_key in prev.topics.keys() {
            if !self.topics.contains_key(topic_key) {
                events.push(types::DiscoveryEvent::TopicRemoved {
                    name: topic_key.clone(),
                });
            }
        }

        for (topic_key, topic) in self.topics.iter() {
            let prev_topic = prev.topics.get(topic_key);
            let new_topic = self.topics.get(topic_key);
            if prev_topic != new_topic {
                events.push(types::DiscoveryEvent::TopicAdded {
                    name: topic_key.clone(),
                    properties: topic.clone(),
                });
            }
        }

        for service_key in prev.services.keys() {
            if !self.services.contains_key(service_key) {
                events.push(types::DiscoveryEvent::ServiceRemoved {
                    name: service_key.clone(),
                });
            }
        }

        for (service_key, service) in self.services.iter() {
            let prev_service = prev.services.get(service_key);
            let new_service = self.services.get(service_key);
            if prev_service != new_service {
                events.push(types::DiscoveryEvent::ServiceAdded {
                    name: service_key.clone(),
                    properties: service.clone(),
                });
            }
        }

        events
    }
}
