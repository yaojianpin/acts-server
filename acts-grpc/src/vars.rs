use std::ops::Deref;

use serde_json::json;

pub struct Vars {
    pub(crate) inner: serde_json::Map<String, serde_json::Value>,
}

impl Deref for Vars {
    type Target = serde_json::Map<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Display for Vars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = serde_json::ser::to_string_pretty(&self.inner).expect("convert vars to string");
        f.write_str(&text)
    }
}

impl Vars {
    pub fn new() -> Self {
        Self {
            inner: serde_json::Map::new(),
        }
    }

    pub fn from_prost(value: &prost_types::Value) -> Self {
        Self {
            inner: utils::prost_to_json(value).as_object().unwrap().clone(),
        }
    }

    pub fn from_json(value: &serde_json::Map<String, serde_json::Value>) -> Self {
        Self {
            inner: value.clone(),
        }
    }
    // pub fn len(&self) -> usize {
    //     self.inner.len()
    // }
    pub fn json_vars(&self) -> serde_json::Map<String, serde_json::Value> {
        // let value = prost_types::Value {
        //     kind: Some(prost_types::value::Kind::StructValue(self.inner.clone())),
        // };
        // utils::prost_to_json(&value).as_object().unwrap().clone()

        self.inner.clone()
    }

    pub fn prost_vars(&self) -> prost_types::Value {
        let mut map = prost::alloc::collections::BTreeMap::new();
        for (k, v) in self.inner.iter() {
            map.insert(k.to_string(), utils::json_to_prost(v));
        }

        prost_types::Value {
            kind: Some(prost_types::value::Kind::StructValue(prost_types::Struct {
                fields: map,
            })),
        }
    }

    pub fn extend(&mut self, vars: &Vars) {
        for (k, v) in vars.inner.iter() {
            self.inner.insert(k.to_string(), v.clone());
        }
    }

    // pub fn value(&self, key: &str) -> Option<&prost_types::Value> {
    //     self.inner.fields.get(key)
    // }

    pub fn value_str(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|v| v.as_str().unwrap())
    }

    pub fn value_number(&self, key: &str) -> Option<f64> {
        self.inner.get(key).map(|v| v.as_f64().unwrap())
    }

    pub fn insert(&mut self, key: String, value: &serde_json::Value) {
        self.inner.insert(key, value.clone());
    }

    pub fn insert_str(&mut self, key: String, value: impl Into<String>) {
        self.inner.insert(key, json!(value.into()));
    }

    pub fn insert_number(&mut self, key: String, value: f64) {
        self.inner.insert(key, json!(value));
    }

    pub fn rm(&mut self, key: &str) {
        self.inner.remove(key);
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }
}

mod utils {
    type JsonValue = serde_json::Value;
    type ProstValue = prost_types::Value;
    use serde_json::json;
    use std::collections::BTreeMap;

    pub fn prost_to_json(v: &prost_types::Value) -> JsonValue {
        match &v.kind {
            Some(kind) => match kind {
                prost_types::value::Kind::NullValue(_) => JsonValue::Null,
                prost_types::value::Kind::NumberValue(v) => json!(v),
                prost_types::value::Kind::StringValue(v) => json!(v),
                prost_types::value::Kind::BoolValue(v) => json!(v),
                prost_types::value::Kind::StructValue(v) => {
                    let mut obj = serde_json::Map::new();
                    for (k, v) in v.fields.iter() {
                        obj.insert(k.to_string(), prost_to_json(v));
                    }
                    JsonValue::Object(obj)
                }
                prost_types::value::Kind::ListValue(list) => {
                    let mut arr = Vec::new();
                    for v in list.values.iter() {
                        arr.push(prost_to_json(v));
                    }

                    JsonValue::Array(arr)
                }
            },
            _ => JsonValue::Null,
        }
    }

    // pub fn as_f64(v: &prost_types::Value) -> Option<&f64> {
    //     match &v.kind {
    //         Some(kind) => match kind {
    //             prost_types::value::Kind::NumberValue(v) => Some(v),
    //             _ => None,
    //         },
    //         _ => None,
    //     }
    // }

    // pub fn as_str(v: &prost_types::Value) -> Option<&str> {
    //     match &v.kind {
    //         Some(kind) => match kind {
    //             prost_types::value::Kind::StringValue(v) => Some(v),
    //             _ => None,
    //         },
    //         _ => None,
    //     }
    // }

    pub fn json_to_prost(v: &JsonValue) -> ProstValue {
        match v {
            serde_json::Value::Null => ProstValue {
                kind: Some(prost_types::value::Kind::NullValue(0)),
            },
            serde_json::Value::Bool(v) => ProstValue {
                kind: Some(prost_types::value::Kind::BoolValue(v.clone())),
            },
            serde_json::Value::Number(v) => ProstValue {
                kind: Some(prost_types::value::Kind::NumberValue(v.as_f64().unwrap())),
            },
            serde_json::Value::String(v) => ProstValue {
                kind: Some(prost_types::value::Kind::StringValue(v.clone())),
            },
            serde_json::Value::Array(arr) => {
                let mut values = Vec::new();
                for v in arr {
                    values.push(json_to_prost(v));
                }
                ProstValue {
                    kind: Some(prost_types::value::Kind::ListValue(
                        prost_types::ListValue { values },
                    )),
                }
            }
            serde_json::Value::Object(obj) => {
                let mut fields = BTreeMap::new();
                for (k, v) in obj {
                    fields.insert(k.to_string(), json_to_prost(v));
                }
                ProstValue {
                    kind: Some(prost_types::value::Kind::StructValue(prost_types::Struct {
                        fields,
                    })),
                }
            }
        }
    }
}
