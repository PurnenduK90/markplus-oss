//! Frontmatter merging logic for included documents.
//!
//! When a Markdown file is included, its frontmatter metadata is merged into
//! the parent document's metadata. The merge follows these rules:
//!
//! - **Scalars**: parent value wins on conflict
//! - **Arrays**: values are concatenated and deduplicated
//! - **Objects**: recursively merged (parent wins on scalar conflicts)
//! - **Missing keys**: added from the included document

use serde_json::{Map, Value};

/// Merge `inner` frontmatter into `outer`. Outer wins on scalar conflicts.
pub fn merge_frontmatter(outer: &mut Option<Value>, inner: &Option<Value>) {
    let inner = match inner {
        Some(v) => v,
        None => return,
    };

    match outer {
        Some(Value::Object(ref mut outer_map)) => {
            if let Value::Object(inner_map) = inner {
                merge_maps(outer_map, inner_map);
            }
        }
        None => {
            *outer = Some(inner.clone());
        }
        _ => {} // outer is non-object, keep it
    }
}

fn merge_maps(outer: &mut Map<String, Value>, inner: &Map<String, Value>) {
    for (key, inner_val) in inner {
        match outer.get_mut(key) {
            Some(Value::Array(ref mut outer_arr)) => {
                if let Value::Array(inner_arr) = inner_val {
                    for item in inner_arr {
                        if !outer_arr.contains(item) {
                            outer_arr.push(item.clone());
                        }
                    }
                }
                // inner is not array but outer is → keep outer
            }
            Some(Value::Object(ref mut outer_obj)) => {
                if let Value::Object(inner_obj) = inner_val {
                    merge_maps(outer_obj, inner_obj);
                }
                // type mismatch → outer wins
            }
            Some(_) => {
                // Outer wins on scalar conflicts — do nothing
            }
            None => {
                outer.insert(key.clone(), inner_val.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_inner_none_is_noop() {
        let mut outer = Some(json!({"title": "A"}));
        merge_frontmatter(&mut outer, &None);
        assert_eq!(outer, Some(json!({"title": "A"})));
    }

    #[test]
    fn merge_outer_none_takes_inner() {
        let mut outer = None;
        merge_frontmatter(&mut outer, &Some(json!({"title": "B"})));
        assert_eq!(outer, Some(json!({"title": "B"})));
    }

    #[test]
    fn outer_scalar_wins_on_conflict() {
        let mut outer = Some(json!({"title": "Parent", "date": "2026-01-01"}));
        let inner = Some(json!({"title": "Child", "author": "Bob"}));
        merge_frontmatter(&mut outer, &inner);
        let m = outer.unwrap();
        assert_eq!(m["title"], "Parent"); // outer wins
        assert_eq!(m["date"], "2026-01-01"); // preserved
        assert_eq!(m["author"], "Bob"); // added from inner
    }

    #[test]
    fn arrays_concatenate_and_deduplicate() {
        let mut outer = Some(json!({"tags": ["a", "b"]}));
        let inner = Some(json!({"tags": ["b", "c"]}));
        merge_frontmatter(&mut outer, &inner);
        let tags = outer.unwrap()["tags"].as_array().unwrap().clone();
        assert_eq!(tags.len(), 3); // a, b, c — "b" not duplicated
        assert!(tags.contains(&json!("a")));
        assert!(tags.contains(&json!("b")));
        assert!(tags.contains(&json!("c")));
    }

    #[test]
    fn nested_objects_merge_recursively() {
        let mut outer = Some(json!({"config": {"theme": "dark", "font": "mono"}}));
        let inner = Some(json!({"config": {"theme": "light", "size": 14}}));
        merge_frontmatter(&mut outer, &inner);
        let cfg = &outer.unwrap()["config"];
        assert_eq!(cfg["theme"], "dark"); // outer wins
        assert_eq!(cfg["font"], "mono"); // preserved
        assert_eq!(cfg["size"], 14); // added from inner
    }

    #[test]
    fn both_none_stays_none() {
        let mut outer: Option<Value> = None;
        merge_frontmatter(&mut outer, &None);
        assert!(outer.is_none());
    }

    #[test]
    fn outer_non_object_stays() {
        let mut outer = Some(json!("just a string"));
        merge_frontmatter(&mut outer, &Some(json!({"key": "val"})));
        assert_eq!(outer, Some(json!("just a string")));
    }

    #[test]
    fn inner_non_object_ignored() {
        let mut outer = Some(json!({"title": "A"}));
        merge_frontmatter(&mut outer, &Some(json!("inner string")));
        assert_eq!(outer, Some(json!({"title": "A"})));
    }

    #[test]
    fn empty_objects_merge_cleanly() {
        let mut outer = Some(json!({}));
        let inner = Some(json!({"key": "val"}));
        merge_frontmatter(&mut outer, &inner);
        assert_eq!(outer, Some(json!({"key": "val"})));
    }
}
