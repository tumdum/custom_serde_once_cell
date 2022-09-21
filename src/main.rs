use once_cell::sync::OnceCell;
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

mod simple {
    use super::*;

    #[derive(Serialize, Debug, PartialEq)]
    pub struct Outer {
        pub inner: Option<Inner>,
    }

    #[derive(Serialize, Debug, PartialEq)]
    pub struct Inner {
        pub f: f64,
        pub u: Uuid,
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct Outer {
    #[serde(deserialize_with = "crate::deserialize")]
    inner: OnceCell<Inner>,
}

#[derive(Debug, PartialEq)]
// #[derive(Debug, PartialEq, Deserialize)]
struct Inner {
    f: f64,
    u: Uuid,
}

fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<OnceCell<Inner>, D::Error> {
    const FIELDS: [&str; 2] = ["f", "u"];
    struct OnceCellVisitor;

    impl<'de> Visitor<'de> for OnceCellVisitor {
        type Value = OnceCell<Inner>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "OnceCell")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Default::default())
        }

        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            d.deserialize_struct("Inner", &FIELDS, InnerVisitor)
                .map(OnceCell::with_value)

            /* We can use generated Serialize here if we can modify Inner
            Ok(OnceCell::with_value(Inner::deserialize(d)?))
            */
        }
    }

    struct InnerVisitor;

    impl<'de> Visitor<'de> for InnerVisitor {
        type Value = Inner;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Inner")
        }

        fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
            let mut f: Option<f64> = None;
            let mut u: Option<Uuid> = None;
            while let Some(key) = map.next_key()? {
                match key {
                    "f" => {
                        f = Some(map.next_value()?);
                    }
                    "u" => {
                        u = Some(map.next_value()?);
                    }
                    unknown_field => return Err(V::Error::unknown_field(unknown_field, &FIELDS)),
                }
            }
            match (f, u) {
                (Some(f), Some(u)) => Ok(Inner { f, u }),
                (None, _) => Err(V::Error::missing_field("f")),
                (_, None) => Err(V::Error::missing_field("u")),
            }
        }
    }

    d.deserialize_option(OnceCellVisitor)
}

/*
#[serde_with::serde_as]
#[derive(Deserialize, Debug, PartialEq)]
struct Outer {
    #[serde_as(as = "OptionInner")]
    inner: OnceCell<Inner>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct Inner {
    f: f64,
    u: Uuid,
}

serde_with::serde_conv!(
    OptionInner,
    OnceCell<Inner>,
    |inner: &OnceCell<Inner>| inner.get().cloned(),
    |value: Option<Inner>| -> Result<_, std::convert::Infallible> {
        Ok(match value {
            Some(v) => OnceCell::with_value(v),
            None => OnceCell::new(),
        })
    }
);
*/

fn main() {
    let expected = Outer {
        inner: OnceCell::new(),
    };
    let simple = simple::Outer { inner: None };
    let input = serde_json::to_string(&simple).unwrap();
    let outer = serde_json::from_str(&input);
    println!("input: {input}");
    println!("outer: {outer:?}\n");
    assert_eq!(expected, outer.unwrap());

    let u = Uuid::new_v4();
    let expected = Outer {
        inner: OnceCell::with_value(Inner { f: 42.0, u }),
    };
    let simple = simple::Outer {
        inner: Some(simple::Inner { f: 42.0, u }),
    };
    let input = serde_json::to_string(&simple).unwrap();
    let outer = serde_json::from_str(&input);
    println!("input: {input}");
    println!("outer: {outer:?}\n");
    assert_eq!(expected, outer.unwrap());

    // Yaml
    let input = serde_yaml::to_string(&simple).unwrap();
    let outer = serde_yaml::from_str(&input);
    println!("input: {input}");
    println!("outer: {outer:?}\n");
    assert_eq!(expected, outer.unwrap());

    // Toml
    let input = toml::to_string(&simple).unwrap();
    let outer = toml::from_str(&input);
    println!("input: {input}");
    println!("outer: {outer:?}\n");
    assert_eq!(expected, outer.unwrap());

    // missing field
    let input = r#"{"inner": {"u": "c9959315-53eb-4f64-965a-957b4c1ef579"}}"#;
    let outer = serde_json::from_str::<Outer>(input).unwrap_err();
    println!("input: {input}");
    println!("outer: {outer:?}\n");

    // unexpected field
    let input =
        r#"{"inner": {"some": "field", "f": 3.14, "u": "c9959315-53eb-4f64-965a-957b4c1ef579"}}"#;
    let outer = serde_json::from_str::<Outer>(input).unwrap_err();
    println!("input: {input}");
    println!("outer: {outer:?}\n");

    // invalid value in inner
    let input = r#"{"inner": {"f": "3.14", "u": "c9959315-53eb-4f64-965a-957b4c1ef579"}}"#;
    let outer = serde_json::from_str::<Outer>(input).unwrap_err();
    println!("input: {input}");
    println!("outer: {outer:?}\n");

    // invalid value in inner
    let input = r#"{"inner": "oops"}"#;
    let outer = serde_json::from_str::<Outer>(input).unwrap_err();
    println!("input: {input}");
    println!("outer: {outer:?}");
}
