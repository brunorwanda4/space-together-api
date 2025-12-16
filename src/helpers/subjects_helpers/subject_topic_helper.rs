use crate::domain::subjects::subject_topic::SubjectTopicWithOthers;
use std::collections::HashMap;

/// Sort and nest topics recursively
pub fn sort_learning_outcome_topics_typed(topics: &mut Vec<SubjectTopicWithOthers>) {
    let mut topic_map: HashMap<String, SubjectTopicWithOthers> = HashMap::new();
    let mut children_map: HashMap<String, Vec<SubjectTopicWithOthers>> = HashMap::new();

    // Index topics by id and build parent → children map
    for topic in topics.drain(..) {
        let id = topic.topic.id.clone().unwrap().to_string();
        let parent_id = topic
            .topic
            .parent_topic_id
            .clone()
            .map(|oid| oid.to_string());

        topic_map.insert(id.clone(), topic.clone());

        if let Some(pid) = parent_id {
            children_map.entry(pid).or_default().push(topic);
        }
    }

    // Recursive function to attach children
    fn attach_children(
        topic: &mut SubjectTopicWithOthers,
        children_map: &HashMap<String, Vec<SubjectTopicWithOthers>>,
    ) {
        let id = topic.topic.id.clone().unwrap().to_string();

        if let Some(children) = children_map.get(&id) {
            let mut sorted_children = children.clone();
            sorted_children.sort_by(|a, b| a.topic.order.partial_cmp(&b.topic.order).unwrap());

            let attached: Vec<SubjectTopicWithOthers> = sorted_children
                .into_iter()
                .map(|mut c| {
                    attach_children(&mut c, children_map);
                    c
                })
                .collect();

            topic.sub_topics = Some(attached);
        } else {
            topic.sub_topics = None;
        }
    }

    // Build roots (topics whose parent is None or parent not in topic_map)
    let mut roots: Vec<SubjectTopicWithOthers> = topic_map
        .values()
        .filter(|t| match &t.topic.parent_topic_id {
            Some(pid) => !topic_map.contains_key(&pid.to_string()),
            None => true,
        })
        .cloned()
        .collect();

    // Attach children recursively
    for root in &mut roots {
        attach_children(root, &children_map);
    }

    // Sort roots by order
    roots.sort_by(|a, b| a.topic.order.partial_cmp(&b.topic.order).unwrap());

    *topics = roots;
}
