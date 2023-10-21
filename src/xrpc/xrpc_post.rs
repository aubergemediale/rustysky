use chrono::{DateTime, Utc};
use regex::Regex;
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub collection: String, // e.g. "app.bsky.feed.post" for posts
    pub repo: String,       // the did
    pub record: Post,
}

impl CreatePostRequest {
    pub fn new(did: &str, post: Post) -> Self {
        Self {
            collection: "app.bsky.feed.post".to_string(),
            repo: did.to_string(),
            record: post,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostResponse {
    pub uri: String,
    pub cid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "$type")]
    pub record_type: String,
    #[serde(skip_serializing)]
    pub created_utc: chrono::DateTime<Utc>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<Facet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub langs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<SelfLabels>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelfLabels {
    #[serde(rename = "$type")]
    pub selflabels_type: String,
    pub values: Vec<SelfLabel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelfLabel {
    pub val: String,
}

impl Post {
    pub fn new(
        text: &str,
        debug_did: &str,
        langs: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        labels: Option<SelfLabels>,
    ) -> anyhow::Result<Self> {
        if text.is_empty() {
            return Err(anyhow::anyhow!("The Post's text cannot be empty"));
        }

        let now = Utc::now();
        let mut facets: Vec<Facet> = Vec::new();

        // Extract mentions and URLs
        let mentions = parse_mentions(text);
        let urls = parse_urls(text);
        let hashtags = parse_hashtags(text);

        if !mentions.is_empty() {
            let mention_features: Vec<Feature> = mentions
                .into_iter()
                .map(|span| Feature::Mention {
                    index: FacetIndex {
                        byteStart: span.start,
                        byteEnd: span.end,
                    },
                    features: vec![MentionFeature {
                        feature_type: "app.bsky.richtext.facet#mention".to_string(),
                        did: debug_did.to_string(),
                    }],
                })
                .collect();

            facets.push(Facet {
                index: FacetIndex {
                    byteStart: 0,
                    byteEnd: text.len(),
                },
                features: mention_features,
            });
        }

        if !urls.is_empty() {
            let url_features: Vec<Feature> = urls
                .into_iter()
                .map(|span| Feature::Link {
                    index: FacetIndex {
                        byteStart: span.start,
                        byteEnd: span.end,
                    },
                    features: vec![LinkFeature {
                        feature_type: "app.bsky.richtext.facet#link".to_string(),
                        uri: span.url,
                    }],
                })
                .collect();

            facets.push(Facet {
                index: FacetIndex {
                    byteStart: 0,
                    byteEnd: text.len(),
                },
                features: url_features,
            });
        }

        if !hashtags.is_empty() {
            let url_features: Vec<Feature> = hashtags
                .into_iter()
                .map(|span| Feature::Hashtag {
                    index: FacetIndex {
                        byteStart: span.start,
                        byteEnd: span.end,
                    },
                    features: vec![HashtagFeature {
                        feature_type: "app.bsky.richtext.facet#tag".to_string(),
                        tag: span.tag,
                    }],
                })
                .collect();

            facets.push(Facet {
                index: FacetIndex {
                    byteStart: 0,
                    byteEnd: text.len(),
                },
                features: url_features,
            });
        }

        Ok(Self {
            record_type: "app.bsky.feed.post".to_string(),
            created_utc: now,
            created_at: date_utc_as_iso8601(now),
            text: text.to_string(),
            facets: if facets.is_empty() {
                None
            } else {
                Some(facets)
            },
            langs,
            tags,
            labels,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Facet {
    pub index: FacetIndex,
    pub features: Vec<Feature>,
}
impl Serialize for Facet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Facet", 2)?;
        match &self.features[0] {
            // assuming there's always at least one feature
            Feature::Link { index, features } => {
                s.serialize_field("index", index)?;
                s.serialize_field("features", features)?;
            }
            Feature::Mention { index, features } => {
                s.serialize_field("index", index)?;
                s.serialize_field("features", features)?;
            }
            Feature::Hashtag { index, features } => {
                s.serialize_field("index", index)?;
                s.serialize_field("features", features)?;
            }
        }
        s.end()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Feature {
    Link {
        index: FacetIndex,
        features: Vec<LinkFeature>,
    },
    Mention {
        index: FacetIndex,
        features: Vec<MentionFeature>,
    },
    Hashtag {
        index: FacetIndex,
        features: Vec<HashtagFeature>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkFeature {
    #[serde(rename = "$type")]
    feature_type: String,
    uri: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentionFeature {
    #[serde(rename = "$type")]
    feature_type: String,
    did: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HashtagFeature {
    #[serde(rename = "$type")]
    feature_type: String,
    tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct FacetIndex {
    byteStart: usize,
    byteEnd: usize,
}

pub fn date_utc_as_iso8601(date_utc: DateTime<Utc>) -> String {
    date_utc.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn parse_mentions(text: &str) -> Vec<MentionSpan> {
    let mention_regex = r"(?i)[\$|\W](@([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)";
    let re = Regex::new(mention_regex).unwrap();

    re.captures_iter(text)
        .map(|cap| MentionSpan {
            start: cap.get(1).unwrap().start(),
            end: cap.get(1).unwrap().end(),
            handle: cap.get(1).unwrap().as_str().chars().skip(1).collect(),
        })
        .collect()
}

fn parse_hashtags(text: &str) -> Vec<HashTagSpan> {
    let tag_regex = r"\B#([a-zA-Z\p{M}][a-zA-Z0-9\p{M}_]{0,59})";

    let re = Regex::new(tag_regex).unwrap();

    re.captures_iter(text)
        .map(|cap| HashTagSpan {
            start: cap.get(1).unwrap().start(),
            end: cap.get(1).unwrap().end(),
            tag: cap.get(1).unwrap().as_str().to_string(),
        })
        .collect()
}

fn parse_urls(text: &str) -> Vec<URLSpan> {
    let url_regex = r"(?i)[\$|\W](https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*[-a-zA-Z0-9@%_\+~#//=])?)";
    let re = Regex::new(url_regex).unwrap();

    re.captures_iter(text)
        .map(|cap| URLSpan {
            start: cap.get(1).unwrap().start(),
            end: cap.get(1).unwrap().end(),
            url: cap.get(1).unwrap().as_str().to_string(),
        })
        .collect()
}

#[derive(Debug, Clone)]
struct MentionSpan {
    start: usize,
    end: usize,
    #[allow(dead_code)]
    handle: String,
}

#[derive(Debug, Clone)]
struct URLSpan {
    start: usize,
    end: usize,
    url: String,
}

#[derive(Debug, Clone)]
struct HashTagSpan {
    start: usize,
    end: usize,
    tag: String,
}
