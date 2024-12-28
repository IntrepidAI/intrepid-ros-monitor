use std::collections::HashMap;

use ros_monitor_lib::state::RosState;
use ros_monitor_lib::types;

pub trait RosStateProvider {
    fn from_ros(node: &r2r::Node) -> Result<RosState, r2r::Error>;
}

impl RosStateProvider for RosState {
    fn from_ros(node: &r2r::Node) -> Result<RosState, r2r::Error> {
        let mut nodes = HashMap::new();
        for (name, namespace, enclave) in node.get_node_names_with_enclaves()? {
            let mut publishers = HashMap::new();
            for (topic, types) in node.get_publisher_names_and_types_by_node(&name, &namespace)? {
                if types.len() == 1 {
                    publishers.insert(topic, types[0].clone());
                } else {
                    eprintln!("publisher {} has multiple types: {:?}", topic, types);
                }
            }

            let mut subscribers = HashMap::new();
            for (topic, types) in node.get_subscriber_names_and_types_by_node(&name, &namespace)? {
                if types.len() == 1 {
                    subscribers.insert(topic, types[0].clone());
                } else {
                    eprintln!("subscriber {} has multiple types: {:?}", topic, types);
                }
            }

            let mut clients = HashMap::new();
            for (name, types) in node.get_client_names_and_types_by_node(&name, &namespace)? {
                if types.len() == 1 {
                    clients.insert(name, types[0].clone());
                } else {
                    eprintln!("client {} has multiple types: {:?}", name, types);
                }
            }

            let mut services = HashMap::new();
            for (name, types) in node.get_service_names_and_types_by_node(&name, &namespace)? {
                if types.len() == 1 {
                    services.insert(name, types[0].clone());
                } else {
                    eprintln!("service {} has multiple types: {:?}", name, types);
                }
            }

            nodes.insert((name, namespace), types::NodeProperties { enclave, publishers, subscribers, clients, services });
        }

        let mut topics = HashMap::new();
        for (name, types) in node.get_topic_names_and_types()? {
            let mut publishers = vec![];
            for endpoint_info in node.get_publishers_info_by_topic(&name, false)? {
                let r2r::TopicEndpointInfo { node_name, node_namespace, topic_type, endpoint_gid: _, qos_profile } = endpoint_info;
                publishers.push(types::PubSubProperties { node_name, node_namespace, topic_type, qos_profile: qos_into(qos_profile) });
            }

            let mut subscribers = vec![];
            for endpoint_info in node.get_subscriptions_info_by_topic(&name, false)? {
                let r2r::TopicEndpointInfo { node_name, node_namespace, topic_type, endpoint_gid: _, qos_profile } = endpoint_info;
                subscribers.push(types::PubSubProperties { node_name, node_namespace, topic_type, qos_profile: qos_into(qos_profile) });
            }

            topics.insert(name.clone(), types::TopicProperties { types, publishers, subscribers });
        }

        let mut services = HashMap::new();
        for (name, types) in node.get_service_names_and_types()? {
            services.insert(name, types::ServiceProperties { types });
        }

        Ok(Self { nodes, topics, services })
    }
}

fn qos_into(qos_profile: r2r::QosProfile) -> types::QosProfile {
    types::QosProfile {
        history: match qos_profile.history {
            r2r::qos::HistoryPolicy::KeepAll => types::HistoryPolicy::KeepAll,
            r2r::qos::HistoryPolicy::KeepLast => types::HistoryPolicy::KeepLast,
            r2r::qos::HistoryPolicy::SystemDefault => types::HistoryPolicy::SystemDefault,
            r2r::qos::HistoryPolicy::Unknown => types::HistoryPolicy::Unknown,
        },
        depth: qos_profile.depth,
        reliability: match qos_profile.reliability {
            r2r::qos::ReliabilityPolicy::BestEffort => types::ReliabilityPolicy::BestEffort,
            r2r::qos::ReliabilityPolicy::Reliable => types::ReliabilityPolicy::Reliable,
            r2r::qos::ReliabilityPolicy::SystemDefault => types::ReliabilityPolicy::SystemDefault,
            r2r::qos::ReliabilityPolicy::BestAvailable => types::ReliabilityPolicy::BestAvailable,
            r2r::qos::ReliabilityPolicy::Unknown => types::ReliabilityPolicy::Unknown,
        },
        durability: match qos_profile.durability {
            r2r::qos::DurabilityPolicy::TransientLocal => types::DurabilityPolicy::TransientLocal,
            r2r::qos::DurabilityPolicy::Volatile => types::DurabilityPolicy::Volatile,
            r2r::qos::DurabilityPolicy::SystemDefault => types::DurabilityPolicy::SystemDefault,
            r2r::qos::DurabilityPolicy::BestAvailable => types::DurabilityPolicy::BestAvailable,
            r2r::qos::DurabilityPolicy::Unknown => types::DurabilityPolicy::Unknown,
        },
        deadline: qos_profile.deadline,
        lifespan: qos_profile.lifespan,
        liveliness: match qos_profile.liveliness {
            r2r::qos::LivelinessPolicy::Automatic => types::LivelinessPolicy::Automatic,
            r2r::qos::LivelinessPolicy::ManualByNode => types::LivelinessPolicy::ManualByNode,
            r2r::qos::LivelinessPolicy::ManualByTopic => types::LivelinessPolicy::ManualByTopic,
            r2r::qos::LivelinessPolicy::SystemDefault => types::LivelinessPolicy::SystemDefault,
            r2r::qos::LivelinessPolicy::BestAvailable => types::LivelinessPolicy::BestAvailable,
            r2r::qos::LivelinessPolicy::Unknown => types::LivelinessPolicy::Unknown,
        },
        liveliness_lease_duration: qos_profile.liveliness_lease_duration,
    }
}
